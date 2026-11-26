#!/usr/bin/env bash
set -euo pipefail

readonly version=1.0.0
readonly file_limit=5242880
readonly batch_limit=10485760

usage() {
  printf '%s\n' 'Usage: check.sh --staged | --pre-push | --range <base> <head> | --version' >&2
}

fail() {
  printf '[repository assets] %s\n' "$*" >&2
  exit 2
}

if [[ ${1-} == --version && $# == 1 ]]; then
  printf '%s\n' "$version"
  exit
fi
case "${1-}:$#" in
  --staged:1|--pre-push:1|--range:3) ;;
  *) usage; exit 2 ;;
esac

cd "$(git rev-parse --show-toplevel)"
policy=.radish-assets/policy.tsv
[[ -f "$policy" ]] || fail "Missing $policy"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/radish-asset-check.XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
: > "$scratch/binaries"
: > "$scratch/exceptions"
: > "$scratch/paths"
baseline=
policy_version=
policy_input=$policy

if [[ "$1" == --staged ]]; then
  if git cat-file -e ":$policy" 2>/dev/null; then
    git show ":$policy" > "$scratch/policy"
    policy_input=$scratch/policy
  elif git cat-file -e "HEAD:$policy" 2>/dev/null; then
    fail "The candidate commit removes $policy"
  fi
fi

while IFS=$'\t' read -r kind first second third fourth rest || [[ -n "$kind" ]]; do
  case "$kind" in
    ''|'#'*) ;;
    version)
      [[ -z "$policy_version" && -n "$first" && -z "$second" ]] || fail "Invalid version row in $policy"
      policy_version=$first
      ;;
    baseline)
      [[ -z "$baseline" && "$first" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ && -z "$second" ]] || fail "Invalid baseline row in $policy"
      baseline=$first
      ;;
    deny)
      [[ -n "$first" && -z "$second" ]] || fail "Invalid deny row in $policy"
      printf '%s\n' "$first" >> "$scratch/paths"
      ;;
    allow)
      [[ -n "$first" && "$second" =~ ^[1-9][0-9]{0,9}$ && "$third" =~ ^(-|[0-9a-f]{64})$ && -n "$fourth" && -n "$rest" ]] ||
        fail "Invalid allow row in $policy"
      printf '%s\t%s\t%s\n' "$first" "$second" "$third" >> "$scratch/exceptions"
      ;;
    *) fail "Unknown '$(printf '%q' "$kind")' row in $policy" ;;
  esac
done < "$policy_input"
[[ "$policy_version" == "$version" && -n "$baseline" ]] || fail "Expected policy version $version and an adoption baseline"
if ! cut -f1 "$scratch/exceptions" | LC_ALL=C sort | uniq -d > "$scratch/duplicates"; then
  fail "Could not read exception paths"
fi
[[ ! -s "$scratch/duplicates" ]] || fail "Each path may have only one exception"

shopt -s nocasematch
errors=0

is_denied() {
  local pattern
  case "$1" in
    private-resources/*|output/playwright/*|playwright-report/*|*/playwright-report/*|test-results/*|*/test-results/*|\
    node_modules/*|*/node_modules/*|target/*|dist/*|build/*|.cache/*|\
    *.chm|*.zip|*.7z|*.rar|*.tar|*.tgz|*.tbz|*.tbz2|*.gz|*.bz2|*.xz|*.zst|\
    *.apk|*.aab|*.ipa|*.dmg|*.msi|*.msix|*.appx|*.deb|*.rpm)
      return 0 ;;
  esac
  while IFS= read -r pattern; do
    [[ "$1" == $pattern ]] && return 0
  done < "$scratch/paths"
  return 1
}

is_binary() {
  [[ "$2" == - ]] && return 0
  case "$1" in
    *.png|*.jpg|*.jpeg|*.gif|*.webp|*.avif|*.bmp|*.ico|*.icns|*.tif|*.tiff|*.psd|*.exr|\
    *.ttf|*.otf|*.woff|*.woff2|*.eot|*.pdf|*.penpot|*.fig|*.pptx|*.docx|*.xlsx|\
    *.glb|*.blend|*.fbx|*.mp4|*.mov|*.webm|*.wav|*.mp3|*.ogg|*.flac|\
    *.db|*.sqlite|*.sqlite3|*.dll|*.so|*.dylib|*.exe|*.wasm|*.pdb|*.jar) return 0 ;;
  esac
  return 1
}

blob_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    git cat-file blob "$1" | sha256sum | cut -d ' ' -f 1
  elif command -v shasum >/dev/null 2>&1; then
    git cat-file blob "$1" | shasum -a 256 | cut -d ' ' -f 1
  else
    fail 'SHA-256 verification requires sha256sum or shasum'
  fi
}

has_exception() {
  local path=$1 size=$2 oid=$3 allowed cap hash actual
  while IFS=$'\t' read -r allowed cap hash; do
    [ "$path" = "$allowed" ] || continue
    (( size <= cap )) || return 1
    if [[ "$hash" != - ]]; then
      actual=$(blob_sha256 "$oid")
      [ "$actual" = "$hash" ] || return 1
    fi
    return 0
  done < "$scratch/exceptions"
  return 1
}

scan_diff() {
  local tree=$1 previous=$2 record path fields oid previous_oid info size binary denied
  while IFS= read -r -d '' record; do
    fields=${record#*$'\t'}
    path=${fields#*$'\t'}
    binary=0
    denied=0
    is_binary "$path" "${record%%$'\t'*}" && binary=1
    is_denied "$path" && denied=1
    (( binary || denied )) || continue

    oid=$(git rev-parse --verify --quiet --end-of-options "$tree:$path") ||
      fail "Could not resolve $(printf '%q' "$path")"
    if [[ -n "$previous" ]] && previous_oid=$(git rev-parse --verify --quiet --end-of-options "$previous:$path"); then
      [[ "$oid" != "$previous_oid" ]] || continue
    fi
    info=$(printf '%s\n' "$oid" | git cat-file --batch-check='%(objecttype) %(objectsize)') ||
      fail "Could not inspect $(printf '%q' "$path")"
    case "$info" in
      blob\ *) size=${info#blob } ;;
      # Gitlinks do not store a binary blob in this repository.
      *)
        if (( denied )); then
          printf '[repository assets] Blocked path %q\n' "$path" >&2
          errors=$((errors + 1))
        fi
        continue ;;
    esac
    (( binary )) && printf '%s %s\n' "$oid" "$size" >> "$scratch/binaries"
    if (( denied || (binary && size > file_limit) )); then
      if ! has_exception "$path" "$size" "$oid"; then
        if (( denied )); then
          printf '[repository assets] Blocked path or artifact type %q (%s bytes)\n' "$path" "$size" >&2
        else
          printf '[repository assets] Binary needs an exception %q (%s bytes; limit %s)\n' "$path" "$size" "$file_limit" >&2
        fi
        errors=$((errors + 1))
      fi
    fi
  done < "$scratch/diff"
}

scan_range() {
  local base=$1 head=$2 commit parent
  local -a revisions diff_args
  head=$(git rev-parse --verify --quiet --end-of-options "$head^{commit}") || fail 'Missing head commit'
  revisions=("$head")
  if [[ -n "$base" ]]; then
    base=$(git rev-parse --verify --quiet --end-of-options "$base^{commit}") || fail 'Missing base commit; fetch the history first'
    revisions+=("^$base")
  fi
  git cat-file -e "$baseline^{commit}" 2>/dev/null || fail 'Missing adoption baseline; fetch the history first'
  revisions+=("^$baseline")
  git rev-list --topo-order --reverse "${revisions[@]}" -- > "$scratch/commits"
  diff_args=(--numstat -z --no-renames --no-ext-diff --no-textconv --diff-filter=AMT)
  while IFS= read -r commit; do
    if parent=$(git rev-parse --verify --quiet "$commit^1"); then
      git diff "${diff_args[@]}" "$parent" "$commit" -- > "$scratch/diff"
    else
      git diff-tree --root --no-commit-id -r "${diff_args[@]}" "$commit" -- > "$scratch/diff"
    fi
    scan_diff "$commit" "$parent"
  done < "$scratch/commits"
}

case "$1" in
  --staged)
    git diff --cached --numstat -z --no-renames --no-ext-diff --no-textconv --diff-filter=AMT -- > "$scratch/diff"
    scan_diff '' HEAD
    ;;
  --range)
    base=$2
    [[ "$base" =~ ^0+$ ]] && base=
    scan_range "$base" "$3"
    ;;
  --pre-push)
    while read -r local_ref local_oid remote_ref remote_oid; do
      [[ "$local_oid" =~ ^0+$ ]] && continue
      [[ "$remote_oid" =~ ^0+$ ]] && remote_oid=
      scan_range "$remote_oid" "$local_oid"
    done
    ;;
esac

total=$(LC_ALL=C sort -u "$scratch/binaries" | awk '{ sum += $2 } END { printf "%.0f\n", sum }')
if (( total > batch_limit )); then
  printf '[repository assets] Notice: unique changed binary content totals %s bytes (10 MiB review threshold).\n' "$total" >&2
fi
if (( errors > 0 )); then
  printf '[repository assets] See .radish-assets/policy.tsv for repository exceptions.\n' >&2
  exit 1
fi
