#!/usr/bin/env bash
set -euo pipefail

usage() { printf '%s\n' 'usage: packaging/pin.sh <key> [--file <path>] [--check] [--dry-run] [--pins <path>] | --all-tbd [--dry-run] [--pins <path>]' >&2; }
token_exit() {
  local code=$1 token=$2 retryable=$3 fix=$4
  printf 'pin: %s retryable=%s fix: %s\n' "$token" "$retryable" "$fix" >&2
  exit "$code"
}
pins=packaging/pins key='' file='' check=0 dry=0 all=0
while (($#)); do
  case $1 in
    --pins) (($# >= 2)) || { usage; token_exit 2 usage no 'name the bad line or key'; }; pins=$2; shift 2 ;;
    --file) (($# >= 2)) || { usage; token_exit 2 usage no 'name the bad line or key'; }; file=$2; shift 2 ;;
    --check) check=1; shift ;;
    --dry-run) dry=1; shift ;;
    --all-tbd) all=1; shift ;;
    --*) usage; token_exit 2 usage no 'name the bad line or key' ;;
    *) [[ -z $key ]] || { usage; token_exit 2 usage no 'name the bad line or key'; }; key=$1; shift ;;
  esac
done
if [[ -n $file && $all == 1 ]]; then usage; token_exit 2 usage no 'name the bad line or key'; fi
if [[ ! -f $pins ]]; then token_exit 2 bad-pins no 'name the bad line or key'; fi

declare -A field=()
declare -a keys=()
while IFS= read -r line || [[ -n $line ]]; do
  [[ -z $line || $line == \#* ]] && continue
  [[ $line == *': '* ]] || token_exit 2 bad-pins no 'name the bad line or key'
  lhs=${line%%: *}; value=${line#*: }
  [[ $lhs == *.* && -n $value ]] || token_exit 2 bad-pins no 'name the bad line or key'
  k=${lhs%%.*}; f=${lhs#*.}
  [[ $k =~ ^[a-z0-9-]+$ ]] || token_exit 2 bad-pins no 'name the bad line or key'
  if [[ $k == crate || $k == npm ]]; then
    [[ $f =~ ^[A-Za-z0-9@/_+.-]+$ ]] || token_exit 2 bad-pins no 'name the bad line or key'
  else
    [[ $f =~ ^[a-z0-9-]+$ ]] || token_exit 2 bad-pins no 'name the bad line or key'
  fi
  [[ -z ${field["$lhs"]+x} ]] || token_exit 2 bad-pins no 'name the bad line or key'
  field["$lhs"]=$value
  case " ${keys[*]} " in *" $k "*) ;; *) keys+=("$k");; esac
done < "$pins"

validate() {
  local k f sha url fetch
  for k in "${keys[@]}"; do
    fetch=${field["$k.fetch"]-}
    if [[ $k == crate || $k == npm ]]; then continue; fi
    if [[ $fetch == auto ]]; then
      for f in version url size sha256 license; do [[ -n ${field["$k.$f"]-} ]] || token_exit 2 bad-pins no 'name the bad line or key'; done
      sha=${field["$k.sha256"]}; url=${field["$k.url"]}
      [[ ${field["$k.size"]} == TBD-at-pin || ${field["$k.size"]} =~ ^[0-9]+$ ]] || token_exit 2 bad-pins no 'name the bad line or key'
      [[ $sha == TBD-at-pin || $sha =~ ^[a-fA-F0-9]{64}$ ]] || token_exit 2 bad-pins no 'name the bad line or key'
      if [[ $sha != TBD-at-pin && ( $url == TBD-at-pin || ${field["$k.size"]} == TBD-at-pin ) ]]; then token_exit 2 bad-pins no 'name the bad line or key'; fi
      [[ $url != TBD-at-pin || -n ${field["$k.page"]-} ]] || token_exit 2 bad-pins no 'name the bad line or key'
    elif [[ $fetch == manual ]]; then
      [[ -n ${field["$k.page"]-} && -n ${field["$k.license"]-} ]] || token_exit 2 bad-pins no 'name the bad line or key'
    elif [[ -n $fetch ]]; then
      token_exit 2 bad-pins no 'name the bad line or key'
    fi
  done
}
validate

pin_one() {
  local k=$1 sha url size page tmp base cap actual actual_size got_digest route repo tag name checks newsha newsize stamp out field_size fill_size
  [[ $k != crate && $k != npm ]] || token_exit 2 not-a-fetch-key no 'name the bad line or key'
  fetch=${field["$k.fetch"]-}
  [[ -n $fetch ]] || token_exit 2 unknown-key no 'name the bad line or key'
  page=${field["$k.page"]-}
  [[ $fetch == auto ]] || token_exit 3 manual no "download it from $page (packaging/README.md)"
  sha=${field["$k.sha256"]}; url=${field["$k.url"]}; size=${field["$k.size"]}
  field_size=$size
  [[ $url != TBD-at-pin ]] || token_exit 3 url-tbd no "set $k.url from $k.page, then rerun"
  if [[ $sha =~ ^[a-fA-F0-9]{64}$ && $check == 0 ]]; then token_exit 5 already-pinned no "packaging/pin.sh $k --check"; fi
  if [[ $url != https://* && ! ( ${PIN_ALLOW_FILE:-0} == 1 && $url == file://* ) ]]; then token_exit 3 scheme no 'use an https url'; fi
  base=${XDG_CACHE_HOME:-${HOME:?HOME is required}/.cache}/flat3d
  mkdir -p "$base" || token_exit 3 network yes "rerun packaging/pin.sh $k"
  tmp=$(mktemp -d "$base/pin.XXXXXX") || token_exit 3 network yes "rerun packaging/pin.sh $k"
  pin_cleanup_tmp=$tmp
  trap 'rm -rf "${pin_cleanup_tmp:-}"' EXIT
  if [[ -n $file ]]; then cp -- "$file" "$tmp/f" || token_exit 3 network yes "rerun packaging/pin.sh $k"; else
    if [[ $size == TBD-at-pin ]]; then cap=4294967296; else cap=$((size + 1)); fi
    proto='=https'; [[ ${PIN_ALLOW_FILE:-0} == 1 ]] && proto='=https,file'
    curl -fL --proto "$proto" --proto-redir "$proto" --tlsv1.2 --retry 3 --max-filesize "$cap" -o "$tmp/f" "$url" || token_exit 3 network yes "rerun packaging/pin.sh $k"
  fi
  checks=; actual_size=$(stat -c %s "$tmp/f") || token_exit 3 network yes "rerun packaging/pin.sh $k"
  if [[ $size != TBD-at-pin ]]; then
    [[ $actual_size == "$size" ]] || token_exit 4 size-mismatch no "do not pin; check $k.url against $k.page"
    checks=size
  fi
  if [[ -n ${field["$k.md5"]-} ]]; then
    got=$(md5sum "$tmp/f" | cut -d' ' -f1)
    [[ $got == "${field["$k.md5"]}" ]] || token_exit 4 md5-mismatch no "do not pin; check $k.url against $k.page"
    checks=${checks:+$checks+}md5
  fi
  actual=$(sha256sum "$tmp/f" | cut -d' ' -f1)
  if [[ $sha != TBD-at-pin && $actual != "$sha" ]]; then token_exit 4 sha-mismatch no "do not pin; check $k.url against $k.page"; fi
  route=${url#https://github.com/}
  if [[ $route != "$url" && $(command -v gh || true) ]]; then
    repo=${route%%/*}; route=${route#*/}; repo=$repo/${route%%/*}; route=${route#*/}
    if [[ $route == releases/download/* ]]; then
      route=${route#releases/download/}; tag=${route%%/*}; name=${route#*/}
      got_digest=$(gh api "repos/$repo/releases/tags/$tag" --jq ".assets[]|select(.name==\"$name\")|.digest") || token_exit 3 gh-failed yes "rerun packaging/pin.sh $k (check gh auth status first)"
      if [[ -n $got_digest && $got_digest != null ]]; then
        [[ $got_digest == "sha256:$actual" ]] || token_exit 4 digest-mismatch no "do not pin; check $k.url against $k.page"
        checks=${checks:+$checks+}gh-digest
      fi
    fi
  fi
  [[ -n $checks ]] || checks=sha256
  stamp=$(date +%F)
  newsha=$actual; newsize=$actual_size
  if ((check)); then printf 'pin: checked retryable=no fix: none\n'; return 0; fi
  if ((dry)); then
    printf '%s.sha256: %s\n' "$k" "$newsha"
    [[ $size != TBD-at-pin ]] || printf '%s.size: %s\n' "$k" "$newsize"
    printf '%s.pinned: %s tofu %s\n' "$k" "$stamp" "$checks"
    printf 'pin: dry-run retryable=no fix: none\n'; return 0
  fi
  out=$(mktemp "$(dirname "$pins")/.pins.XXXXXX") || token_exit 3 network yes "rerun packaging/pin.sh $k"
  fill_size=0
  if [[ $field_size == TBD-at-pin ]]; then fill_size=1; fi
  awk -v k="$k" -v sha="$newsha" -v size="$newsize" -v fill_size="$fill_size" -v pinned="$stamp tofu $checks" '
    BEGIN { sha_seen=0; size_seen=0; pin_seen=0 }
    index($0,k ".sha256: ")==1 { print k ".sha256: " sha; sha_seen=1; next }
    index($0,k ".size: ")==1 && fill_size { print k ".size: " size; size_seen=1; next }
    index($0,k ".pinned: ")==1 { print k ".pinned: " pinned; pin_seen=1; next }
    { print }
    END { if (!sha_seen) print k ".sha256: " sha; if (fill_size && !size_seen) print k ".size: " size; if (!pin_seen) print k ".pinned: " pinned }
  ' "$pins" > "$out" || { rm -f "$out"; token_exit 3 network yes "rerun packaging/pin.sh $k"; }
  mv "$out" "$pins" || { rm -f "$out"; token_exit 3 network yes "rerun packaging/pin.sh $k"; }
  printf 'pin: ok retryable=no fix: none\nreview: git diff --stat -- packaging/pins\n'
}

if ((all)); then
  code=0
  for k in "${keys[@]}"; do
    [[ ${field["$k.fetch"]-} == auto && ${field["$k.sha256"]-} == TBD-at-pin ]] || continue
    if (pin_one "$k"); then :; else s=$?; ((s > code)) && code=$s; fi
  done
  exit "$code"
fi
[[ -n $key ]] || { usage; token_exit 2 usage no 'name the bad line or key'; }
pin_one "$key"
