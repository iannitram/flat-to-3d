//! Locked tests for E6c: relay and client command lists. Pure: nothing is run.

use flat3d_inject::tether::relay::*;

fn c(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn the_constants() {
    assert_eq!(RELAY_PORT, 31416);
    assert_eq!(CLIENT_PACKAGE, "com.genymobile.gnirehtet");
}

#[test]
fn the_tool_paths() {
    assert_eq!(
        relay_path("/home/deck"),
        "/home/deck/.local/share/flat3d/bin/gnirehtet"
    );
    assert_eq!(
        apk_path("/home/deck"),
        "/home/deck/.local/share/flat3d/bin/gnirehtet.apk"
    );
}

#[test]
fn the_relay_runs_for_one_serial() {
    assert_eq!(
        relay_command("/r/gnirehtet", "S1"),
        c(&["/r/gnirehtet", "run", "S1"])
    );
}

#[test]
fn the_client_starts_with_the_reverse_then_the_intent() {
    let want = vec![
        c(&["adb", "reverse", "localabstract:gnirehtet", "tcp:31416"]),
        c(&[
            "adb",
            "shell",
            "am",
            "start",
            "-n",
            "com.genymobile.gnirehtet/.GnirehtetActivity",
            "-a",
            "com.genymobile.gnirehtet.START",
        ]),
    ];
    assert_eq!(client_start_commands(), want);
}

#[test]
fn stopping_the_client() {
    assert_eq!(
        client_stop_command(),
        c(&[
            "adb",
            "shell",
            "am",
            "start",
            "-n",
            "com.genymobile.gnirehtet/.GnirehtetActivity",
            "-a",
            "com.genymobile.gnirehtet.STOP",
        ])
    );
    assert_eq!(
        force_stop_command(),
        c(&[
            "adb",
            "shell",
            "am",
            "force-stop",
            "com.genymobile.gnirehtet"
        ])
    );
}

#[test]
fn only_the_gnirehtet_reverse_is_removed() {
    let cmd = remove_reverse_command();
    assert_eq!(
        cmd,
        c(&["adb", "reverse", "--remove", "localabstract:gnirehtet"])
    );
    assert!(!cmd.iter().any(|w| w.contains("remove-all")));
}
