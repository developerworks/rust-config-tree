#!/usr/bin/env bash
set -euo pipefail

# Runs the same local gates used before publishing, then optionally commits,
# pushes, waits for the Pages workflow, and publishes the already prepared
# crate version.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY_RUN=1
WAIT_PAGES=1
COMMIT_MESSAGE=""

# Prints release script usage.
usage() {
  cat <<'EOF'
Usage:
  scripts/release.sh [--execute] [--message <message>] [--no-wait-pages]

Runs the full release flow:
  1. build the mdBook Pages artifact
  2. run Rust checks
  3. commit and push code
  4. wait for the GitHub Pages workflow
  5. publish the crate

Default mode is a dry run. Pass --execute to commit, push, wait for Pages, and
publish to crates.io.
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
    --message|-m)
      shift
      if (($# == 0)); then
        echo "--message requires a value" >&2
        exit 2
      fi
      COMMIT_MESSAGE="$1"
      ;;
    --no-wait-pages)
      WAIT_PAGES=0
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

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
BRANCH="$(git branch --show-current)"

if [[ -z "${COMMIT_MESSAGE}" ]]; then
  COMMIT_MESSAGE="Release ${VERSION}"
fi

if [[ "${BRANCH}" != "main" ]]; then
  echo "release must run from main; current branch is ${BRANCH}" >&2
  exit 1
fi

scripts/publish-pages.sh
scripts/publish-crate.sh --prepare-only --allow-dirty
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check

if ((DRY_RUN)); then
  scripts/publish-crate.sh --dry-run --allow-dirty --no-bump
  echo "release dry-run passed"
  echo "no commit, push, Pages deploy, or crate publish was performed"
  exit 0
fi

if [[ -n "$(git status --porcelain)" ]]; then
  git add -A
  git commit -m "${COMMIT_MESSAGE}"
else
  echo "working tree is clean; no commit created"
fi

git push

if ((WAIT_PAGES)); then
  if command -v gh >/dev/null 2>&1; then
    sleep 5
    RUN_ID="$(
      gh run list \
        --workflow "Publish mdBook" \
        --branch "${BRANCH}" \
        --limit 1 \
        --json databaseId \
        --jq '.[0].databaseId'
    )"

    if [[ -n "${RUN_ID}" && "${RUN_ID}" != "null" ]]; then
      gh run watch "${RUN_ID}" --exit-status
    else
      echo "could not find the GitHub Pages workflow run" >&2
      exit 1
    fi
  else
    echo "gh is not installed; skipped GitHub Pages workflow wait"
  fi
fi

scripts/publish-crate.sh --execute --no-bump
