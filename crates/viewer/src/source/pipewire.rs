use crate::frame::{Frame, FrameData, Plane};
use crate::source::FrameSource;
use ::pipewire as pw;
use pw::spa::{self, pod::Pod};
use std::os::fd::{BorrowedFd, OwnedFd};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const XR24: u32 = u32::from_le_bytes(*b"XR24");
const AR24: u32 = u32::from_le_bytes(*b"AR24");
const XB24: u32 = u32::from_le_bytes(*b"XB24");
const AB24: u32 = u32::from_le_bytes(*b"AB24");
const NV12: u32 = u32::from_le_bytes(*b"NV12");

pub enum Remote {
    Default,
    Fd(OwnedFd),
}
pub enum Target {
    NodeName(String),
    NodeId(u32),
}
pub struct Offer {
    pub fourccs: Vec<u32>,
    pub modifiers: Vec<u64>,
    pub max_w: u32,
    pub max_h: u32,
}
pub struct PwSource {
    rx: Receiver<Result<Frame, String>>,
    stop: pw::channel::Sender<()>,
    thread: Option<JoinHandle<()>>,
}

fn spa_format(value: u32) -> Option<spa::param::video::VideoFormat> {
    use spa::param::video::VideoFormat as V;
    match value {
        XR24 => Some(V::BGRx),
        AR24 => Some(V::BGRA),
        XB24 => Some(V::RGBx),
        AB24 => Some(V::RGBA),
        NV12 => Some(V::NV12),
        _ => None,
    }
}
fn fourcc_of(value: spa::param::video::VideoFormat) -> Option<u32> {
    use spa::param::video::VideoFormat as V;
    match value {
        V::BGRx => Some(XR24),
        V::BGRA => Some(AR24),
        V::RGBx => Some(XB24),
        V::RGBA => Some(AB24),
        V::NV12 => Some(NV12),
        _ => None,
    }
}
fn pod_bytes(object: spa::pod::Object) -> Result<Vec<u8>, String> {
    spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &spa::pod::Value::Object(object),
    )
    .map(|v| v.0.into_inner())
    .map_err(|e| e.to_string())
}
fn format_params(offer: &Offer) -> Vec<Vec<u8>> {
    use spa::{
        param::{
            format::{self, FormatProperties as F},
            ParamType,
        },
        pod::{ChoiceValue, PropertyFlags, Value},
        utils::{Choice, ChoiceEnum, ChoiceFlags, Fraction, Id, Rectangle},
    };
    let formats: Vec<_> = offer
        .fourccs
        .iter()
        .filter_map(|f| spa_format(*f))
        .collect();
    if formats.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let choices = if offer.modifiers.is_empty() {
        vec![false]
    } else {
        vec![true, false]
    };
    for with_modifier in choices {
        let mut props = vec![
            spa::pod::property!(F::MediaType, Id, format::MediaType::Video),
            spa::pod::property!(F::MediaSubtype, Id, format::MediaSubtype::Raw),
            spa::pod::property!(
                F::VideoFormat,
                Value::Choice(ChoiceValue::Id(Choice(
                    ChoiceFlags::empty(),
                    ChoiceEnum::Enum {
                        default: Id(formats[0].as_raw()),
                        alternatives: formats.iter().map(|f| Id(f.as_raw())).collect()
                    }
                )))
            ),
            spa::pod::property!(
                F::VideoSize,
                Value::Choice(ChoiceValue::Rectangle(Choice(
                    ChoiceFlags::empty(),
                    ChoiceEnum::Range {
                        default: Rectangle {
                            width: 1,
                            height: 1
                        },
                        min: Rectangle {
                            width: 1,
                            height: 1
                        },
                        max: Rectangle {
                            width: offer.max_w,
                            height: offer.max_h
                        }
                    }
                )))
            ),
            spa::pod::property!(
                F::VideoFramerate,
                Value::Choice(ChoiceValue::Fraction(Choice(
                    ChoiceFlags::empty(),
                    ChoiceEnum::Range {
                        default: Fraction { num: 0, denom: 1 },
                        min: Fraction { num: 0, denom: 1 },
                        max: Fraction {
                            num: 1000,
                            denom: 1
                        }
                    }
                )))
            ),
        ];
        if with_modifier {
            let values = offer
                .modifiers
                .iter()
                .map(|m| *m as i64)
                .collect::<Vec<_>>();
            let flags = PropertyFlags::MANDATORY
                | PropertyFlags::from_bits_retain(spa::sys::SPA_POD_PROP_FLAG_DONT_FIXATE);
            props.push(spa::pod::Property {
                key: F::VideoModifier.as_raw(),
                flags,
                value: Value::Choice(ChoiceValue::Long(Choice(
                    ChoiceFlags::empty(),
                    ChoiceEnum::Enum {
                        default: values[0],
                        alternatives: values,
                    },
                ))),
            });
        }
        let object = spa::pod::Object {
            type_: spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
            id: ParamType::EnumFormat.as_raw(),
            properties: props,
        };
        if let Ok(bytes) = pod_bytes(object) {
            result.push(bytes);
        }
    }
    result
}
fn buffers_data_type(dmabuf: bool) -> i32 {
    if dmabuf {
        1 << spa::buffer::DataType::DmaBuf.as_raw()
    } else {
        (1 << spa::buffer::DataType::MemFd.as_raw()) | (1 << spa::buffer::DataType::MemPtr.as_raw())
    }
}
fn skip_chunk(size: u32, flags: spa::buffer::ChunkFlags) -> bool {
    size == 0 || flags.contains(spa::buffer::ChunkFlags::CORRUPTED)
}

pub fn open(remote: Remote, target: Target, offer: &Offer) -> Result<PwSource, String> {
    pw::init();
    let offer = Offer {
        fourccs: offer.fourccs.clone(),
        modifiers: offer.modifiers.clone(),
        max_w: offer.max_w,
        max_h: offer.max_h,
    };
    let (target, target_name) = match target {
        Target::NodeId(id) => (Some(id), None),
        Target::NodeName(name) => (None, Some(name)),
    };
    let (tx, rx) = mpsc::sync_channel(2);
    let (stop, stop_rx) = pw::channel::channel();
    let stop_thread = stop.clone();
    let thread = thread::Builder::new().name("flat3d-pipewire".into()).spawn(move || {
        let run = || -> Result<(), String> {
            let ml = pw::main_loop::MainLoopRc::new(None).map_err(|e| e.to_string())?;
            let context = pw::context::ContextRc::new(&ml, None).map_err(|e| e.to_string())?;
            let core = match remote { Remote::Default => context.connect_rc(None), Remote::Fd(fd) => context.connect_fd_rc(fd, None) }.map_err(|e| e.to_string())?;
            let mut props = pw::properties::properties! { *pw::keys::MEDIA_TYPE => "Video", *pw::keys::MEDIA_CATEGORY => "Capture", *pw::keys::MEDIA_ROLE => "Screen" };
            if let Some(name) = target_name { props.insert("target.object", name); }
            let stream = pw::stream::StreamBox::new(&core, "flat3d-viewer", props).map_err(|e| e.to_string())?;
            let (format_tx, format_rx) = mpsc::channel::<(u32,u32,u32,u64,bool)>();
            let offered_modifiers = offer.modifiers.clone();
            let tx_cb = tx.clone();
            let tx_process = tx.clone();
            let mut seq = 0u64;
            let _listener = stream.add_local_listener::<()>()
                .state_changed(move |_,_,_,state| if let pw::stream::StreamState::Error(error) = state { let _ = tx_cb.try_send(Err(error)); })
                .param_changed({ let format_tx = format_tx.clone(); move |stream,_,id,param| {
                    if id != spa::param::ParamType::Format.as_raw() { return; }
                    let Some(param) = param else { return; };
                    if let (Some(modifier), Ok((_, spa::pod::Value::Object(mut object)))) = (
                        offered_modifiers.first().copied(),
                        spa::pod::deserialize::PodDeserializer::deserialize_from::<spa::pod::Value>(param.as_bytes()),
                    ) {
                        let fixated = if let Some(property) = object.properties.iter_mut().find(|p| p.key == spa::param::format::FormatProperties::VideoModifier.as_raw()) {
                            if matches!(property.value, spa::pod::Value::Choice(spa::pod::ChoiceValue::Long(_))) {
                                property.value = spa::pod::Value::Long(modifier as i64);
                                true
                            } else { false }
                        } else { false };
                        if fixated {
                            if let Ok(bytes) = pod_bytes(object) {
                                if let Some(pod) = Pod::from_bytes(&bytes) {
                                    let mut params = [pod];
                                    let _ = stream.update_params(&mut params);
                                }
                            }
                        }
                    }
                    let mut info = spa::param::video::VideoInfoRaw::new();
                    if info.parse(param).is_err() { return; }
                    let Some(fourcc) = fourcc_of(info.format()) else { return; };
                    let modifier = info.modifier();
                    let dmabuf = modifier != u64::MAX;
                    let _ = format_tx.send((info.size().width, info.size().height, fourcc, modifier, dmabuf));
                    let ty = buffers_data_type(dmabuf);
                    let obj = spa::pod::Object { type_: spa::utils::SpaTypes::ObjectParamBuffers.as_raw(), id: spa::param::ParamType::Buffers.as_raw(), properties: vec![spa::pod::Property { key: spa::sys::SPA_PARAM_BUFFERS_dataType, flags: spa::pod::PropertyFlags::empty(), value: spa::pod::Value::Int(ty) }] };
                    if let Ok(bytes) = pod_bytes(obj) { if let Some(pod) = Pod::from_bytes(&bytes) { let mut params = [pod]; let _ = stream.update_params(&mut params); } }
                }})
                .process({ let mut current_format = None; move |stream,_| {
                    if let Some(format) = format_rx.try_iter().last() { current_format = Some(format); }
                    let Some(mut buffer) = stream.dequeue_buffer() else { return; };
                    let Some((width,height,fourcc,modifier,dmabuf)) = current_format else { return; };
                    let frame_data = if dmabuf {
                        let mut planes = Vec::new();
                        for data in buffer.datas_mut().iter() {
                            let chunk = data.chunk();
                            if data.type_() != spa::buffer::DataType::DmaBuf || skip_chunk(chunk.size(), chunk.flags()) { return; }
                            let fd = match unsafe { BorrowedFd::borrow_raw(data.fd()).try_clone_to_owned() } {
                                Ok(fd) => fd,
                                Err(error) => { let _ = tx_process.try_send(Err(error.to_string())); return; }
                            };
                            planes.push(Plane { fd, offset: chunk.offset(), stride: chunk.stride().max(0) as u32 });
                        }
                        FrameData::DmaBuf { planes, modifier }
                    } else {
                        let Some(data) = buffer.datas_mut().first_mut() else { return; };
                        let chunk = data.chunk();
                        let size = chunk.size();
                        if skip_chunk(size, chunk.flags()) { return; }
                        let offset = chunk.offset() as usize;
                        let stride = chunk.stride().max(0) as u32;
                        let Some(bytes) = data.data() else { return; };
                        let Some(end) = offset.checked_add(size as usize).filter(|end| *end <= bytes.len()) else { return; };
                        FrameData::Mem { stride, bytes: bytes[offset..end].to_vec() }
                    };
                    let frame = Frame { width, height, fourcc, seq, data: frame_data }; seq += 1;
                    let _ = tx_process.try_send(Ok(frame));
                }}).register().map_err(|e| e.to_string())?;
            let bytes = format_params(&offer);
            let pods = bytes.iter().filter_map(|b| Pod::from_bytes(b)).collect::<Vec<_>>();
            let mut refs = pods.to_vec();
            stream.connect(spa::utils::Direction::Input, target, pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS, &mut refs).map_err(|e| e.to_string())?;
            let stop_loop = ml.clone();
            let _stop = stop_rx.attach(ml.loop_(), move |_| stop_loop.quit());
            ml.run();
            Ok(())
        };
        if let Err(error) = run() { let _ = tx.try_send(Err(error)); }
    }).map_err(|e| e.to_string())?;
    Ok(PwSource {
        rx,
        stop: stop_thread,
        thread: Some(thread),
    })
}

pub fn find_node(remote: Remote, name: &str, wait: Duration) -> Result<Option<u32>, String> {
    pw::init();
    let ml = pw::main_loop::MainLoopRc::new(None).map_err(|e| e.to_string())?;
    let context = pw::context::ContextRc::new(&ml, None).map_err(|e| e.to_string())?;
    let core = match remote {
        Remote::Default => context.connect_rc(None),
        Remote::Fd(fd) => context.connect_fd_rc(fd, None),
    }
    .map_err(|e| e.to_string())?;
    let found = std::rc::Rc::new(std::cell::Cell::new(None));
    let name = name.to_owned();
    let f = found.clone();
    let registry = core.get_registry().map_err(|e| e.to_string())?;
    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            if global.type_ == pw::types::ObjectType::Node
                && global.props.as_ref().and_then(|p| p.get("node.name")) == Some(name.as_str())
            {
                f.set(Some(global.id));
            }
        })
        .register();
    core.sync(0).map_err(|e| e.to_string())?;
    let timeout_loop = ml.clone();
    let timer = ml.loop_().add_timer(move |_| timeout_loop.quit());
    timer
        .update_timer(Some(wait), None)
        .into_sync_result()
        .map_err(|e| e.to_string())?;
    ml.run();
    Ok(found.get())
}

impl FrameSource for PwSource {
    fn next(&mut self, timeout: Duration) -> Result<Option<Frame>, String> {
        match self.rx.recv_timeout(timeout) {
            Ok(value) => value.map(Some),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err("PipeWire stream stopped".into()),
        }
    }
}
impl Drop for PwSource {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spa::pod::{ChoiceValue, PropertyFlags, Value};

    fn offer(modifiers: Vec<u64>) -> Offer {
        Offer {
            fourccs: vec![XR24, AR24, XB24, AB24, NV12, 123],
            modifiers,
            max_w: 3840,
            max_h: 2160,
        }
    }
    fn object(bytes: &[u8]) -> spa::pod::Object {
        let (_, value) =
            spa::pod::deserialize::PodDeserializer::deserialize_from::<Value>(bytes).unwrap();
        match value {
            Value::Object(object) => object,
            _ => panic!("expected pod object"),
        }
    }

    #[test]
    fn fourcc_map_round_trips_supported_formats() {
        for fourcc in [XR24, AR24, XB24, AB24, NV12] {
            assert_eq!(fourcc_of(spa_format(fourcc).unwrap()), Some(fourcc));
        }
        assert_eq!(spa_format(0), None);
    }

    #[test]
    fn format_params_advertise_shm_and_modifier_choices() {
        let plain = format_params(&offer(Vec::new()));
        assert_eq!(plain.len(), 1);
        let plain = object(&plain[0]);
        assert!(!plain
            .properties
            .iter()
            .any(|p| p.key == spa::param::format::FormatProperties::VideoModifier.as_raw()));
        let mut modified = offer(vec![7, 11]);
        let pods = format_params(&modified);
        assert_eq!(pods.len(), 2);
        let first = object(&pods[0]);
        let property = first
            .properties
            .iter()
            .find(|p| p.key == spa::param::format::FormatProperties::VideoModifier.as_raw())
            .unwrap();
        assert_eq!(
            property.flags,
            PropertyFlags::MANDATORY
                | PropertyFlags::from_bits_retain(spa::sys::SPA_POD_PROP_FLAG_DONT_FIXATE)
        );
        assert!(
            matches!(&property.value, Value::Choice(ChoiceValue::Long(choice)) if choice.1 == spa::utils::ChoiceEnum::Enum { default: 7, alternatives: vec![7, 11] })
        );
        let formats = first
            .properties
            .iter()
            .find(|p| p.key == spa::param::format::FormatProperties::VideoFormat.as_raw())
            .unwrap();
        assert!(
            matches!(&formats.value, Value::Choice(ChoiceValue::Id(choice)) if choice.1 == spa::utils::ChoiceEnum::Enum { default: spa::utils::Id(spa::param::video::VideoFormat::BGRx.as_raw()), alternatives: vec![spa::utils::Id(spa::param::video::VideoFormat::BGRx.as_raw()), spa::utils::Id(spa::param::video::VideoFormat::BGRA.as_raw()), spa::utils::Id(spa::param::video::VideoFormat::RGBx.as_raw()), spa::utils::Id(spa::param::video::VideoFormat::RGBA.as_raw()), spa::utils::Id(spa::param::video::VideoFormat::NV12.as_raw())] })
        );
        modified.fourccs = vec![123];
        assert!(format_params(&modified).is_empty());
    }

    #[test]
    fn buffers_types_match_pipewire_data_flags() {
        assert_eq!(
            buffers_data_type(true),
            1 << spa::buffer::DataType::DmaBuf.as_raw()
        );
        assert_eq!(
            buffers_data_type(false),
            (1 << spa::buffer::DataType::MemFd.as_raw())
                | (1 << spa::buffer::DataType::MemPtr.as_raw())
        );
    }

    #[test]
    fn corrupted_and_empty_chunks_are_skipped() {
        assert!(skip_chunk(0, spa::buffer::ChunkFlags::empty()));
        assert!(skip_chunk(12, spa::buffer::ChunkFlags::CORRUPTED));
        assert!(!skip_chunk(12, spa::buffer::ChunkFlags::empty()));
    }
}
