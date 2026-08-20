#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

args=(run --quiet -p xtask -- check-repo)

while (($# > 0)); do
  case "$1" in
    --skip-clippy|--skip-text-files)
      args+=("$1")
      shift
      ;;
    --base-ref)
      if (($# < 2)); then
        echo "--base-ref requires a value" >&2
        exit 2
      fi
      args+=("$1" "$2")
      shift 2
      ;;
    *)
      echo "unsupported argument: $1" >&2
      exit 2
      ;;
  esac
done

cd "$repo_root"
echo "==> cargo ${args[*]}"
cargo "${args[@]}"
