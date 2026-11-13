use crate::headset_page::{HeadsetState, Msg};
use flat3d_inject::face::link_model::link_view;
use iced::widget::{button, column, text, toggler};
use iced::{Color, Element};

pub fn view(s: &HeadsetState, now: u64) -> Element<'_, Msg> {
    let v = link_view(s.snapshot.clone(), now);
    let health_colour = v.health.colour.parse::<Color>().unwrap_or(Color::WHITE);
    let mut content = column![
        text(v.label),
        text(v.hint),
        text(format!(
            "Battery: {}   Temperature: {}",
            v.health.battery, v.health.temp
        ))
        .color(health_colour),
    ]
    .spacing(10);
    // Read-only: Quest Tether's state as one line. No button, no switch, no hint.
    content = content.push(text(v.quest_tether));
    if v.start && !s.busy {
        content = content.push(button("Start 3D").on_press(Msg::StartPressed));
    }
    if v.stop && !s.busy {
        content = content.push(button("Stop 3D").on_press(Msg::StopPressed));
    }
    content = content.push(
        toggler(v.auto_3d)
            .label("Auto 3D")
            .on_toggle(Msg::SetAuto3d),
    );
    if let Some(error) = &s.error {
        content = content.push(text(error));
    }
    content.into()
}
