#!/usr/bin/env python3
"""Read-only issue readiness, roadmap lint, and changed-file scope checks."""
from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT / "docs/plans/roadmap-v2"
BLOCKED = re.compile(r"[Bb]locked by:?\**\s*(.*)")
WP_RE = re.compile(r"WP-(\d{2}[a-d]?)")
PATH_HEADS = {"crates", "tools", "packaging", "docs", "decky", "games", "sessions", "recipes", "inject", "app", ".github", ".gitignore", "Cargo.toml", "Cargo.lock", "README.md", "SPEC.md", "PIPELINE.md", "AGENTS.md", "CHANGELOG.md"}


def read_issues(path: str | None) -> list[dict]:
    if path:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    proc = subprocess.run(
        ["gh", "issue", "list", "--state", "open", "--limit", "500", "--json", "number,title,body,labels"],
        cwd=ROOT, text=True, capture_output=True,
    )
    if proc.returncode:
        sys.stderr.write(proc.stderr)
        raise SystemExit(1)
    return json.loads(proc.stdout)


def issue_rows(issues: list[dict], device_label: str) -> list[dict]:
    open_numbers = {int(i["number"]) for i in issues}
    result = []
    for issue in issues:
        blockers: set[int] = set()
        external: list[str] = []
        for line in (issue.get("body") or "").splitlines():
            match = BLOCKED.search(line)
            if not match:
                continue
            text = match.group(1).strip()
            refs = [int(n) for n in re.findall(r"#(\d+)", text)]
            if refs:
                blockers.update(n for n in refs if n in open_numbers)
            elif not text.lower().startswith("none"):
                external.append(text)
        status = "blocked" if blockers else "blocked-external" if external else "ready"
        labels = [label["name"] for label in issue.get("labels", [])]
        result.append({
            "number": int(issue["number"]),
            "title": issue.get("title", ""),
            "status": status,
            "blockers": sorted(blockers),
            "external": external,
            "device": device_label in labels,
        })
    return result


def git(*args: str, cwd: Path = ROOT) -> str:
    proc = subprocess.run(["git", *args], cwd=cwd, text=True, capture_output=True)
    if proc.returncode:
        raise RuntimeError(proc.stderr.strip() or f"git {' '.join(args)} failed")
    return proc.stdout.strip()


def expand_braces(pattern: str) -> list[str]:
    match = re.search(r"\{([^{}]+)\}", pattern)
    if not match:
        return [pattern]
    out = []
    choices, start, depth = [], 0, 0
    inner = match.group(1)
    for i, char in enumerate(inner):
        if char == "{": depth += 1
        elif char == "}": depth -= 1
        elif char == "," and depth == 0:
            choices.append(inner[start:i]); start = i + 1
    choices.append(inner[start:])
    for choice in choices:
        out.extend(expand_braces(pattern[:match.start()] + choice + pattern[match.end():]))
    return out


def split_index_files(value: str) -> list[str]:
    pieces, start, braces, parens = [], 0, 0, 0
    for i, char in enumerate(value):
        if char == "{": braces += 1
        elif char == "}": braces = max(0, braces - 1)
        elif char == "(": parens += 1
        elif char == ")": parens = max(0, parens - 1)
        elif char == "," and braces == 0 and parens == 0:
            pieces.append(value[start:i]); start = i + 1
    pieces.append(value[start:])
    return [re.sub(r"\s*\([^)]*\)", "", p).strip() for p in pieces if p.strip()]


def index_rows(plan: Path) -> dict[str, dict]:
    text = (plan / "03-wp-index.md").read_text(encoding="utf-8")
    lines = text.splitlines()
    raw_rows: list[str] = []
    current = None
    for line in lines:
        if re.match(r"^- WP-\d{2}[a-d]? ", line):
            if current is not None:
                raw_rows.append(current)
            current = line
        elif current is not None:
            current += " " + line.strip()
    if current:
        raw_rows.append(current)
    rows: dict[str, dict] = {}
    for raw in raw_rows:
        parts = [p.strip() for p in raw.split(" | ")]
        match = re.match(r"- (WP-\d{2}[a-d]?) \(wave (\d+)\) (.*)", parts[0])
        if not match:
            continue
        wp, wave, title = match.groups()
        joined = " | ".join(parts[1:])
        fields = {}
        for name in ("closes", "files", "depends", "device"):
            m = re.search(rf"(?:^|\|)\s*{name}:\s*(.*?)(?=\s*\|\s*(?:closes|files|depends|device):|$)", joined)
            if m:
                fields[name] = m.group(1).strip()
        files_text = fields.get("files", "")
        files = split_index_files(files_text)
        dep_text = re.sub(r"\([^()]*\)", "", fields.get("depends", "-"))
        deps = [] if dep_text.strip() == "-" else [d for d in WP_RE.findall(dep_text)]
        rows[wp] = {"id": wp, "wave": int(wave), "title": title, "files": files, "depends": deps,
                    "device": fields.get("device", "none"), "closes": fields.get("closes", "")}
    return rows


def wp_file_ids(text: str) -> list[str]:
    return [m.group(0) for m in WP_RE.finditer(text)]


def files_block(plan: Path, wp: str) -> tuple[list[str], bool]:
    path = plan / f"{wp}.md"
    if not path.exists():
        return [], False
    lines = path.read_text(encoding="utf-8").splitlines()
    start = next((i for i, line in enumerate(lines) if line.startswith("**Files") or line.startswith("- **Files")), None)
    if start is None:
        return [], False
    block = [lines[start].split(":", 1)[1] if ":" in lines[start] else ""]
    for line in lines[start + 1:]:
        if line.startswith("**") or line.startswith("- **"):
            break
        if line.lstrip().startswith("- Not touched:"):
            continue
        block.append(line)
    content = "\n".join(block)
    if "as in the index" in content:
        return [], True
    paths = []
    for line in block:
        if re.match(r"\s*-\s*(?:not touched|untouched|must not|never|do not|if\b|no\b)", line, re.I):
            continue
        # Parenthetical line citations are prose, not file ownership tokens.
        line = re.sub(r"\([^()]*\)", "", line)
        for token in re.findall(r"`([^`]+)`", line):
            if "/" not in token and "." not in token:
                continue
            head = token.split("/", 1)[0]
            if head in PATH_HEADS and not token.endswith("/"):
                paths.extend(expand_braces(token))
    return paths, False


def matches(path: str, patterns: list[str]) -> bool:
    return any(fnmatch.fnmatchcase(path, pat) for pat in patterns)


def expanded(patterns: list[str]) -> list[str]:
    return [x for p in patterns for x in expand_braces(p)]


def deps_for(dep: str) -> list[str]:
    return [f"WP-01{x}" for x in "abcd"] if dep in {"01", "WP-01"} else [dep if dep.startswith("WP-") else "WP-" + dep]


def roadmap(plan: Path, base: str) -> dict:
    plan = plan.resolve()
    candidate_root = plan.parents[2] if len(plan.parents) >= 3 else ROOT
    repo_root = candidate_root if (candidate_root / ".git").exists() else ROOT
    rows = index_rows(plan)
    merges = git("log", "--merges", "--format=%s", base, cwd=repo_root).splitlines()
    merged: set[str] = set()
    for wp in rows:
        prefix = f"v2/wp-{wp[3:]}-"
        if any(prefix in subject for subject in merges):
            merged.add(wp)
    findings: list[dict] = []
    def lint(wp: str, rule: str, msg: str, fix: str) -> None:
        findings.append({"wp": wp, "rule": rule, "msg": msg, "fix": fix})

    plan0 = (plan / "00-run-the-pass.md").read_text(encoding="utf-8")
    frozen_text = plan0.split("Frozen after wave 1", 1)[-1].split("Enforcement", 1)[0]
    frozen_lines = [line for line in frozen_text.splitlines() if line.lstrip().startswith("-") and "`" in line]
    frozen = [(line, re.findall(r"`([^`]+)`", line), set(wp_file_ids(line))) for line in frozen_lines]
    all_row_files = {wp: expanded(row["files"]) for wp, row in rows.items()}
    for wp, row in rows.items():
        for dep in row["depends"]:
            for dep_id in deps_for(dep):
                if dep_id not in rows:
                    continue
                if rows[dep_id]["wave"] >= row["wave"] and not (wp.startswith("WP-01") and dep_id.startswith("WP-01")):
                    lint(wp, "depends-wave", f"depends on {dep_id} in wave {rows[dep_id]['wave']}", "move the dependency to an earlier wave")
        body = (plan / f"{wp}.md").read_text(encoding="utf-8") if (plan / f"{wp}.md").exists() else ""
        dep_line = next((ln for ln in body.splitlines() if ln.startswith("**Depends on") or ln.startswith("- **Depends on")), "")
        if dep_line:
            main = re.sub(r"\([^()]*\)", "", dep_line.split(". ", 1)[0])
            got = sorted(set(x for dep in wp_file_ids(main) if dep != wp for x in deps_for(dep)))
            expected = sorted(x for d in row["depends"] for x in deps_for(d))
            if got != expected:
                lint(wp, "depends-drift", f"body has {got}, index has {expected}", "make the Depends on line match the index")
        own, index_only = files_block(plan, wp)
        if not index_only:
            idx = expanded(row["files"])
            missing = [p for p in idx if not any(matches(p, [candidate]) or matches(candidate, [p]) for candidate in own)]
            extra = [p for p in own if not any(matches(p, [candidate]) or matches(candidate, [p]) for candidate in idx)]
            # A body-only frozen path may appear as a mention in this list.
            frozen_mentions = set(re.findall(r"`([^`]+)`", (plan / "00-run-the-pass.md").read_text(encoding="utf-8").split("Frozen after wave 1", 1)[-1].split("Enforcement", 1)[0]))
            other_files = [path for other, paths in all_row_files.items() if other != wp for path in paths]
            extra = [p for p in extra if p not in frozen_mentions and
                     not any(matches(p, [candidate]) or matches(candidate, [p]) for candidate in other_files) and
                     not any(matches(p, [candidate]) or matches(candidate, [p]) for candidate in frozen_mentions)]
            if missing or extra:
                lint(wp, "files-drift", f"missing {missing}; extra {extra}", "make the Files block match the index")
            if row["wave"] >= 2:
                for path in idx:
                    for source, frozen_paths, owners in frozen:
                        if any(matches(path, [candidate]) or matches(candidate, [path]) for candidate in frozen_paths) and wp not in owners:
                            lint(wp, "frozen-path", f"{path} is frozen by a line that does not name {wp}", "use the WP that owns the frozen path")
    wps = list(rows)
    for i, left in enumerate(wps):
        for right in wps[i + 1:]:
            if rows[left]["wave"] != rows[right]["wave"] or left.startswith("WP-01") and right.startswith("WP-01"):
                continue
            a, _ = files_block(plan, left); b, _ = files_block(plan, right)
            a = expanded(rows[left]["files"]); b = expanded(rows[right]["files"])
            clash = next((x for x in a if matches(x, b)), None) or next((x for x in b if matches(x, a)), None)
            if clash:
                lint(left, "wave-file-clash", f"{clash} also appears in {right}", "give the path to one WP")

    ready, blocked = [], []
    for wp, row in rows.items():
        if wp in merged:
            continue
        waiting = sorted({d for dep in row["depends"] for d in deps_for(dep) if d in rows and d not in merged})
        if waiting:
            blocked.append({"wp": wp, "waiting_on": waiting})
        else:
            ready.append(wp)
    owner_device = [wp for wp, row in rows.items() if wp in merged and row["device"].lower() not in {"none", "-", ""}]
    q_ready = []
    qfile = (plan / "02-queues-and-decisions.md").read_text(encoding="utf-8")
    in_q = False
    for line in qfile.splitlines():
        if line.startswith("## 5."): in_q = True; continue
        if in_q and line.startswith("## "): break
        if line.startswith("| Q"):
            cols = [c.strip() for c in line.strip("| ").split("|")]
            if len(cols) < 5:
                continue
            qid, needs = cols[0], cols[3]
            if not re.fullmatch(r"Q\d+", qid):
                continue
            refs = re.findall(r"(?<!Q)(?:WP-)?(\d{2})\b", needs)
            not_ready = [f"WP-{n}" for n in refs if f"WP-{n}" in rows and f"WP-{n}" not in merged]
            after = re.findall(r"Q\d+", needs)
            if not not_ready:
                q_ready.append(qid + (" after " + ",".join(after) if after else ""))
    ods = []
    in_od = False
    for line in qfile.splitlines():
        if line.startswith("## 7."): in_od = True; continue
        if in_od and line.startswith("## "): break
        if in_od and line.startswith("| OD"):
            cols = [c.strip() for c in line.strip("| ").split("|")]
            if len(cols) >= 3 and cols[0] != "OD":
                ods.append({"id": cols[0], "question": cols[1], "answer": cols[2]})
    return {"ready": ready, "blocked": blocked, "owner_device": owner_device, "q_ready": q_ready, "ods": ods, "lint": findings}


def scope_check(wp_arg: str, plan: Path, base: str) -> int:
    wp = wp_arg if wp_arg.startswith("WP-") else "WP-" + wp_arg
    rows = index_rows(plan)
    if wp not in rows:
        print(f"scope: unknown {wp} (fix: use a WP in the index)", file=sys.stderr)
        return 1
    try:
        plan = plan.resolve()
        candidate_root = plan.parents[2] if len(plan.parents) >= 3 else ROOT
        repo_root = candidate_root if (candidate_root / ".git").exists() else ROOT
        merge_base = git("merge-base", "HEAD", base, cwd=repo_root)
        changed = set(git("diff", "--name-only", merge_base, cwd=repo_root).splitlines())
        changed.update(git("ls-files", "--others", "--exclude-standard", cwd=repo_root).splitlines())
    except RuntimeError as exc:
        print(f"scope: {exc}", file=sys.stderr)
        return 1
    allowed = expanded(rows[wp]["files"])
    failure = 0
    for path in sorted(changed):
        if not matches(path, allowed):
            print(f"scope: {path} is not in {wp}'s files (fix: move the change to the WP that owns it)", file=sys.stderr)
            failure = 1
    return failure


def main() -> int:
    parser = argparse.ArgumentParser(add_help=True)
    parser.add_argument("--json-file")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--device-label", default="needs-owner-device")
    parser.add_argument("--roadmap", action="store_true")
    parser.add_argument("--scope")
    parser.add_argument("--plan-dir", default=str(PLAN))
    parser.add_argument("--base")
    args = parser.parse_args()
    if args.roadmap or args.scope:
        plan = Path(args.plan_dir)
        base = args.base or os.environ.get("FLAT3D_BASE") or "origin/main"
        if args.scope:
            return scope_check(args.scope, plan, base)
        try:
            result = roadmap(plan, base)
        except (OSError, RuntimeError, ValueError) as exc:
            print(f"issues_report: {exc}", file=sys.stderr)
            return 1
        if args.json:
            print(json.dumps(result, sort_keys=True))
        else:
            print("ready:", ", ".join(result["ready"]))
            print("blocked:", ", ".join(f"{x['wp']} waits on {','.join(x['waiting_on'])}" for x in result["blocked"]))
            print("owner-device:", ", ".join(result["owner_device"]))
            print("q-ready:", ", ".join(result["q_ready"]))
            print("ods:", "; ".join(f"{x['id']} {x['answer']}" for x in result["ods"]))
            for item in result["lint"]:
                print(f"{item['wp']}: {item['rule']}: {item['msg']} (fix: {item['fix']})")
        return 1 if result["lint"] else 0
    try:
        rows = issue_rows(read_issues(args.json_file), args.device_label)
    except (OSError, json.JSONDecodeError) as exc:
        print(f"issues_report: {exc}", file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps(rows, sort_keys=True))
    else:
        print("# | title | status | blockers | device")
        print("---|---|---|---|---")
        for row in rows:
            blockers = ", ".join(f"#{n}" for n in row["blockers"])
            print(f"{row['number']} | {row['title'].replace('|', '\\|')} | {row['status']} | {blockers} | {'yes' if row['device'] else ''}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
