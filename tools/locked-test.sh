#!/usr/bin/env bash
set -euo pipefail

fail() {
  local code=$1 token=$2 fix=$3
  printf 'locked-test: %s fix: %s\n' "$token" "$fix" >&2
  exit "$code"
}
[[ $# == 3 && -n ${SCRATCH:-} ]] || fail 2 usage 'export SCRATCH=<your scratch dir>'
issue=$1 dest=$2 expected=$3
[[ $issue =~ ^[0-9]+$ && $expected =~ ^[a-fA-F0-9]{64}$ ]] || fail 2 usage 'export SCRATCH=<your scratch dir>'
snap=$SCRATCH/issues/$issue.md
if [[ ! -s $snap ]]; then
  mkdir -p "$SCRATCH/issues" || fail 3 no-snapshot 'run the §0 issue snapshot step'
  gh issue view "$issue" --json body -q .body > "$snap" || { rm -f "$snap"; fail 3 no-snapshot 'run the §0 issue snapshot step'; }
fi
dir=$(dirname "$dest")
mkdir -p "$dir" || fail 4 dest-unwritable "make $dir writable, then rerun"
tmp=$(mktemp "$dir/.locked-test.XXXXXX") || fail 4 dest-unwritable "make $dir writable, then rerun"
cleanup() { rm -f "$tmp"; }
trap cleanup EXIT
in_heading=0 in_rust=0
while IFS= read -r line || [[ -n $line ]]; do
  if [[ $line == '## Tests (locked)'* ]]; then in_heading=1; continue; fi
  if ((in_heading)) && [[ $line == '```rust'* ]]; then in_rust=1; continue; fi
  if ((in_rust)) && [[ $line == '```'* ]]; then break; fi
  if ((in_rust)) && [[ $line == *$'\r'* ]]; then fail 4 cr "rm \"\$SCRATCH/issues/$issue.md\" and rerun"; fi
done < "$snap"
awk '/^## Tests \(locked\)/{f=1} f&&/^```rust/{g=1;next} g&&/^```/{exit} g' "$snap" > "$tmp"
actual=$(sha256sum "$tmp" | cut -d' ' -f1)
[[ $actual == "${expected,,}" ]] || fail 4 sha-mismatch "rm \"\$SCRATCH/issues/$issue.md\" and rerun"
chmod 644 "$tmp" && mv "$tmp" "$dest" || fail 4 dest-unwritable "make $dir writable, then rerun"
printf 'locked-test: ok %s %s\n' "$dest" "$expected"
