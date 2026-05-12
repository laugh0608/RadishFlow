#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
fail_on_exceeded=false

for arg in "$@"; do
  case "$arg" in
    --fail-on-exceeded)
      fail_on_exceeded=true
      ;;
    *)
      echo "unsupported argument: $arg" >&2
      exit 2
      ;;
  esac
done

rule_for_path() {
  local path="$1"

  case "$path" in
    AGENTS.md|CLAUDE.md)
      echo "14000 entry enforced"
      ;;
    docs/status/current.md)
      echo "8000 status enforced"
      ;;
    docs/README.md)
      echo "10000 docs-index enforced"
      ;;
    docs/adr/*.md)
      echo "12000 adr enforced"
      ;;
    docs/guides/*.md|docs/capeopen/pme-validation.md)
      echo "15000 guide enforced"
      ;;
    docs/reference/*.md)
      echo "25000 reference enforced"
      ;;
    docs/architecture/*.md|docs/capeopen/boundary.md|docs/thermo/*.md|docs/mvp/*.md|docs/radishflow-mvp-roadmap.md)
      echo "30000 topic enforced"
      ;;
    docs/devlogs/*.md|docs/*draft*.md|docs/*checklist*.md)
      echo "30000 history advisory"
      ;;
    *)
      echo "25000 other enforced"
      ;;
  esac
}

cd "$repo_root"

mapfile -t files < <(
  {
    for path in AGENTS.md CLAUDE.md README.md; do
      [[ -f "$path" ]] && printf '%s\n' "$path"
    done
    find docs -type f -name '*.md' | sort
  } | sort -u
)

over_limit_rows=()
enforced_count=0

for path in "${files[@]}"; do
  read -r limit scope mode < <(rule_for_path "$path")
  chars="$(wc -m < "$path" | tr -d '[:space:]')"

  if (( chars > limit )); then
    over_limit_rows+=("$path	$chars	$limit	$scope	$mode")
    if [[ "$mode" == "enforced" ]]; then
      enforced_count=$((enforced_count + 1))
    fi
  fi
done

if (( ${#over_limit_rows[@]} == 0 )); then
  echo "doc size check: all markdown files are within target limits"
else
  echo "doc size check: files over target limits"
  printf 'path\tchars\tlimit\tscope\tmode\n'
  printf '%s\n' "${over_limit_rows[@]}"
fi

if [[ "$fail_on_exceeded" == "true" && "$enforced_count" -gt 0 ]]; then
  echo "doc size check failed: ${enforced_count} enforced file(s) exceed target limits" >&2
  exit 1
fi
