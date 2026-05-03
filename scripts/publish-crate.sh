#!/usr/bin/env bash
set -euo pipefail

# Prepares or publishes the crate. The prepare step is intentionally reusable by
# `scripts/release.sh`, so version auto-bumping can happen before the final
# workspace checks and commit.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY_RUN=1
ALLOW_DIRTY=0
AUTO_BUMP=1
BUMP_KIND="patch"
PREPARE_ONLY=0
BUMPED=0
PUBLISH_ATTEMPTS="${PUBLISH_ATTEMPTS:-5}"
PUBLISH_RETRY_DELAY="${PUBLISH_RETRY_DELAY:-10}"

export CARGO_NET_RETRY="${CARGO_NET_RETRY:-10}"
export CARGO_HTTP_TIMEOUT="${CARGO_HTTP_TIMEOUT:-60}"

# Prints CLI usage and release-related environment variables.
usage() {
  cat <<'EOF'
Usage:
  scripts/publish-crate.sh [--execute] [--allow-dirty] [--bump patch|minor|major] [--no-bump]

Runs the crate release checks and performs a cargo publish dry-run by default.
If package.version already exists on crates.io, the script bumps it by patch
by default. Pass --execute to publish to crates.io after the checks pass.

Network-sensitive publish steps are retried. Override retry behavior with:
  PUBLISH_ATTEMPTS=5
  PUBLISH_RETRY_DELAY=10
EOF
}

# Reads the crate package name from Cargo.toml.
crate_name() {
  sed -n 's/^name = "\(.*\)"/\1/p' Cargo.toml | head -n 1
}

# Reads the crate package version from Cargo.toml.
crate_version() {
  sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1
}

# Runs a command with exponential backoff for transient network failures.
run_with_retry() {
  local attempts="$1"
  local delay="$2"
  local attempt=1
  local status=0

  shift 2

  while true; do
    if "$@"; then
      return 0
    fi

    status=$?
    if ((attempt >= attempts)); then
      return "${status}"
    fi

    echo "command failed with status ${status}; retrying in ${delay}s (${attempt}/${attempts})" >&2
    sleep "${delay}"
    attempt=$((attempt + 1))
    delay=$((delay * 2))
  done
}

# Fetches only the HTTP status code for a URL with retry handling.
curl_status_with_retry() {
  local url="$1"

  run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
    curl \
      --silent \
      --show-error \
      --output /dev/null \
      --write-out "%{http_code}" \
      --user-agent "rust-config-tree-release-script" \
      --connect-timeout 20 \
      --max-time 60 \
      "${url}"
}

# Checks whether a crate version already exists on crates.io.
crate_version_exists() {
  local name="$1"
  local version="$2"
  local status

  command -v curl >/dev/null 2>&1 || {
    echo "curl is required to check crates.io before publishing" >&2
    exit 1
  }

  status="$(curl_status_with_retry "https://crates.io/api/v1/crates/${name}/${version}")"

  case "${status}" in
    200)
      return 0
      ;;
    404)
      return 1
      ;;
    *)
      echo "failed to check crates.io for ${name}@${version}; HTTP ${status}" >&2
      exit 1
      ;;
  esac
}

# Bumps a semantic x.y.z version according to the requested release kind.
bump_version() {
  local version="$1"
  local kind="$2"
  local major minor patch

  IFS='.' read -r major minor patch <<<"${version}"

  if [[ ! "${major}" =~ ^[0-9]+$ || ! "${minor}" =~ ^[0-9]+$ || ! "${patch}" =~ ^[0-9]+$ ]]; then
    echo "cannot auto bump non-x.y.z version: ${version}" >&2
    exit 1
  fi

  case "${kind}" in
    patch)
      patch=$((patch + 1))
      ;;
    minor)
      minor=$((minor + 1))
      patch=0
      ;;
    major)
      major=$((major + 1))
      minor=0
      patch=0
      ;;
    *)
      echo "invalid bump kind: ${kind}" >&2
      exit 2
      ;;
  esac

  echo "${major}.${minor}.${patch}"
}

# Rewrites package.version in Cargo.toml and verifies the result.
set_crate_version() {
  local old_version="$1"
  local new_version="$2"

  OLD_VERSION="${old_version}" NEW_VERSION="${new_version}" \
    perl -0pi -e 's/^version = "\Q$ENV{OLD_VERSION}\E"/version = "$ENV{NEW_VERSION}"/m' Cargo.toml

  if [[ "$(crate_version)" != "${new_version}" ]]; then
    echo "failed to update package.version to ${new_version}" >&2
    exit 1
  fi
}

# Auto-bumps package.version until it is not present on crates.io.
ensure_unpublished_version() {
  local name="$1"
  local version="$2"
  local next_version

  while crate_version_exists "${name}" "${version}"; do
    if (( ! AUTO_BUMP )); then
      echo "crate ${name}@${version} already exists on crates.io" >&2
      echo "bump package.version in Cargo.toml before publishing" >&2
      exit 1
    fi

    next_version="$(bump_version "${version}" "${BUMP_KIND}")"
    echo "crate ${name}@${version} already exists on crates.io; bumping to ${next_version}"
    set_crate_version "${version}" "${next_version}"
    version="${next_version}"
    BUMPED=1
  done
}

while (($# > 0)); do
  case "$1" in
    --execute)
      DRY_RUN=0
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    --allow-dirty)
      ALLOW_DIRTY=1
      ;;
    --bump)
      shift
      if (($# == 0)); then
        echo "--bump requires patch, minor, or major" >&2
        exit 2
      fi
      BUMP_KIND="$1"
      AUTO_BUMP=1
      ;;
    --no-bump)
      AUTO_BUMP=0
      ;;
    --prepare-only)
      PREPARE_ONLY=1
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

if (( ! ALLOW_DIRTY )) && [[ -n "$(git status --porcelain)" ]]; then
  echo "working tree must be clean before publishing" >&2
  git status --short >&2
  exit 1
fi

NAME="$(crate_name)"
VERSION="$(crate_version)"

ensure_unpublished_version "${NAME}" "${VERSION}"

if ((PREPARE_ONLY)); then
  echo "crate version is ready: ${NAME}@$(crate_version)"
  exit 0
fi

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

if ((ALLOW_DIRTY || BUMPED)); then
  cargo package --list --allow-dirty >/dev/null
else
  cargo package --list >/dev/null
fi

if ((DRY_RUN)); then
  if ((ALLOW_DIRTY || BUMPED)); then
    run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
      cargo publish --dry-run --allow-dirty
  else
    run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
      cargo publish --dry-run
  fi
  echo "crate publish dry-run passed"
else
  if ((BUMPED)); then
    run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
      cargo publish --allow-dirty
  else
    run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
      cargo publish
  fi
fi
