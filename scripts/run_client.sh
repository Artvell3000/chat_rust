#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: $0 <ADDR:PORT> <USERNAME>"
  exit 1
fi

addr="$1"
name="$2"
shift 2

cargo run -- client --addr "$addr" --name "$name" "$@"
