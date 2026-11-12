#!/usr/bin/env python3
# Flat to 3D (GPL-3.0). Offline: reads map.toml, targets.py and a Version Tracking CSV; writes no game file.
"""fill_map.py: merge Version Tracking matches into the Spider-Man 2 address map and render its doc block.

usage: fill_map.py --map games/spiderman2/map.toml [--vt out.csv] --targets tools/xmatch/targets.py
                   --doc docs/spiderman2/address-map.md [--check]

games/spiderman2/map.toml holds the addresses (one [[item]] per targets.py row, one [[herovr]]
per HEROVR checklist item). The doc block between <!-- BEGIN GENERATED --> and
<!-- END GENERATED --> is generated from it.

- With --vt: each item whose Remastered RVA has a VT match becomes `found`; map.toml is written
  back, then the doc block is spliced.
- Without --vt: map.toml is never written; only the doc block is spliced.
- --check writes nothing and exits 1 on drift (the doc block differs from the render, or, with
  --vt, map.toml differs from the merged map).

Exit: 0 ok, 1 lint or drift, 2 usage, unreadable input or bad markers.
"""
import argparse
import csv
import hashlib
import importlib.util
import math
import os
import re
import sys
import tomllib

BEGIN = "<!-- BEGIN GENERATED -->"
END = "<!-- END GENERATED -->"
FIX = ("fix: python3 tools/re/fill_map.py --map games/spiderman2/map.toml "
       "--targets tools/xmatch/targets.py --doc docs/spiderman2/address-map.md")
HEADER = ("# Spider-Man 2 address map: the one owner of the SM2 addresses (WP-30).\n"
          "# tools/re/fill_map.py writes this file from a Version Tracking CSV and renders\n"
          "# docs/spiderman2/address-map.md (generated block) from it.\n")

# Headings, header and separator as tools/xmatch/mkmap.py prints them.
PN = {1: 'Steps 1-3: camera, frame entry, view setup', 2: 'Steps 4-5: eye views, render jobs, buffers, memory, pause gate',
      3: 'Step 6-7 and removable: HUD, game-time checks, shadows, diagnostics, gate extras'}
TABLE_HEAD = ('| Item | Kind | Remastered RVA | Spidy checks these bytes there | SM2 address | SM2 pattern | Confidence | How found |\n'
              '|---|---|---|---|---|---|---|---|\n')
HEROVR_HEADING = "### Checked against HEROVR's list"
HEROVR_HEAD = '| HEROVR item | Our row | Check | Evidence |\n|---|---|---|---|\n'

STATUS = ("pending", "found")
CHECKS = ("found", "not-found", "pending", "runtime", "off-path", "no-row")
HEX_RUN = re.compile(r"[0-9a-fA-F]{5,}")
BARE_KEY = re.compile(r"[A-Za-z0-9_-]+")


class MarkerError(ValueError):
    """The doc does not hold exactly one ordered BEGIN/END GENERATED pair."""


# ---------- inputs ----------

def load_targets(path):
    """targets.py's list T of (name, rva, kind, prio, hex), loaded by file path (no sys.path edits)."""
    spec = importlib.util.spec_from_file_location("flat3d_targets", path)
    if spec is None or spec.loader is None:
        raise OSError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return list(mod.T)


def load_map(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def load_vt(path) -> dict[int, tuple[int, str, float, float, str]]:
    """VT CSV (src_rva,dst_rva,status,similarity,confidence,correlator) -> {src_rva: (dst_rva, status,
    similarity, confidence, correlator)}. Per source RVA it keeps the row with the highest similarity,
    then the highest confidence, then the first seen."""
    out = {}
    with open(path, newline="", encoding="utf-8") as f:
        for n, row in enumerate(csv.DictReader(f), 2):
            try:
                src = int(row["src_rva"], 16)
                rec = (int(row["dst_rva"], 16), row["status"].strip(), float(row["similarity"]),
                       float(row["confidence"]), row["correlator"].strip())
            except (KeyError, TypeError, ValueError) as e:
                raise ValueError(f"{path}:{n}: bad VT row ({e}); fix: write the header "
                                 "src_rva,dst_rva,status,similarity,confidence,correlator and hex RVAs") from None
            old = out.get(src)
            if old is None or (rec[2], rec[3]) > (old[2], old[3]):
                out[src] = rec
    return out


def receipt(path):
    """`verified_by` for a CSV: '<basename> sha256:<first 12 hex of its sha256>'."""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return f"{os.path.basename(path)} sha256:{h.hexdigest()[:12]}"


# ---------- merge ----------

def found_fields(row):
    """The item fields a VT row sets (sig is left alone)."""
    dst, _, sim, conf, corr = row
    return {"sm2": hex(dst), "conf": f"vt {sim:.2f}", "how": corr, "status": "found",
            "similarity": sim, "confidence": conf, "correlator": corr}


def merge(d, vt, verified_by):
    """A copy of map `d` with every item that has a VT row (by Remastered RVA) set to found."""
    out = dict(d)
    items = []
    for it in d.get("item", []):
        it = dict(it)
        row = vt.get(rva_of(it))
        if row is not None:
            it.update(found_fields(row))
            it["verified_by"] = verified_by
        items.append(it)
    out["item"] = items
    return out


def rva_of(it):
    try:
        return int(str(it.get("rem_rva", "")), 16)
    except ValueError:
        return None


# ---------- lints ----------

def lint(d, targets):
    """One line per problem, each naming the row and the fix."""
    errs = []
    items = d.get("item", [])
    by_rva = {}
    for it in items:
        r = rva_of(it)
        name = it.get("name", "?")
        if r is None:
            errs.append(f"[[item]] {name!r}: rem_rva {it.get('rem_rva')!r} is not hex; fix: write rem_rva as a hex string such as \"0x1646e10\"")
            continue
        if r in by_rva:
            errs.append(f"[[item]] {name!r}: rem_rva {hex(r)} is also {by_rva[r].get('name')!r}; fix: keep one [[item]] per Remastered RVA")
        by_rva[r] = it
    rvas = set()
    for name, rva, *_ in targets:
        rvas.add(rva)
        if rva not in by_rva:
            errs.append(f"targets.py row {name!r} ({hex(rva)}) has no [[item]]; fix: add an [[item]] with rem_rva = \"{hex(rva)}\" to map.toml")
    for r, it in by_rva.items():
        if r not in rvas:
            errs.append(f"[[item]] {it.get('name')!r} ({hex(r)}) has no targets.py row; fix: remove the [[item]] or add the row to tools/xmatch/targets.py")
    for it in items:
        name = it.get("name", "?")
        st = it.get("status")
        if st not in STATUS:
            errs.append(f"[[item]] {name!r}: status {st!r}; fix: use one of {', '.join(STATUS)}")
        if st == "found":
            miss = [k for k in ("similarity", "confidence", "correlator") if k not in it]
            if miss:
                errs.append(f"[[item]] {name!r}: found without {', '.join(miss)} (not a computed match); fix: run fill_map.py --vt <csv> from a VT run, or set status = \"pending\"")
            if not it.get("verified_by"):
                errs.append(f"[[item]] {name!r}: found without verified_by; fix: run fill_map.py --vt <csv> so the match carries its CSV receipt, or set status = \"pending\"")
    for h in d.get("herovr", []):
        hid = h.get("id", "?")
        if h.get("evidence") == "herovr":
            errs.append(f"[[herovr]] {hid}: evidence is \"herovr\"; fix: name our own tool that checked it (HEROVR is a checklist, not a source)")
        if h.get("check") not in CHECKS:
            errs.append(f"[[herovr]] {hid}: check {h.get('check')!r}; fix: use one of {', '.join(CHECKS)}")
        for k, v in h.items():
            if isinstance(v, str) and HEX_RUN.search(v):
                errs.append(f"[[herovr]] {hid}: {k} holds hex {HEX_RUN.search(v).group(0)!r}; fix: describe the item in words, with no peer address")
    return errs


# ---------- render ----------

def join(targets, d):
    """Render rows in targets.py order: each map item plus its targets.py `bytes`."""
    by_rva = {rva_of(it): it for it in d.get("item", [])}
    out = []
    for name, rva, kind, prio, hx in targets:
        it = dict(by_rva.get(rva, {}))
        it.setdefault("name", name)
        it.setdefault("kind", kind)
        it["prio"] = prio
        it["rem_rva"] = hex(rva)
        it["bytes"] = hx
        out.append(it)
    return out


def _row(it, vt):
    r = rva_of(it)
    if vt and r in vt:
        it = {**it, **found_fields(vt[r])}
    hx = it.get("bytes", "")
    sm2, sig, how = it.get("sm2", ""), it.get("sig", ""), it.get("how", "")
    return '| %s | %s | `%s` | %s | %s | %s | %s | %s |\n' % (
        it.get("name", ""), it.get("kind", ""), it.get("rem_rva", ""),
        ('`%s`' % hx.replace(' ', '')) if hx else '-',
        ('`%s`' % sm2) if sm2 else 'pending', ('`%s`' % sig) if sig else '-',
        it.get("conf", "pending"), how if how else '-')


def render_table(prio: int, items, vt) -> str:
    """The priority-`prio` table: mkmap.py's header and separator, then one row per item of that prio."""
    return TABLE_HEAD + "".join(_row(it, vt) for it in items if it.get("prio") == prio)


def render_herovr(herovr) -> str:
    rows = "".join('| %s %s | %s | %s | %s |\n' % (
        h.get("id", ""), h.get("what", ""), h.get("item") or '-', h.get("check", ""), h.get("evidence") or '-')
        for h in herovr)
    return HEROVR_HEAD + rows


def render_block(items, vt, herovr) -> str:
    """The text between the markers: the three priority tables under mkmap.py's headings, then the
    HEROVR table (when there are rows), then one blank line."""
    parts = []
    for p in (1, 2, 3):
        parts.append('### Priority %d. %s\n\n' % (p, PN[p]) + render_table(p, items, vt))
    if herovr:
        parts.append(HEROVR_HEADING + '\n\n' + render_herovr(herovr))
    return "\n".join(parts) + "\n"


def splice(doc: str, block: str) -> str:
    """`doc` with the text between its one BEGIN/END GENERATED pair replaced by `block`."""
    lines = doc.splitlines(keepends=True)
    begins = [i for i, l in enumerate(lines) if l.rstrip("\r\n") == BEGIN]
    ends = [i for i, l in enumerate(lines) if l.rstrip("\r\n") == END]
    if len(begins) != 1 or len(ends) != 1 or begins[0] > ends[0]:
        raise MarkerError("markers")
    b, e = begins[0], ends[0]
    return "".join(lines[:b + 1]) + block + "".join(lines[e:])


# ---------- TOML ----------

def _key(k):
    return k if BARE_KEY.fullmatch(k) else _str(k)


def _str(s):
    out = ['"']
    for ch in s:
        if ch == '"':
            out.append('\\"')
        elif ch == '\\':
            out.append('\\\\')
        elif ch == '\n':
            out.append('\\n')
        elif ch == '\t':
            out.append('\\t')
        elif ord(ch) < 0x20 or ord(ch) == 0x7f:
            out.append('\\u%04x' % ord(ch))
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def _val(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, float):
        if math.isnan(v):
            return "nan"
        if math.isinf(v):
            return "inf" if v > 0 else "-inf"
        return repr(v)
    if isinstance(v, str):
        return _str(v)
    if isinstance(v, list) and not any(isinstance(x, dict) for x in v):
        return "[" + ", ".join(_val(x) for x in v) + "]"
    raise TypeError(f"emit_toml: cannot write {type(v).__name__} {v!r}")


def _pairs(tbl):
    return "".join(f"{_key(k)} = {_val(v)}\n" for k, v in tbl.items())


def emit_toml(d) -> str:
    """TOML text for `d` (scalars, then [tables], then [[arrays of tables]], in dict order), under the
    fixed map.toml header comment. tomllib has no writer; this covers what map.toml holds."""
    scalars = {k: v for k, v in d.items() if not isinstance(v, dict) and not _is_aot(v)}
    out = [HEADER]
    if scalars:
        out.append("\n" + _pairs(scalars))
    for k, v in d.items():
        if isinstance(v, dict):
            out.append(f"\n[{_key(k)}]\n" + _pairs(v))
        elif _is_aot(v):
            for t in v:
                out.append(f"\n[[{_key(k)}]]\n" + _pairs(t))
    return "".join(out)


def _is_aot(v):
    return isinstance(v, list) and bool(v) and all(isinstance(x, dict) for x in v)


# ---------- CLI ----------

def read_text(path):
    with open(path, encoding="utf-8", newline="") as f:
        return f.read()


def write_text(path, text):
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.write(text)


def main(argv=None):
    ap = argparse.ArgumentParser(prog="fill_map.py", description="Merge VT matches into map.toml and render the address-map block.")
    ap.add_argument("--map", required=True)
    ap.add_argument("--vt")
    ap.add_argument("--targets", required=True)
    ap.add_argument("--doc", required=True)
    ap.add_argument("--check", action="store_true", help="write nothing; exit 1 on drift")
    args = ap.parse_args(argv)
    try:
        targets = load_targets(args.targets)
        d = load_map(args.map)
        doc = read_text(args.doc)
        vt = load_vt(args.vt) if args.vt else {}
        merged = merge(d, vt, receipt(args.vt)) if args.vt else d
    except (OSError, ValueError, tomllib.TOMLDecodeError) as e:
        print(f"fill_map: {e}; fix: pass readable --map, --targets, --doc (and --vt) files", file=sys.stderr)
        return 2
    errs = lint(merged, targets)
    if errs:
        for e in errs:
            print(f"fill_map: {args.map}: {e}", file=sys.stderr)
        return 1
    block = render_block(join(targets, merged), {}, merged.get("herovr", []))
    try:
        new_doc = splice(doc, block)
    except MarkerError:
        print(f"fill_map: {args.doc}: needs exactly one {BEGIN} / {END} pair; fix: restore the marker lines (WP-25b placement)", file=sys.stderr)
        return 2
    new_map = emit_toml(merged) if args.vt else None
    if args.check:
        drift = False
        if new_doc != doc:
            print(f"fill_map: {args.doc}: the generated block differs from {args.map}", file=sys.stderr)
            drift = True
        if new_map is not None and new_map != read_text(args.map):
            print(f"fill_map: {args.map}: differs from the merge of {args.vt}", file=sys.stderr)
            drift = True
        if drift:
            fix = FIX if new_map is None else FIX.replace(" --targets", f" --vt {args.vt} --targets")
            print(fix, file=sys.stderr)
            return 1
        return 0
    if new_map is not None:
        write_text(args.map, new_map)
    if new_doc != doc:
        write_text(args.doc, new_doc)
    return 0


if __name__ == "__main__":
    sys.exit(main())
