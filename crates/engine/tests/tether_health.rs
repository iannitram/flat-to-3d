use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use flat3d_inject::api::{EventKind, TetherMode, Zone};
use flat3d_inject::exec::RelayRunner;
use flat3d_inject::tether::heal::Heal;
use flat3d_inject::watch::HealthLoop;

#[derive(Default)]
struct Fake {
    replies: HashMap<String, VecDeque<Result<String, String>>>,
    ran: Vec<Vec<String>>,
}

impl Fake {
    fn set(&mut self, key: &str, values: impl IntoIterator<Item = Result<String, String>>) {
        self.replies
            .insert(key.into(), values.into_iter().collect());
    }

    fn put(&mut self, key: &str, value: &str) {
        self.set(key, [Ok(value.into())]);
    }
}

impl RelayRunner for Fake {
    fn run(&mut self, argv: &[String]) -> Result<String, String> {
        self.ran.push(argv.to_vec());
        let key = argv.join(" ");
        let Some(queue) = self.replies.get_mut(&key) else {
            return Err("no reply".into());
        };
        if queue.len() > 1 {
            queue.pop_front().unwrap()
        } else {
            queue
                .front()
                .cloned()
                .unwrap_or_else(|| Err("no reply".into()))
        }
    }

    fn start_relay(&mut self, argv: &[String], name: &str) -> Result<(), String> {
        let mut words = vec!["relay".into()];
        words.extend_from_slice(argv);
        words.push(name.into());
        self.ran.push(words);
        Ok(())
    }

    fn stop_relay(&mut self) -> Result<(), String> {
        self.ran.push(vec!["relay".into(), "stop".into()]);
        Ok(())
    }
}

fn good(fake: &mut Fake) {
    fake.put("ps -eo pid=,args=", "7 /x/gnirehtet run SER");
    fake.put(
        "adb shell ip addr show tun0",
        "    inet 10.0.0.2/30 scope global tun0",
    );
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
}

fn netdev(bytes: u64) -> String {
    format!("Inter-| Receive | Transmit\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo frame compressed multicast\n tun0: {bytes} 0 0 0 0 0 0 0 {bytes} 0 0 0 0 0 0 0\n")
}

fn temp_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

fn tick(loop_: &mut HealthLoop, t: u64, fake: &mut Fake) -> flat3d_inject::watch::HealthTick {
    loop_.tick(t, TetherMode::Own, true, fake)
}

#[test]
fn a_good_tunnel_fills_health_stats_and_tunnel() {
    let mut fake = Fake::default();
    good(&mut fake);
    fake.set(
        "adb shell cat /proc/net/dev",
        [Ok(netdev(1_000)), Ok(netdev(6_000))],
    );
    let mut loop_ = HealthLoop::new(0, 0, None);
    let first = tick(&mut loop_, 0, &mut fake);
    assert!(first.tunnel);
    assert!(first.stats.is_some());
    assert_eq!(first.health.unwrap().zone, Zone::Green);
    assert_eq!(first.heal, None);
    assert_eq!(first.toast, None);
    let second = tick(&mut loop_, 5, &mut fake);
    let stats = second.stats.unwrap();
    assert_eq!((stats.rx_bytes_s, stats.tx_bytes_s), (1_000, 1_000));
}

#[test]
fn three_misses_then_one_reconnect() {
    let mut fake = Fake::default();
    fake.put("ps -eo pid=,args=", "");
    fake.put(
        "adb shell ip addr show tun0",
        "Device \"tun0\" does not exist.",
    );
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    let heals: Vec<_> = [0, 5, 10]
        .into_iter()
        .map(|t| tick(&mut loop_, t, &mut fake).heal)
        .collect();
    assert_eq!(heals, [None, None, Some(Heal::RestartClient)]);
}

#[test]
fn ticks_between_probes_run_nothing() {
    let mut fake = Fake::default();
    good(&mut fake);
    fake.put("adb shell cat /proc/net/dev", &netdev(1000));
    let mut loop_ = HealthLoop::new(0, 0, None);
    tick(&mut loop_, 0, &mut fake);
    let before = fake.ran.len();
    for t in 1..=4 {
        let result = tick(&mut loop_, t, &mut fake);
        assert!(result.tunnel);
        assert!(result.stats.is_some());
    }
    assert_eq!(fake.ran.len() - before, 0);
}

#[test]
fn battery_is_polled_every_30_s() {
    let mut fake = Fake::default();
    fake.put("ps -eo pid=,args=", "");
    fake.put("adb shell ip addr show tun0", "absent");
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    for t in (0..=60).step_by(5) {
        tick(&mut loop_, t, &mut fake);
    }
    assert_eq!(
        fake.ran
            .iter()
            .filter(|v| v.join(" ") == "adb shell dumpsys battery")
            .count(),
        3
    );
}

#[test]
fn other_modes_skip_the_tunnel_probe_but_keep_health() {
    for mode in [TetherMode::Off, TetherMode::QuestTether] {
        let mut fake = Fake::default();
        fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
        let mut loop_ = HealthLoop::new(0, 0, None);
        let result = loop_.tick(0, mode, true, &mut fake);
        assert!(!result.tunnel);
        assert_eq!(result.heal, None);
        assert_eq!(
            fake.ran,
            [vec!["adb", "shell", "dumpsys", "battery"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()]
        );
    }
}

#[test]
fn at_most_ten_full_reconnects_then_give_up() {
    let mut fake = Fake::default();
    fake.put("ps -eo pid=,args=", "");
    fake.put("adb shell ip addr show tun0", "absent");
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    let mut actions = Vec::new();
    for t in (0..=15_000).step_by(120) {
        if let Some(action) = tick(&mut loop_, t, &mut fake).heal {
            actions.push(action);
        }
    }
    assert_eq!(
        actions
            .iter()
            .filter(|&&a| a == Heal::FullReconnect)
            .count(),
        10
    );
    assert_eq!(actions.iter().filter(|&&a| a == Heal::GiveUp).count(), 1);
    assert_eq!(loop_.reconnects(), 10);
    assert!(loop_
        .tick(15_005, TetherMode::Own, true, &mut fake)
        .heal
        .is_none());
}

#[test]
fn a_new_own_session_heals_again_after_give_up() {
    let mut fake = Fake::default();
    fake.put("ps -eo pid=,args=", "");
    fake.put("adb shell ip addr show tun0", "absent");
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    let mut t = 0;
    while t <= 15_000 {
        tick(&mut loop_, t, &mut fake);
        t += 120;
    }
    assert!(tick(&mut loop_, t, &mut fake).heal.is_none());
    let before = loop_.reconnects();
    assert_eq!(before, 10);
    t += 5;
    loop_.tick(t, TetherMode::Off, true, &mut fake);
    let heals: Vec<_> = (1..=3)
        .map(|i| tick(&mut loop_, t + 5 * i, &mut fake).heal)
        .collect();
    assert_eq!(heals, [None, None, Some(Heal::RestartClient)]);
    assert_eq!(loop_.reconnects(), before);
}

#[test]
fn fast_misses_trip_the_flap_breaker() {
    let mut fake = Fake::default();
    fake.put("ps -eo pid=,args=", "");
    fake.put("adb shell ip addr show tun0", "absent");
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    let mut gave_up_at = None;
    for t in (0..=400).step_by(5) {
        if tick(&mut loop_, t, &mut fake).heal == Some(Heal::GiveUp) {
            gave_up_at = Some(t);
            break;
        }
    }
    assert!(gave_up_at.is_some_and(|t| t < 400));
}

#[test]
fn warm_toasts_once_while_playing() {
    let mut fake = Fake::default();
    good(&mut fake);
    fake.put("adb shell cat /proc/net/dev", &netdev(1_000));
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 395\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    assert_eq!(
        tick(&mut loop_, 0, &mut fake).toast,
        Some((
            EventKind::Warm,
            "Headset is warm: 39.5 C. It may slow down.".into()
        ))
    );
    assert_eq!(tick(&mut loop_, 30, &mut fake).toast, None);
}

#[test]
fn no_toast_while_not_playing() {
    let mut fake = Fake::default();
    good(&mut fake);
    fake.put("adb shell cat /proc/net/dev", &netdev(1_000));
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 395\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    assert_eq!(loop_.tick(0, TetherMode::Own, false, &mut fake).toast, None);
}

#[test]
fn soak_lines_are_written() {
    let path = temp_dir().join("flat3d-soak-1.txt");
    let _ = std::fs::remove_file(&path);
    let mut fake = Fake::default();
    good(&mut fake);
    fake.put("adb shell cat /proc/net/dev", &netdev(1000));
    let mut loop_ = HealthLoop::new(0, 0, Some(path.clone()));
    for t in [0, 5, 10] {
        tick(&mut loop_, t, &mut fake);
    }
    let summary = loop_.finish(10, 0, 0);
    let text = std::fs::read_to_string(path).unwrap();
    assert_eq!(text.matches("sample\t").count(), 3);
    assert!(text.ends_with(&summary.to_text()));
}

#[test]
fn adb_failure_is_a_miss_not_a_panic() {
    let mut fake = Fake::default();
    fake.put("ps -eo pid=,args=", "");
    fake.set("adb shell ip addr show tun0", [Err("offline".into())]);
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    for t in [0, 5, 10] {
        let _ = tick(&mut loop_, t, &mut fake);
    }
    assert_eq!(loop_.tick(15, TetherMode::Own, true, &mut fake).heal, None);
}

#[test]
fn tick_never_runs_a_heal() {
    let mut fake = Fake::default();
    fake.put("ps -eo pid=,args=", "");
    fake.put("adb shell ip addr show tun0", "absent");
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    for t in (0..=600).step_by(5) {
        tick(&mut loop_, t, &mut fake);
    }
    assert!(fake.ran.iter().all(|argv| {
        matches!(
            argv.join(" ").as_str(),
            "ps -eo pid=,args="
                | "adb shell ip addr show tun0"
                | "adb shell cat /proc/net/dev"
                | "adb shell dumpsys battery"
        )
    }));
}

#[test]
fn heal_words_and_soak_fields_are_locked() {
    assert_eq!(
        [
            Heal::None,
            Heal::RestartClient,
            Heal::RestartAdbThenClient,
            Heal::FullReconnect,
            Heal::GiveUp
        ]
        .map(Heal::word),
        [
            "none",
            "restart-client",
            "restart-adb-then-client",
            "full-reconnect",
            "give-up"
        ]
    );
    let path = temp_dir().join("flat3d-soak-fields.txt");
    let mut fake = Fake::default();
    good(&mut fake);
    fake.put("adb shell cat /proc/net/dev", &netdev(1000));
    let mut loop_ = HealthLoop::new(0, 0, Some(path.clone()));
    tick(&mut loop_, 0, &mut fake);
    let lines = std::fs::read_to_string(path).unwrap();
    let sample: Vec<_> = lines
        .lines()
        .find(|line| line.starts_with("sample\t"))
        .unwrap()
        .split('\t')
        .collect();
    let health: Vec<_> = lines
        .lines()
        .find(|line| line.starts_with("health\t"))
        .unwrap()
        .split('\t')
        .collect();
    assert_eq!(sample.len(), 7);
    assert_eq!(&sample[0..2], &["sample", "0"]);
    assert_eq!(sample[6], "none");
    assert_eq!(health.len(), 5);
    assert_eq!(&health[0..3], &["health", "0", "80"]);
    assert_eq!(health[4], "green");
}

#[test]
fn a_clock_step_backwards_does_not_stall_the_probes() {
    let mut fake = Fake::default();
    fake.put("ps -eo pid=,args=", "");
    fake.put("adb shell ip addr show tun0", "absent");
    fake.put("adb shell dumpsys battery", "level: 80\ntemperature: 300\n");
    let mut loop_ = HealthLoop::new(0, 0, None);
    tick(&mut loop_, 10_000, &mut fake);
    let before = fake.ran.len();
    tick(&mut loop_, 100, &mut fake);
    assert_eq!(fake.ran.len() - before, 3);
}
