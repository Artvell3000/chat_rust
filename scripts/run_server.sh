#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <BIND_ADDR:PORT>"
  exit 1
fi

bind_addr="$1"
shift

cargo run -- server --bind "$bind_addr" "$@"
