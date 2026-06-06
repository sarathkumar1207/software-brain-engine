#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: scripts/validate-benchmark.sh <project-path> [query]" >&2
  exit 2
fi

PROJECT_PATH="$1"
QUERY="${2:-jwt to passport}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$REPO_ROOT/target/release/sbe"

cd "$REPO_ROOT"
cargo build --release -p sbe-cli

"$BINARY" validate "$PROJECT_PATH" --query "$QUERY"
"$BINARY" benchmark "$PROJECT_PATH" --query "$QUERY" --json
