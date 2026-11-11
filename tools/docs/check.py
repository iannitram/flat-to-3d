#!/usr/bin/env python3
"""Read-only consistency checks for the repository's documentation."""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

RULES = ("status", "links", "mermaid", "usage", "paths", "ids", "generated", "private")
PRIVATE = [
    r"\bI[a]n\b", "Hyp" + "erion", "B:" + "/", r"/home/[a-z_][a-z0-9_-]*",
    "qw" + "en", "FF8" + "S5", "sand" + "isk", r"\bMDT\b",
    "cra" + "ft-build", r"(?<![\w/.])ian" + "nitram(?!/)",
    r"[\w.+-]+@[\w-]+\.[a-z]{2,}",
]
PATHS_RE = re.compile(r"(?<![\w./-])(?:inject/|app/src\b)")
USAGE_RE = re.compile(r'usage: flat3d-inject ([^"\\]*)')
RULING_RE = re.compile(r"\b[Rr]ulings?\s+([0-9]+(?:\s*(?:,?\s*and|,|-)\s*[0-9]+)*)")
DATE_RE = re.compile(r"\d{4}-\d{2}-\d{2}")
M_RE = re.compile(r"\bM(\d+)\b")


@dataclass(frozen=True, order=True)
class Finding:
    path: str
    line: int
    rule: str
    msg: str
    fix: str


def rel(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def markdown_files(root: Path) -> list[Path]:
    try:
        out = subprocess.run(["git", "-C", str(root), "ls-files", "-z"], check=True, capture_output=True).stdout
        paths = [root / p for p in out.decode().split("\0") if p]
    except (OSError, subprocess.CalledProcessError):
        paths = [p for p in root.rglob("*") if p.is_file()]
    return [p for p in paths if p.suffix.lower() == ".md"]


def selected_files(root: Path, args: argparse.Namespace) -> list[Path]:
    if args.changed:
        if args.paths:
            raise ValueError("--changed cannot be used with PATHS")
        try:
            base = subprocess.run(["git", "-C", str(root), "merge-base", args.changed, "HEAD"], check=True, capture_output=True, text=True).stdout.strip()
            names = subprocess.run(["git", "-C", str(root), "diff", "--name-only", "--diff-filter=d", base], check=True, capture_output=True, text=True).stdout.splitlines()
        except (OSError, subprocess.CalledProcessError) as exc:
            raise ValueError("--changed requires a git repository and a resolvable BASE") from exc
        paths = [root / n for n in names]
    elif args.paths:
        paths = []
        for name in args.paths:
            p = root / name
            if not p.is_file():
                raise ValueError(f"path does not exist: {name}")
            paths.append(p)
    else:
        paths = [root / p for p in subprocess.run(["git", "-C", str(root), "ls-files", "-z"], capture_output=True, check=False).stdout.decode(errors="replace").split("\0") if p] if (root / ".git").exists() else [p for p in root.rglob("*") if p.is_file()]
    result = []
    for p in paths:
        if not p.is_file():
            continue
        name = rel(root, p)
        if name.startswith(("docs/plans/", "third_party/")) or "tests" in Path(name).parts or p.name in ("Cargo.lock", "pnpm-lock.yaml"):
            continue
        try:
            p.read_text(encoding="utf-8")
        except (UnicodeError, OSError):
            continue
        result.append(p)
    return sorted(set(result), key=lambda p: rel(root, p))


def add(out: list[Finding], p: Path, n: int, rule: str, msg: str, fix: str):
    out.append(Finding(p.as_posix(), n, rule, msg, fix))


def read(root: Path, path: str) -> list[str]:
    p = root / path
    try:
        return p.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError):
        return []


def allowlist(root: Path, filename: str) -> list[tuple[str, str | None]]:
    p = root / filename
    if not p.exists():
        return []
    rows = []
    for line in p.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        glob, sep, sub = line.partition("\t")
        rows.append((glob, sub if sep else None))
    return rows


def run(root: Path, files: list[Path], rules: set[str], allow: list[tuple[str, str | None]]) -> list[Finding]:
    out: list[Finding] = []
    names = {rel(root, p) for p in files}
    texts: dict[str, list[str]] = {}
    for p in files:
        try:
            texts[rel(root, p)] = p.read_text(encoding="utf-8").splitlines()
        except (OSError, UnicodeError):
            continue

    code_map = read(root, "docs/code-map.md")
    if "status" in rules and "docs/code-map.md" in names:
        start = next((i for i, line in enumerate(code_map) if line.startswith("## Parts")), None)
        if start is not None:
            table = next((i for i in range(start + 1, len(code_map)) if code_map[i].startswith("|")), None)
            if table is not None:
                previous_dir = Path(".")
                for n in range(table + 2, len(code_map)):
                    line = code_map[n]
                    if not line.startswith("|"):
                        break
                    cells = [c.strip() for c in line.strip("|").split("|")]
                    if len(cells) < 2 or not re.match(r"(?:present|partly present|stub|skeleton)\b", cells[1]):
                        continue
                    paths = re.findall(r"`([^`]+)`", cells[0])
                    for cellpath in paths:
                        brace = re.search(r"\{([^{}]+)\}", cellpath)
                        expanded = [cellpath[:brace.start()] + alt + cellpath[brace.end():] for alt in brace.group(1).split(",")] if brace else [cellpath]
                        for item in expanded:
                            if "*" in item:
                                if not list(root.glob(item)):
                                    add(out, Path("docs/code-map.md"), n + 1, "status", f"path glob has no matches: {item}", "create the path, or mark the row planned")
                                previous_dir = Path(item.rstrip("/")).parent
                                continue
                            rooted = "/" in item
                            item = item.rstrip("/")
                            item_path = Path(item)
                            if not rooted:
                                resolved = previous_dir / item_path
                            elif (root / item_path).exists():
                                resolved = item_path
                            else:
                                resolved = previous_dir / item_path
                                if not (root / resolved).exists():
                                    resolved = None
                            if resolved is None:
                                add(out, Path("docs/code-map.md"), n + 1, "status", f"missing path: {item}", "create the path, or mark the row planned")
                                previous_dir = (previous_dir / item_path).parent
                                continue
                            if not (root / resolved).exists():
                                add(out, Path("docs/code-map.md"), n + 1, "status", f"missing path: {item}", "create the path, or mark the row planned")
                            previous_dir = resolved.parent

    for name, lines in texts.items():
        p = Path(name)
        in_fence = False
        fence_lang = None
        in_mv_map = False
        for n, line in enumerate(lines, 1):
            if line.strip() == "<!-- mv-map -->":
                in_mv_map = True
            elif line.strip() == "<!-- /mv-map -->":
                in_mv_map = False
            if line.lstrip().startswith("```"):
                if in_fence:
                    in_fence = False
                    fence_lang = None
                else:
                    in_fence = True
                    fence_lang = line.strip()[3:].strip().lower()
                continue
            if "links" in rules and p.suffix == ".md" and not in_fence:
                for target in re.findall(r"\[[^\]]*\]\(([^)]+)\)", line):
                    parts = target.split()
                    target = parts[0] if parts else ""
                    target = target.split("#", 1)[0].split("?", 1)[0]
                    if not target or re.match(r"(?:https?://|mailto:|#)", target):
                        continue
                    base = root / target.lstrip("/") if target.startswith("/") else root / p.parent / target
                    if not base.exists():
                        add(out, p, n, "links", f"dead link: {target}", "point the link at an existing file")
            if "paths" in rules and name in ("docs/ROADMAP.md", "docs/code-map.md") and not in_mv_map and not line.strip().startswith("<!-- mv-map -->") and not line.strip().startswith("<!-- /mv-map -->") and PATHS_RE.search(line):
                add(out, p, n, "paths", "legacy path token", "write the crates/engine/ or crates/app/src path")
            if "private" in rules and not any(
                fnmatch.fnmatch(name, glob) and (sub is None or sub in line)
                for glob, sub in allow
            ):
                for pat in PRIVATE:
                    if re.search(pat, line, re.IGNORECASE if pat in ("Hyp" + "erion", "qw" + "en", "sand" + "isk", "cra" + "ft-build") else 0):
                        add(out, p, n, "private", "private term", "remove the word, or add an allow line to tools/docs/private-allow.txt")
                        break
            if "mermaid" in rules and p.suffix == ".md":
                if in_fence and fence_lang == "mermaid":
                    probe = re.sub(r'"[^"]*"', "", line)
                    probe = re.sub(r"\|[^|]*\|", "", probe)
                    probe = re.sub(r"\[[^\]]*\]|\([^)]*\)|\{[^}]*\}", "", probe)
                    probe = probe.replace(" & ", " ")
                    parts = re.split(r"-{2,}>|={2,}>|-\.+->|-{3,}", probe)
                    if any(len(part.split()) > 1 for part in parts[1:-1]):
                        add(out, p, n, "mermaid", "2 statements on one line", "put one statement per line")

    if "usage" in rules and "docs/code-map.md" in names:
        cli_paths = [root / "crates/engine/src/cli.rs", *((root / "crates/engine/src/cli").glob("*.rs") if (root / "crates/engine/src/cli").is_dir() else [])]
        commands_i = next((i for i, l in enumerate(code_map) if l.startswith("## Commands")), None)
        if commands_i is None:
            add(out, Path("docs/code-map.md"), 1, "usage", "missing ## Commands table", "add a ## Commands table to code-map")
        command_words = set()
        if commands_i is not None:
            for line in code_map[commands_i + 1:]:
                if line.startswith("## "):
                    break
                if line.startswith("|"):
                    cell = line.strip("|").split("|", 1)[0]
                    marks = re.findall(r"`([^`]+)`", cell)
                    command_words.update((x.split()[0] for x in marks if x.split()))
                    if not marks and cell.split():
                        command_words.add(cell.split()[0])
        for cp in cli_paths:
            if cp.is_file():
                for n, line in enumerate(cp.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
                    for usage in USAGE_RE.findall(line):
                        for alt in usage_commands(usage):
                            word = alt.strip().split()[0] if alt.strip() else ""
                            if word.startswith(("{", "<", "[")):
                                continue
                            if not command_words or word not in command_words:
                                add(out, Path("docs/code-map.md"), commands_i + 1 if commands_i is not None else 1, "usage", f"usage command missing from table: {word}", f"add a {word} row to code-map ## Commands")

    if "ids" in rules:
        spec = "\n".join(read(root, "SPEC.md"))
        decisions = "\n".join(read(root, "docs/notes/DECISIONS.md"))
        for name, lines in texts.items():
            if not name.endswith(".md"):
                continue
            for n, line in enumerate(lines, 1):
                for mid in M_RE.findall(line):
                    if not re.search(rf"\|\s*M{mid}\s*\|", spec):
                        add(out, Path(name), n, "ids", f"unknown spec item M{mid}", "cite an id that exists in SPEC.md")
                for group in RULING_RE.findall(DATE_RE.sub("", line)):
                    nums = []
                    for lo, hi in re.findall(r"(\d+)(?:\s*-\s*(\d+))?", group):
                        if hi and 0 <= int(hi) - int(lo) <= 1000:
                            nums.extend(range(int(lo), int(hi) + 1))
                        else:
                            nums.extend(int(x) for x in (lo, hi) if x)
                    for num in nums:
                        if len(str(num)) == 4:
                            continue
                        if not re.search(rf"^\|\s*{num}\s*\|", decisions, re.M) and not re.search(rf"^Ruling\s+{num}\s", decisions, re.M):
                            add(out, Path(name), n, "ids", f"unknown ruling {num}", "cite an id that exists in DECISIONS.md")

    if "generated" in rules:
        for name, lines in texts.items():
            if not name.endswith(".md"):
                continue
            marks = [(i, l.strip()) for i, l in enumerate(lines, 1) if l.strip() in ("<!-- BEGIN GENERATED -->", "<!-- END GENERATED -->")]
            if marks and (len(marks) != 2 or marks[0][1] != "<!-- BEGIN GENERATED -->" or marks[1][1] != "<!-- END GENERATED -->"):
                add(out, Path(name), marks[-1][0], "generated", "generated markers must be one ordered pair", "keep one BEGIN/END GENERATED pair")
        recipe = texts.get("recipes/README.md", []) if "recipes/README.md" in names else []
        if recipe:
            begin = next((i for i, x in enumerate(recipe) if x.strip() == "<!-- BEGIN GENERATED -->"), None)
            end = next((i for i, x in enumerate(recipe) if x.strip() == "<!-- END GENERATED -->"), None)
            src = "\n".join(read(root, "crates/engine/src/recipes.rs"))
            if begin is not None and end is not None and begin < end:
                for key in re.findall(r'"([^"]+)"', "\n".join(re.findall(r"(?:UEVR|GEO11)_KEYS\s*:[^{=]*=\s*&?\[(.*?)\]", src, re.S))):
                    if f"`{key}`" not in "\n".join(recipe[begin + 1:end]):
                        add(out, Path("recipes/README.md"), begin + 1, "generated", f"generated route key missing: {key}", "add the key to the generated block")
        runfile = root / "tools/session/session.py"
        sheet = root / "docs/STEP0-RUNSHEET.md"
        if "docs/STEP0-RUNSHEET.md" in names and sheet.is_file() and runfile.is_file() and sheet.read_text(encoding="utf-8").splitlines()[:1] and sheet.read_text(encoding="utf-8").splitlines()[0].startswith("<!-- generated by tools/session/session.py render"):
            try:
                rendered = subprocess.run([sys.executable, str(runfile), "render", "sessions/step0.toml"], cwd=root, check=True, capture_output=True, text=True).stdout
                if sheet.read_text(encoding="utf-8") != rendered:
                    add(out, Path("docs/STEP0-RUNSHEET.md"), 1, "generated", "runsheet differs from renderer", "python3 tools/session/session.py render sessions/step0.toml > docs/STEP0-RUNSHEET.md")
            except (OSError, subprocess.CalledProcessError) as exc:
                add(out, Path("docs/STEP0-RUNSHEET.md"), 1, "generated", f"runsheet renderer failed: {exc}", "fix tools/session/session.py render")

    return sorted(out)


def usage_commands(usage: str) -> list[str]:
    """Split a usage string into commands: only a spaced ` | ` outside [...] and <...> separates them.

    `[--filter <status>|all]` is an option-value alternative and `launch add|remove` is one command with sub-verbs.
    """
    out, depth, start = [], 0, 0
    for i, ch in enumerate(usage):
        if ch in "[<{":
            depth += 1
        elif ch in "]>}":
            depth = max(0, depth - 1)
        elif ch == "|" and depth == 0 and usage[i - 1:i] == " " and usage[i + 1:i + 2] == " ":
            out.append(usage[start:i])
            start = i + 1
    out.append(usage[start:])
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--only")
    parser.add_argument("--allow", default="tools/docs/private-allow.txt")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--changed")
    parser.add_argument("paths", nargs="*")
    args = parser.parse_args()
    if args.only:
        chosen = set(args.only.split(","))
        bad = chosen - set(RULES)
        if bad:
            print(f"unknown rule(s): {', '.join(sorted(bad))}; valid names: {', '.join(RULES)}", file=sys.stderr)
            return 2
    else:
        chosen = set(RULES)
    root = args.root.resolve()
    try:
        files = selected_files(root, args)
        findings = run(root, files, chosen, allowlist(root, args.allow))
    except (ValueError, OSError) as exc:
        print(f"{exc} (fix: use an existing path and valid options)", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps([asdict(f) for f in findings], ensure_ascii=False))
    else:
        for f in findings:
            print(f"{f.path}:{f.line}: {f.rule}: {f.msg} (fix: {f.fix})")
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
