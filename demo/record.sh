#!/usr/bin/env bash
# Re-record demo GIFs from scripted scenarios
# Prerequisites: asciinema, agg (https://github.com/asciinema/agg)
#   brew install asciinema
#   cargo install --git https://github.com/asciinema/agg
#
# Usage:
#   bash demo/record.sh          # record all
#   bash demo/record.sh usage    # only usage demo

set -euo pipefail

DEMO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

record_usage() {
  echo "==> Recording usage demo..."
  asciinema rec "$DEMO_DIR/usage.cast" \
    --command "bash $DEMO_DIR/usage.sh" \
    --overwrite
  echo "==> Rendering usage.gif..."
  agg "$DEMO_DIR/usage.cast" "$DEMO_DIR/usage.gif" \
    --font-size 16 --cols 100 --rows 40
  echo "    done: demo/usage.gif"
}

case "${1:-all}" in
  usage) record_usage ;;
  all)   record_usage ;;
  *)
    echo "Usage: $0 [usage|all]" >&2
    exit 1
    ;;
esac

echo ""
echo "✓ Done. Commit demo/usage.gif to the repo."
