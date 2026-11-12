//! The kept part of the health loop (ruling 42): the battery poll every 30 s and the heat
//! toasts while a 3D game plays. Ported from the deleted `tether_health.rs`.

use flat3d_inject::api::EventKind;
use flat3d_inject::exec::LinkRunner;
use flat3d_inject::watch::HealthLoop;

struct Fake {
    battery: String,
    ran: Vec<String>,
}

impl LinkRunner for Fake {
    fn run(&mut self, argv: &[String]) -> Result<String, String> {
        let line = argv.join(" ");
        self.ran.push(line.clone());
        if line == "adb shell dumpsys battery" {
            Ok(self.battery.clone())
        } else {
            Err("no reply".into())
        }
    }
}

fn fake(temperature: &str) -> Fake {
    Fake {
        battery: format!("level: 80\ntemperature: {temperature}\n"),
        ran: Vec::new(),
    }
}

#[test]
fn battery_is_polled_every_30_s() {
    let mut fake = fake("300");
    let mut loop_ = HealthLoop::new(None);
    for t in (0..=60).step_by(5) {
        loop_.tick(t, true, &mut fake);
    }
    assert_eq!(fake.ran, vec!["adb shell dumpsys battery"; 3]);
}

#[test]
fn warm_toasts_once_while_playing() {
    let mut fake = fake("395");
    let mut loop_ = HealthLoop::new(None);
    assert_eq!(
        loop_.tick(0, true, &mut fake).toast,
        Some((
            EventKind::Warm,
            "Headset is warm: 39.5 C. It may slow down.".into()
        ))
    );
    assert_eq!(loop_.tick(30, true, &mut fake).toast, None);
}

#[test]
fn no_toast_while_not_playing() {
    let mut fake = fake("395");
    let mut loop_ = HealthLoop::new(None);
    assert_eq!(loop_.tick(0, false, &mut fake).toast, None);
}
