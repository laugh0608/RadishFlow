#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

args=(run --quiet -p xtask -- check-repo)

for arg in "$@"; do
  case "$arg" in
    --skip-clippy|--skip-text-files)
      args+=("$arg")
      ;;
    *)
      echo "unsupported argument: $arg" >&2
      exit 2
      ;;
  esac
done

cd "$repo_root"
echo "==> cargo ${args[*]}"
cargo "${args[@]}"
