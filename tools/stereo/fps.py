#!/usr/bin/env python3
"""Summarize MangoHud CSV frame rates."""
import argparse
import csv
import json
import math
import statistics
import sys
from pathlib import Path


def read_mangohud(path):
    lines = Path(path).read_text(encoding="utf-8-sig").splitlines()
    header = next((i for i, line in enumerate(lines) if line.split(",", 1)[0].strip().lower() == "fps"), None)
    if header is None:
        raise ValueError("missing MangoHud fps header")
    rows = []
    for row in csv.DictReader(lines[header:]):
        try:
            fps = float(row["fps"])
            ft = float(row["frametime"]) if row.get("frametime", "").strip() else None
        except (ValueError, TypeError, KeyError):
            continue
        if math.isfinite(fps) and fps > 0 and (ft is None or (math.isfinite(ft) and ft > 0)):
            rows.append((fps, ft))
    if not rows:
        raise ValueError("no valid fps rows")
    return rows


def _nearest(values, p):
    ordered = sorted(values)
    return ordered[max(0, math.ceil(p * len(ordered)) - 1)]


def summarize(rows):
    if not rows:
        raise ValueError("no valid fps rows")
    fps = [r[0] for r in rows]
    frametimes = [r[1] for r in rows if r[1] is not None]
    return {"n": len(rows), "p50": statistics.median(fps), "p1": 1000 / _nearest(frametimes, .99) if frametimes else _nearest(fps, .01)}


def main(argv=None):
    p = argparse.ArgumentParser()
    p.add_argument("csv")
    p.add_argument("--min", type=float, default=30)
    p.add_argument("--skip-rows", type=int, default=0)
    p.add_argument("--json", action="store_true")
    a = p.parse_args(argv)
    try:
        rows = read_mangohud(a.csv)[a.skip_rows:]
        result = summarize(rows)
    except (OSError, ValueError) as e:
        print(str(e), file=sys.stderr)
        return 2
    result["min"] = a.min
    result["pass"] = result["p50"] >= a.min
    print(json.dumps(result, sort_keys=True) if a.json else f"n={result['n']} p50={result['p50']:.2f} p1={result['p1']:.2f} pass={result['pass']}")
    return 0 if result["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
