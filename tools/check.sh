#!/usr/bin/env bash
set -euo pipefail

usage() { printf '%s\n' 'usage: tools/check.sh [--ci] [--only rust|py|decky] [--list] [--tests <cargo test args>]'; }
ci=0 only='' tests_args='' list=0
while (($#)); do
  case $1 in
    --ci) ci=1; shift ;;
    --only) (($# >= 2)) || { usage >&2; exit 2; }; only=$2; shift 2 ;;
    --tests) (($# >= 2)) || { usage >&2; exit 2; }; tests_args=$2; shift 2 ;;
    --list) list=1; shift ;;
    *) usage >&2; exit 2 ;;
  esac
done
[[ -z $only || $only == rust || $only == py || $only == decky ]] || { usage >&2; exit 2; }
root=$(git rev-parse --show-toplevel)
cd "$root"
if [[ -z ${CARGO_TARGET_DIR:-} ]]; then export CARGO_TARGET_DIR=$root/target; fi
if ((ci)); then export FLAT3D_REQUIRE_HOST_TOOLS=1; fi

steps=(
  'rust fmt' 'rust clippy' 'rust clippy-xr' 'rust clippy-capture' 'rust clippy-msvc' 'rust test' 'rust lock' 'rust app-io'
  'py py-compile' 'py unittest' 'py decky-py' 'py shellcheck' 'py docs' 'py fill-map' 'py scope'
  'decky decky'
)
if ((list)); then for entry in "${steps[@]}"; do printf '%s\n' "$entry"; done; exit 0; fi

passed=0 skipped=0 na=0 failed=0
run_step() {
  local group=$1 name=$2 fn=$3 status
  [[ -z $only || $only == "$group" ]] || return 0
  if "$fn"; then
    printf 'ok %s\n' "$name"; ((passed+=1))
  else
    status=$?
    case $status in
      2) if ((ci)); then printf 'FAIL %s: fix: %s\n' "$name" "${step_fix:-install the missing tool}"; ((failed+=1)); else printf 'skip %s: %s (fix: %s)\n' "$name" "${step_why:-tool missing}" "${step_fix:-install the missing tool}"; ((skipped+=1)); fi ;;
      3) printf 'n/a %s: %s\n' "$name" "${step_why:-input has not landed yet}"; ((na+=1)) ;;
      *) printf 'FAIL %s: fix: %s\n' "$name" "${step_fix:-see the command output above}"; ((failed+=1)) ;;
    esac
  fi
}

step_fmt() { cargo fmt --all --check; }
step_clippy() { cargo clippy --locked --workspace --all-targets -- -D warnings; }
step_clippy_xr() { cargo clippy --locked -p flat3d-viewer --all-targets --features xr -- -D warnings; }
step_clippy_capture() {
  if command -v pkg-config >/dev/null 2>&1 && pkg-config --exists libpipewire-0.3; then
    cargo clippy --locked -p flat3d-viewer --all-targets --features xr,capture -- -D warnings
  else step_why='pkg-config or libpipewire-0.3 is missing'; step_fix='distrobox enter craft-build -- tools/check.sh'; return 2; fi
}
step_clippy_msvc() {
  if rustup target list --installed | grep -qx x86_64-pc-windows-msvc; then
    cargo clippy --locked -p flat3d-uevr-inject --target x86_64-pc-windows-msvc -- -D warnings
  else step_why='x86_64-pc-windows-msvc target is missing'; step_fix='rustup target add x86_64-pc-windows-msvc'; return 2; fi
}
step_test() {
  rt=$(mktemp -d) && trap 'rm -rf "$rt"' RETURN && if [[ -n $tests_args ]]; then
    read -r -a test_args <<< "$tests_args" && XDG_RUNTIME_DIR="$rt" cargo test --locked "${test_args[@]}"
  else XDG_RUNTIME_DIR="$rt" cargo test --locked --workspace; fi
}
step_lock() { git diff --exit-code -- Cargo.lock; }
step_app_io() {
  [[ -d crates/app/src ]] || return 3
  python3 -c 'from pathlib import Path; import re,sys; p=Path("crates/app/src"); bad=[]; pat=re.compile(r"\bstd::(?:fs|process|net|thread)\b|\bCommand::new\s*\("); use=re.compile(r"\buse\s+std::\{[^}]*\b(?:fs|process|net|thread)\b[^}]*\}");
for f in p.rglob("*.rs"):
 if f.name!="engine.rs":
  for n,line in enumerate(f.read_text(errors="replace").splitlines(),1):
   if pat.search(line) or use.search(line): bad.append(f"{f}:{n}")
print("\n".join(bad)); sys.exit(bool(bad))'
}
step_py_compile() {
  mapfile -t py_files < <(git ls-files '*.py') && python3 -W error -m py_compile "${py_files[@]}"
}
step_unittest() {
  files=$(git ls-files '*test_*.py') && dirs=$(printf '%s\n' "$files" | xargs -r -n1 dirname | sort -u | grep -vx 'decky/tests' || true) && \
  for d in $dirs; do python3 -m unittest discover -s "$d" -p 'test_*.py' || return 1; done
}
step_decky_py() {
  [[ -n $(git ls-files 'decky/tests/test_*.py') ]] || return 3
  (cd decky && python3 -m unittest discover -s tests -p 'test_*.py')
}
step_shellcheck() {
  mapfile -t shell_files < <(git ls-files '*.sh') || return 1
  if command -v shellcheck >/dev/null 2>&1; then shellcheck "${shell_files[@]}"
  else
    for f in "${shell_files[@]}"; do bash -n "$f" || return 1; done && step_why='shellcheck is not installed; bash -n fallback passed' && step_fix="CI's shellcheck step is the gate" && return 2
  fi
}
step_docs() { [[ -f docs/workbench.md ]] || return 3; python3 tools/docs/check.py; }
step_fill_map() { [[ -f tools/re/fill_map.py ]] || return 3; python3 tools/re/fill_map.py --check --map games/spiderman2/map.toml --targets tools/xmatch/targets.py --doc docs/spiderman2/address-map.md; }
step_scope() {
  branch=${GITHUB_HEAD_REF:-$(git rev-parse --abbrev-ref HEAD)}
  [[ -f docs/plans/roadmap-v2/03-wp-index.md ]] || return 3
  if [[ $branch =~ ^v2/wp-([0-9]{2}[a-d]?)- ]]; then python3 tools/pm/issues_report.py --scope "WP-${BASH_REMATCH[1]}" --base "${FLAT3D_BASE:-origin/main}"; else return 3; fi
}
step_decky() {
  [[ -f decky/package.json ]] || return 3
  command -v pnpm >/dev/null 2>&1 || { step_why='pnpm is missing'; step_fix='install pnpm (CI uses 10.34.9)'; return 2; }
  if ((ci)); then pnpm -C decky install --frozen-lockfile && pnpm -C decky run typecheck && pnpm -C decky run build
  else pnpm -C decky install --frozen-lockfile --offline && pnpm -C decky run typecheck && pnpm -C decky run build; fi
}

if [[ -z $only || $only == rust ]]; then
  if [[ -f Cargo.toml ]]; then
    run_step rust fmt step_fmt
    run_step rust clippy step_clippy
    run_step rust clippy-xr step_clippy_xr
    run_step rust clippy-capture step_clippy_capture
    run_step rust clippy-msvc step_clippy_msvc
    run_step rust test step_test
    run_step rust lock step_lock
    run_step rust app-io step_app_io
  else printf 'n/a rust: no workspace yet\n'; ((na+=1)); fi
fi
if [[ -z $only || $only == py ]]; then
  run_step py py-compile step_py_compile
  run_step py unittest step_unittest
  run_step py decky-py step_decky_py
  run_step py shellcheck step_shellcheck
  run_step py docs step_docs
  run_step py fill-map step_fill_map
  run_step py scope step_scope
fi
if [[ -z $only || $only == decky ]]; then run_step decky decky step_decky; fi
printf 'check.sh: %d passed, %d skipped, %d n/a, %d failed\n' "$passed" "$skipped" "$na" "$failed"
((failed == 0))
