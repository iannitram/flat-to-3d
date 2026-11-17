"""Locked test: decky/src/types.ts carries the engine's closed word lists (crates/engine/src/api.rs).
LinkEvent.kind is the EventKind union and LinkView.adb_source the AdbSource union, both in the
engine's serde order; ToastBatch has only the two keys the engine sends. Reads both files as text;
no node, no device."""
import os
import re
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
TYPES_TS = os.path.join(HERE, "..", "src", "types.ts")
API_RS = os.path.join(HERE, "..", "..", "crates", "engine", "src", "api.rs")

EVENT_KINDS = ("ready", "late-plug", "grace-ended", "allow-debugging", "warm", "hot")
ADB_SOURCES = ("running", "bundled", "missing")


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


def ts_union(text, name):
    """The quoted words of `export type <name> = "a" | "b";`, in order; None when it is missing."""
    m = re.search(r"export type " + name + r"\s*=\s*([^;]+);", text)
    if not m:
        return None
    return tuple(re.findall(r'"([^"]*)"', m.group(1)))


def ts_interface(text, name):
    """The field lines of `export interface <name> { ... }`, one string per field, spaces squeezed."""
    m = re.search(r"export interface " + name + r"\s*\{([^}]*)\}", text)
    if not m:
        return None
    fields = re.split(r"[;\n]", m.group(1))
    return tuple(" ".join(f.split()) for f in fields if f.strip())


def rust_kebab_variants(text, name):
    """The variants of `pub enum <name>`, kebab-cased the way serde(rename_all = "kebab-case") does."""
    m = re.search(r"pub enum " + name + r"\s*\{([^}]*)\}", text)
    assert m, name
    words = re.findall(r"^\s*([A-Z][A-Za-z0-9]*),", m.group(1), re.M)
    return tuple(re.sub(r"(?<!^)([A-Z])", r"-\1", w).lower() for w in words)


class TypesMatchEngineTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ts = read(TYPES_TS)
        cls.rs = read(API_RS)

    def test_engine_event_kinds_are_these(self):
        # Catches: this test's own list going stale against api.rs (guard; green on main).
        self.assertEqual(rust_kebab_variants(self.rs, "EventKind"), EVENT_KINDS)

    def test_engine_adb_sources_are_these(self):
        # Catches: this test's own list going stale against api.rs (guard; green on main).
        self.assertEqual(rust_kebab_variants(self.rs, "AdbSource"), ADB_SOURCES)

    def test_event_kind_union_in_engine_order(self):
        # Catches: no EventKind type, a word missing or misspelled, or a different order.
        self.assertEqual(ts_union(self.ts, "EventKind"), EVENT_KINDS)

    def test_adb_source_union_in_engine_order(self):
        # Catches: no AdbSource type, a word missing or misspelled, or a different order.
        self.assertEqual(ts_union(self.ts, "AdbSource"), ADB_SOURCES)

    def test_link_event_kind_uses_the_union(self):
        # Catches: LinkEvent.kind left as string.
        self.assertEqual(ts_interface(self.ts, "LinkEvent"), ("seq: number", "kind: EventKind", "text: string"))

    def test_link_view_adb_source_uses_the_union(self):
        # Catches: LinkView.adb_source left as string; any other LinkView field changed.
        self.assertEqual(
            ts_interface(self.ts, "LinkView"),
            (
                "state: LinkState",
                "label: string",
                "hint: string",
                "start: boolean",
                "stop: boolean",
                "health: HealthView",
                "auto_3d: boolean",
                "adb_source: AdbSource",
                "quest_tether: string | null",
            ),
        )

    def test_toast_batch_has_only_the_engine_keys(self):
        # Catches: the title?: string field the engine never sends left in place.
        self.assertEqual(ts_interface(self.ts, "ToastBatch"), ("toasts: LinkEvent[]", "last_seq: number"))

    def test_the_other_types_are_unchanged(self):
        # Catches: an edit outside the three types above (LinkState, Zone, GameRow, TOAST_TITLE).
        self.assertEqual(ts_union(self.ts, "LinkState"), ("no-headset", "unauthorised", "starting", "ready", "stopped"))
        self.assertEqual(ts_union(self.ts, "Zone"), ("green", "yellow", "red", "unknown"))
        self.assertIn("flat_reason: string | null", ts_interface(self.ts, "GameRow"))
        self.assertIn('export const TOAST_TITLE = "Flat to 3D";', self.ts)


if __name__ == "__main__":
    unittest.main()
