#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

cd "$repo_root"
echo "==> cargo run --quiet -p xtask -- check-text-files"
cargo run --quiet -p xtask -- check-text-files
