//! Locked tests for E7d: the soak summary.

use flat3d_inject::stats::{mem_used_kb, soak_path, Soak};

#[test]
fn the_text_is_exactly_this() {
    let mut s = Soak::new(100, 1000);
    for v in [900, 100, 200] {
        s.sample(v);
    }
    let sum = s.finish(700, 1200, 1, 0);
    assert_eq!(
        sum.to_text(),
        "flat3d-soak 1\nstart\t100\nend\t700\npeak_bytes_s\t900\navg_bytes_s\t400\nreconnects\t1\nmem_start_kb\t1000\nmem_end_kb\t1200\norphans\t0\n"
    );
}

#[test]
fn peak_and_average() {
    let mut s = Soak::new(0, 0);
    s.sample(10);
    s.sample(5);
    let sum = s.finish(1, 0, 0, 0);
    assert_eq!((sum.peak_bytes_s, sum.avg_bytes_s), (10, 7));
}

#[test]
fn an_empty_session_has_zero_figures() {
    let sum = Soak::new(5, 7).finish(9, 8, 3, 2);
    assert_eq!((sum.peak_bytes_s, sum.avg_bytes_s), (0, 0));
    assert_eq!(
        (sum.start, sum.end, sum.reconnects, sum.orphans),
        (5, 9, 3, 2)
    );
    assert_eq!((sum.mem_start_kb, sum.mem_end_kb), (7, 8));
}

#[test]
fn memory_used_is_total_minus_available() {
    let m = "MemTotal:       65000000 kB\nMemFree:         1000 kB\nMemAvailable:   40000000 kB\n";
    assert_eq!(mem_used_kb(m), Some(25000000));
    assert_eq!(mem_used_kb("MemTotal: 5 kB\n"), None);
    assert_eq!(mem_used_kb(""), None);
}

#[test]
fn the_file_path() {
    assert_eq!(
        soak_path("/home/deck", 1762000000),
        "/home/deck/.local/state/flat3d/soak-1762000000.txt"
    );
}
