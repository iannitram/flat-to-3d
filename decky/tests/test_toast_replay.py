"""Locked test for issue #133 (toast replay): decky/src/toastpoll.ts. The poller must learn the
engine's current last_seq on its first poll without showing anything, then show only newer toasts.
Runs the real TypeScript file under node (type stripping); no dependency, no network, no device."""
import json
import os
import shutil
import subprocess
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
MODULE = os.path.join(HERE, "..", "src", "toastpoll.ts")

DRIVER = r"""
import { pathToFileURL } from "node:url";
const { createToastPoller } = await import(pathToFileURL(process.argv[2]).href);

// Same rules as crates/engine/src/face/toasts.rs toasts_since.
function engineOver(events) {
  const calls = [];
  const fetchBatch = async (last) => {
    calls.push(last);
    if (events.list.length === 0) return { ok: true, command: "toasts", data: { toasts: [], last_seq: last } };
    const max = Math.max(...events.list.map((e) => e.seq));
    const restarted = max < last;
    const toasts = events.list.filter((e) => restarted || e.seq > last).sort((a, b) => a.seq - b.seq);
    return { ok: true, command: "toasts", data: { toasts, last_seq: max > last || restarted ? max : last } };
  };
  return { calls, fetchBatch };
}
const ev = (seq, text) => ({ seq, kind: "ready", text });
const out = {};

{ // old events are primed away, newer ones show once
  const events = { list: [ev(1, "a"), ev(2, "b"), ev(3, "c")] };
  const { calls, fetchBatch } = engineOver(events);
  const shown = [];
  const p = createToastPoller(fetchBatch, (t) => shown.push(t));
  await p.tick();
  const afterPrime = shown.slice();
  await p.tick();
  const afterQuiet = shown.slice();
  events.list.push(ev(4, "d"));
  await p.tick();
  const afterNew = shown.slice();
  await p.tick();
  out.prime = { afterPrime, afterQuiet, afterNew, final: shown, calls };
}
{ // engine restart: seq goes back down, everything in the ring shows (restarted flag)
  const events = { list: [ev(1, "a"), ev(2, "b"), ev(3, "c"), ev(4, "d")] };
  const { calls, fetchBatch } = engineOver(events);
  const shown = [];
  const p = createToastPoller(fetchBatch, (t) => shown.push(t));
  await p.tick();
  events.list = [ev(1, "x")];
  await p.tick();
  await p.tick();
  out.restart = { shown, calls };
}
{ // empty ring at start: the first toast after that still shows
  const events = { list: [] };
  const { calls, fetchBatch } = engineOver(events);
  const shown = [];
  const p = createToastPoller(fetchBatch, (t) => shown.push(t));
  await p.tick();
  events.list.push(ev(1, "first"));
  await p.tick();
  out.empty = { shown, calls };
}
{ // a failed first poll does not prime; the next good one primes without showing
  const events = { list: [ev(1, "a"), ev(2, "b")] };
  const real = engineOver(events);
  let fail = true;
  const calls = [];
  const fetchBatch = async (last) => {
    calls.push(last);
    if (fail) { fail = false; return { ok: false, command: "toasts", error: { code: "io", message: "x" } }; }
    return real.fetchBatch(last);
  };
  const shown = [];
  const p = createToastPoller(fetchBatch, (t) => shown.push(t));
  await p.tick();
  await p.tick();
  events.list.push(ev(3, "c"));
  await p.tick();
  out.failed = { shown, calls };
}
{ // overlapping ticks make one call; stop() while a call is out shows nothing
  const events = { list: [ev(1, "a")] };
  const real = engineOver(events);
  let release = null;
  let count = 0;
  const shown = [];
  const p = createToastPoller(async (last) => {
    count++;
    if (release === null || count < 2) return real.fetchBatch(last);
    await new Promise((r) => (release = r));
    return real.fetchBatch(last);
  }, (t) => shown.push(t));
  release = true;
  await p.tick();
  events.list.push(ev(2, "b"));
  const t1 = p.tick();
  const t2 = p.tick();
  p.stop();
  await new Promise((r) => setTimeout(r, 5));
  release();
  await Promise.all([t1, t2]);
  out.stop = { count, shown };
}
console.log(JSON.stringify(out));
"""


def _node_ok():
    if not shutil.which("node"):
        return False
    try:
        v = subprocess.run(["node", "-p", "process.versions.node"], capture_output=True, text=True, timeout=30).stdout
        return tuple(int(x) for x in v.strip().split(".")[:2]) >= (22, 18)
    except (OSError, ValueError, subprocess.SubprocessError):
        return False


@unittest.skipUnless(_node_ok(), "node 22.18 or newer is not installed")
class ToastReplayTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        tmp = tempfile.TemporaryDirectory()
        cls.addClassCleanup(tmp.cleanup)
        driver = os.path.join(tmp.name, "driver.mjs")
        with open(driver, "w") as f:
            f.write(DRIVER)
        run = subprocess.run(
            ["node", "--no-warnings", "--experimental-strip-types", driver, os.path.abspath(MODULE)],
            capture_output=True, text=True, timeout=60,
        )
        if run.returncode != 0:
            raise AssertionError("node driver failed: " + run.stderr)
        cls.out = json.loads(run.stdout)

    def test_first_poll_shows_nothing_old(self):
        self.assertEqual(self.out["prime"]["afterPrime"], [])

    def test_first_poll_asks_from_zero_then_from_the_primed_seq(self):
        self.assertEqual(self.out["prime"]["calls"], [0, 3, 3, 4])

    def test_quiet_poll_after_prime_shows_nothing(self):
        self.assertEqual(self.out["prime"]["afterQuiet"], [])

    def test_only_the_newer_toast_shows_and_only_once(self):
        self.assertEqual(self.out["prime"]["afterNew"], ["d"])
        self.assertEqual(self.out["prime"]["final"], ["d"])

    def test_engine_restart_shows_the_ring_again(self):
        self.assertEqual(self.out["restart"]["shown"], ["x"])
        self.assertEqual(self.out["restart"]["calls"], [0, 4, 1])

    def test_empty_ring_at_start_does_not_swallow_the_first_toast(self):
        self.assertEqual(self.out["empty"]["shown"], ["first"])
        self.assertEqual(self.out["empty"]["calls"], [0, 0])

    def test_failed_first_poll_does_not_prime(self):
        self.assertEqual(self.out["failed"]["shown"], ["c"])
        self.assertEqual(self.out["failed"]["calls"], [0, 0, 2])

    def test_overlap_makes_one_call_and_stop_shows_nothing(self):
        self.assertEqual(self.out["stop"], {"count": 2, "shown": []})


if __name__ == "__main__":
    unittest.main()
