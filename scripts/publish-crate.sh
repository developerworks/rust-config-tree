#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY_RUN=1

usage() {
  cat <<'EOF'
Usage:
  scripts/publish-crate.sh [--execute]

Runs the crate release checks and performs a cargo publish dry-run by default.
Pass --execute to publish to crates.io after the checks pass.
EOF
}

while (($# > 0)); do
  case "$1" in
    --execute)
      DRY_RUN=0
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac

  shift
done

cd "${ROOT_DIR}"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "working tree must be clean before publishing" >&2
  git status --short >&2
  exit 1
fi

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo package --list >/dev/null

if ((DRY_RUN)); then
  cargo publish --dry-run
  echo "crate publish dry-run passed"
else
  cargo publish
fi
