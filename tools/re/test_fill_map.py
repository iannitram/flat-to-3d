import contextlib
import hashlib
import io
import pathlib
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest

import fill_map

ROOT = pathlib.Path(__file__).resolve().parents[2]
MAP = ROOT / "games/spiderman2/map.toml"
TARGETS = ROOT / "tools/xmatch/targets.py"
DOC = ROOT / "docs/spiderman2/address-map.md"

# Three rows; source 0x1646e10 appears twice and the second (higher similarity) must win.
VT_CSV = (
    "src_rva,dst_rva,status,similarity,confidence,correlator\n"
    "0x1646e10,0x2c00010,AVAILABLE,0.5,1.0,Exact Function Bytes Match\n"
    "0x164703f,0x2c00230,ACCEPTED,1.0,10.0,Exact Function Instructions Match\n"
    "0x1646e10,0x2c76310,ACCEPTED,0.9,2.0,Combined Function and Data Reference Match\n"
)

DOC_TEXT = (
    "# Title\n\nIntro text, kept as it is.\n\n"
    "<!-- BEGIN GENERATED -->\nold block\n<!-- END GENERATED -->\n\n"
    "## Next leg\n\nTail text, kept as it is.\n"
)


def item(name, rva, kind="call", prio=1, **kw):
    d = {"name": name, "rem_rva": rva, "kind": kind, "prio": prio, "sm2": "", "sig": "", "conf": "pending",
         "how": "", "status": "pending"}
    d.update(kw)
    return d


def herovr(hid, what, check="pending", item="", evidence=""):
    return {"id": hid, "what": what, "check": check, "item": item, "evidence": evidence}


TARGETS_PY = (
    "T = [\n"
    "    ('stock camera fn (calls submit)', 0x1646e10, 'call', 1, ''),\n"
    "    ('camera submit call site', 0x164703f, 'site', 1, 'e86c2a2500'),\n"
    "    ('view update (exposure copy)', 0x189bd30, 'hook', 2, '48895c240848897c2410'),\n"
    "]\n"
)


def small_map():
    return {
        "meta": {"herovr_rev": "82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859"},
        "item": [item("stock camera fn (calls submit)", "0x1646e10"),
                 item("camera submit call site", "0x164703f", "site"),
                 item("view update (exposure copy)", "0x189bd30", "hook", 2)],
        "herovr": [herovr("H1", "Runtime queue discovery", "runtime"),
                   herovr("H5", "Pose setter (3 arguments)"),
                   herovr("H11", "Steam ticket guard", "off-path")],
    }


class Tmp(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)
        self.targets = self.dir / "targets.py"
        self.targets.write_text(TARGETS_PY, encoding="utf-8")
        self.map = self.dir / "map.toml"
        self.map.write_text(fill_map.emit_toml(small_map()), encoding="utf-8", newline="")
        self.doc = self.dir / "address-map.md"
        self.doc.write_text(DOC_TEXT, encoding="utf-8", newline="")
        self.csv = self.dir / "vt-matches.csv"
        self.csv.write_text(VT_CSV, encoding="utf-8", newline="")

    def tearDown(self):
        self.tmp.cleanup()

    def run_main(self, *extra, vt=False):
        argv = ["--map", str(self.map), "--targets", str(self.targets), "--doc", str(self.doc)]
        if vt:
            argv += ["--vt", str(self.csv)]
        err = io.StringIO()
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(io.StringIO()):
            rc = fill_map.main(argv + list(extra))
        return rc, err.getvalue()


class VtAndRender(Tmp):
    def test_load_vt_keeps_the_better_duplicate(self):
        vt = fill_map.load_vt(self.csv)
        self.assertEqual(vt, {
            0x1646e10: (0x2c76310, "ACCEPTED", 0.9, 2.0, "Combined Function and Data Reference Match"),
            0x164703f: (0x2c00230, "ACCEPTED", 1.0, 10.0, "Exact Function Instructions Match"),
        })

    def test_equal_scores_keep_the_first_seen(self):
        self.csv.write_text("src_rva,dst_rva,status,similarity,confidence,correlator\n"
                            "0x10,0x20,AVAILABLE,1.0,1.0,first\n0x10,0x30,AVAILABLE,1.0,1.0,second\n"
                            "0x10,0x40,AVAILABLE,1.0,0.5,third\n", encoding="utf-8")
        self.assertEqual(fill_map.load_vt(self.csv), {0x10: (0x20, "AVAILABLE", 1.0, 1.0, "first")})

    def test_vt_rows_render_the_right_table(self):
        targets = fill_map.load_targets(self.targets)
        vt = fill_map.load_vt(self.csv)
        items = fill_map.join(targets, small_map())
        self.assertEqual(fill_map.render_table(1, items, vt), (
            "| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found |\n"
            "|---|---|---|---|---|---|---|---|\n"
            "| stock camera fn (calls submit) | call | `0x1646e10` | - | `0x2c76310` | - | vt 0.90 | Combined Function and Data Reference Match |\n"
            "| camera submit call site | site | `0x164703f` | `e86c2a2500` | `0x2c00230` | - | vt 1.00 | Exact Function Instructions Match |\n"))
        self.assertEqual(fill_map.render_table(2, items, {}), (
            fill_map.TABLE_HEAD +
            "| view update (exposure copy) | hook | `0x189bd30` | `48895c240848897c2410` | pending | - | pending | - |\n"))

    def test_splice_is_idempotent_and_keeps_text_outside_the_markers(self):
        block = fill_map.render_block(fill_map.join(fill_map.load_targets(self.targets), small_map()), {}, small_map()["herovr"])
        once = fill_map.splice(DOC_TEXT, block)
        self.assertEqual(fill_map.splice(once, block), once)
        head, tail = DOC_TEXT.split("old block\n")
        self.assertTrue(once.startswith(head))
        self.assertTrue(once.endswith(tail))
        self.assertEqual(once, head + block + tail)
        self.assertEqual(once.count(fill_map.BEGIN), 1)
        self.assertEqual(once.count(fill_map.END), 1)
        self.assertTrue(block.startswith("### Priority 1. "))

    def test_herovr_table_has_one_row_per_entry(self):
        block = fill_map.render_block([], {}, small_map()["herovr"])
        part = block.split("### Checked against HEROVR's list\n\n", 1)[1]
        self.assertEqual(part, (
            "| HEROVR item | Our row | Check | Evidence |\n|---|---|---|---|\n"
            "| H1 Runtime queue discovery | - | runtime | - |\n"
            "| H5 Pose setter (3 arguments) | - | pending | - |\n"
            "| H11 Steam ticket guard | - | off-path | - |\n\n"))
        found = herovr("H5", "Pose setter (3 arguments)", "found", "pose setter", "fill_map")
        self.assertIn("| H5 Pose setter (3 arguments) | pose setter | found | fill_map |\n",
                      fill_map.render_herovr([found]))

    def test_no_herovr_rows_render_exactly_what_mkmap_prints(self):
        r = subprocess.run([sys.executable, str(ROOT / "tools/xmatch/mkmap.py")], capture_output=True, text=True, check=True)
        targets = fill_map.load_targets(TARGETS)
        d = fill_map.load_map(MAP)
        pending = {**d, "item": [{**i, "sm2": "", "sig": "", "conf": "pending", "how": ""} for i in d["item"]]}
        self.assertEqual(fill_map.render_block(fill_map.join(targets, pending), {}, []), r.stdout[1:] + "\n")


class Cli(Tmp):
    def test_missing_markers_refuse_with_exit_2(self):
        self.doc.write_text("# Title\n\nno markers\n", encoding="utf-8")
        rc, err = self.run_main()
        self.assertEqual(rc, 2)
        self.assertIn("needs exactly one <!-- BEGIN GENERATED --> / <!-- END GENERATED --> pair; fix: restore the marker lines (WP-25b placement)", err)
        self.assertEqual(self.doc.read_text(encoding="utf-8"), "# Title\n\nno markers\n")

    def test_duplicated_markers_refuse_with_exit_2(self):
        for text in (DOC_TEXT + "<!-- BEGIN GENERATED -->\n", DOC_TEXT + "<!-- END GENERATED -->\n",
                     "<!-- END GENERATED -->\nx\n<!-- BEGIN GENERATED -->\n"):
            self.doc.write_text(text, encoding="utf-8", newline="")
            rc, err = self.run_main()
            self.assertEqual(rc, 2, text)
            self.assertIn("needs exactly one", err)
            self.assertEqual(self.doc.read_text(encoding="utf-8"), text)

    def test_check_without_vt_passes_in_sync_and_fails_on_drift(self):
        rc, _ = self.run_main("--check")
        self.assertEqual(rc, 1)
        self.assertEqual(self.doc.read_text(encoding="utf-8"), DOC_TEXT)
        map_before = self.map.read_bytes()
        self.assertEqual(self.run_main()[0], 0)
        self.assertEqual(self.map.read_bytes(), map_before)
        rc, err = self.run_main("--check")
        self.assertEqual((rc, err), (0, ""))
        self.doc.write_text(self.doc.read_text(encoding="utf-8").replace("| pending |", "| `0x1` |", 1), encoding="utf-8", newline="")
        drifted = self.doc.read_bytes()
        rc, err = self.run_main("--check")
        self.assertEqual(rc, 1)
        self.assertIn(fill_map.FIX, err)
        self.assertEqual(self.doc.read_bytes(), drifted)

    def test_vt_writes_the_map_with_a_receipt_then_checks_clean(self):
        self.assertEqual(self.run_main(vt=True)[0], 0)
        d = tomllib.loads(self.map.read_text(encoding="utf-8"))
        receipt = "vt-matches.csv sha256:" + hashlib.sha256(VT_CSV.encode()).hexdigest()[:12]
        first, second, third = d["item"]
        self.assertEqual(first, {**item("stock camera fn (calls submit)", "0x1646e10"),
                                 "sm2": "0x2c76310", "conf": "vt 0.90", "how": "Combined Function and Data Reference Match",
                                 "status": "found", "similarity": 0.9, "confidence": 2.0,
                                 "correlator": "Combined Function and Data Reference Match", "verified_by": receipt})
        self.assertEqual(second["sm2"], "0x2c00230")
        self.assertEqual(third["status"], "pending")
        self.assertIn("| stock camera fn (calls submit) | call | `0x1646e10` | - | `0x2c76310` |", self.doc.read_text(encoding="utf-8"))
        self.assertEqual(self.run_main("--check", vt=True)[0], 0)
        self.assertEqual(self.run_main("--check")[0], 0)
        self.map.write_text(fill_map.emit_toml(small_map()), encoding="utf-8", newline="")
        before = self.map.read_bytes()
        rc, err = self.run_main("--check", vt=True)
        self.assertEqual(rc, 1)
        self.assertIn("--vt " + str(self.csv), err)
        self.assertEqual(self.map.read_bytes(), before)

    def test_no_vt_never_writes_the_map(self):
        self.map.write_text(self.map.read_text(encoding="utf-8") + "\n# hand comment\n", encoding="utf-8", newline="")
        before = self.map.read_bytes()
        self.assertEqual(self.run_main()[0], 0)
        self.assertEqual(self.map.read_bytes(), before)


class Lints(unittest.TestCase):
    T = [("a", 0x10, "call", 1, ""), ("b", 0x20, "hook", 2, "")]

    def base(self):
        return {"meta": {}, "item": [item("a", "0x10"), item("b", "0x20", "hook", 2)],
                "herovr": [herovr("H1", "Runtime queue discovery", "runtime")]}

    def assertLint(self, d, *needles):
        errs = "\n".join(fill_map.lint(d, self.T))
        for n in needles:
            self.assertIn(n, errs)
        self.assertIn("fix:", errs)

    def test_clean_map_has_no_lint(self):
        self.assertEqual(fill_map.lint(self.base(), self.T), [])

    def test_targets_row_without_item(self):
        d = self.base()
        d["item"].pop()
        self.assertLint(d, "targets.py row 'b' (0x20) has no [[item]]")

    def test_item_without_targets_row(self):
        d = self.base()
        d["item"].append(item("c", "0x30"))
        self.assertLint(d, "[[item]] 'c' (0x30) has no targets.py row")

    def test_evidence_herovr(self):
        d = self.base()
        d["herovr"][0]["evidence"] = "herovr"
        self.assertLint(d, "[[herovr]] H1: evidence is \"herovr\"")

    def test_hex_in_herovr_text(self):
        d = self.base()
        d["herovr"][0]["what"] = "Pose setter at 2c76310"
        self.assertLint(d, "[[herovr]] H1: what holds hex '2c76310'")

    def test_found_without_a_computed_match(self):
        d = self.base()
        d["item"][0].update(status="found", sm2="0x99", verified_by="x.csv sha256:000000000000")
        self.assertLint(d, "[[item]] 'a': found without similarity, confidence, correlator")

    def test_found_without_verified_by(self):
        d = self.base()
        d["item"][0].update(status="found", sm2="0x99", similarity=1.0, confidence=1.0, correlator="c")
        errs = fill_map.lint(d, self.T)
        self.assertEqual(len(errs), 1)
        self.assertIn("[[item]] 'a': found without verified_by", errs[0])

    def test_lint_exits_1_from_the_cli(self):
        with tempfile.TemporaryDirectory() as t:
            t = pathlib.Path(t)
            (t / "targets.py").write_text(TARGETS_PY, encoding="utf-8")
            d = small_map()
            d["herovr"][0]["evidence"] = "herovr"
            (t / "map.toml").write_text(fill_map.emit_toml(d), encoding="utf-8")
            (t / "doc.md").write_text(DOC_TEXT, encoding="utf-8")
            err = io.StringIO()
            with contextlib.redirect_stderr(err):
                rc = fill_map.main(["--map", str(t / "map.toml"), "--targets", str(t / "targets.py"), "--doc", str(t / "doc.md")])
            self.assertEqual(rc, 1)
            self.assertIn("evidence is \"herovr\"", err.getvalue())
            self.assertEqual((t / "doc.md").read_text(encoding="utf-8"), DOC_TEXT)


class Toml(unittest.TestCase):
    def test_emit_round_trips_through_tomllib(self):
        d = {"top": "x", "meta": {"herovr_rev": "abc", "n": 3, "on": True},
             "item": [{"name": 'quote " back \\ tab \t nl \n', "rem_rva": "0x10", "prio": 1, "similarity": 0.95,
                       "confidence": 10.0, "whole": 1.0, "status": "found"}],
             "herovr": [{"id": "H1", "what": "Pose setter (3 arguments)", "item": ""}]}
        text = fill_map.emit_toml(d)
        self.assertEqual(tomllib.loads(text), d)
        self.assertEqual(fill_map.emit_toml(tomllib.loads(text)), text)
        self.assertTrue(text.startswith(fill_map.HEADER))

    def test_repo_map_is_its_own_emit(self):
        text = MAP.read_text(encoding="utf-8")
        d = tomllib.loads(text)
        self.assertEqual(fill_map.emit_toml(d), text)
        self.assertEqual(d["meta"]["herovr_rev"], "82f3ffe4e4dc65d9f0df3b29918af2ccb7fc9859")
        targets = fill_map.load_targets(TARGETS)
        self.assertEqual(len(d["item"]), 85)
        self.assertEqual([i["rem_rva"] for i in d["item"]], [hex(t[1]) for t in targets])
        self.assertEqual({i["status"] for i in d["item"]}, {"pending"})
        self.assertEqual([h["id"] for h in d["herovr"]], ["H%d" % n for n in range(1, 12)])
        checks = {h["id"]: h["check"] for h in d["herovr"]}
        self.assertEqual(checks, {"H1": "runtime", "H6": "no-row", "H11": "off-path",
                                  **{"H%d" % n: "pending" for n in (2, 3, 4, 5, 7, 8, 9, 10)}})
        self.assertEqual(fill_map.lint(d, targets), [])

    def test_repo_doc_block_is_in_sync(self):
        with tempfile.TemporaryDirectory() as t:
            copy = pathlib.Path(t) / "address-map.md"
            shutil.copyfile(DOC, copy)
            err = io.StringIO()
            with contextlib.redirect_stderr(err):
                rc = fill_map.main(["--map", str(MAP), "--targets", str(TARGETS), "--doc", str(copy), "--check"])
            self.assertEqual(rc, 0, err.getvalue())


if __name__ == "__main__":
    unittest.main()
