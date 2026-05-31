#!/usr/bin/env bash
set -euo pipefail

# Prepares or publishes the crate. The prepare step is intentionally reusable by
# `scripts/release.sh`, so version auto-bumping can happen before the final
# workspace checks and commit.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_MANIFEST="${ROOT_DIR}/Cargo.toml"
MACROS_MANIFEST="${ROOT_DIR}/macros/Cargo.toml"
DRY_RUN=1
ALLOW_DIRTY=0
AUTO_BUMP=1
BUMP_KIND="patch"
PREPARE_ONLY=0
BUMPED=0
PUBLISH_ATTEMPTS="${PUBLISH_ATTEMPTS:-5}"
PUBLISH_RETRY_DELAY="${PUBLISH_RETRY_DELAY:-10}"
MACROS_PUBLISHED=0

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

When the procedural macro crate is present, it is checked and published before
the main crate because crates.io must resolve published dependencies from the
registry. A main-crate dry run is skipped until the macro crate version is
visible on crates.io.

Network-sensitive publish steps are retried. Override retry behavior with:
  PUBLISH_ATTEMPTS=5
  PUBLISH_RETRY_DELAY=10
EOF
}

# Reads the crate package name from a manifest.
crate_name() {
  local manifest="$1"
  sed -n 's/^name = "\(.*\)"/\1/p' "${manifest}" | head -n 1
}

# Reads the crate package version from a manifest.
crate_version() {
  local manifest="$1"
  sed -n 's/^version = "\(.*\)"/\1/p' "${manifest}" | head -n 1
}

# Reads the root dependency version for the macro crate.
macro_dependency_version() {
  sed -n 's/^rust-config-tree-macros = { version = "\([^"]*\)".*/\1/p' "${ROOT_MANIFEST}" | head -n 1
}

# Runs a command with exponential backoff for transient network failures.
run_with_retry() {
  local attempts="$1"
  local delay="$2"
  local attempt=1
  local status=0

  shift 2

  while true; do
    set +e
    "$@"
    status=$?
    set -e

    if ((status == 0)); then
      return 0
    fi

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

# Rewrites the root crate, macro crate, and dependency versions together.
set_release_version() {
  local old_version="$1"
  local new_version="$2"

  OLD_VERSION="${old_version}" NEW_VERSION="${new_version}" \
    perl -0pi -e 's/^version = "\Q$ENV{OLD_VERSION}\E"/version = "$ENV{NEW_VERSION}"/m;
      s/(rust-config-tree-macros = \{ version = ")\Q$ENV{OLD_VERSION}\E(")/$1$ENV{NEW_VERSION}$2/m' "${ROOT_MANIFEST}"

  if [[ -f "${MACROS_MANIFEST}" ]]; then
    OLD_VERSION="${old_version}" NEW_VERSION="${new_version}" \
      perl -0pi -e 's/^version = "\Q$ENV{OLD_VERSION}\E"/version = "$ENV{NEW_VERSION}"/m' "${MACROS_MANIFEST}"
  fi

  if [[ "$(crate_version "${ROOT_MANIFEST}")" != "${new_version}" ]]; then
    echo "failed to update root package.version to ${new_version}" >&2
    exit 1
  fi

  if [[ -f "${MACROS_MANIFEST}" && "$(crate_version "${MACROS_MANIFEST}")" != "${new_version}" ]]; then
    echo "failed to update macro package.version to ${new_version}" >&2
    exit 1
  fi

  if [[ -f "${MACROS_MANIFEST}" && "$(macro_dependency_version)" != "${new_version}" ]]; then
    echo "failed to update rust-config-tree-macros dependency to ${new_version}" >&2
    exit 1
  fi
}

# Auto-bumps the release version until the main crate version is unpublished.
ensure_release_version() {
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
    set_release_version "${version}" "${next_version}"
    version="${next_version}"
    BUMPED=1
  done
}

# Returns whether the macro crate manifest exists.
has_macros_crate() {
  [[ -f "${MACROS_MANIFEST}" ]]
}

# Returns cargo's dirty-working-tree publish/package flag when needed.
dirty_flag() {
  if ((ALLOW_DIRTY || BUMPED)); then
    echo "--allow-dirty"
  fi
}

# Waits until a just-published crate version is visible in the registry.
wait_for_crate_version() {
  local name="$1"
  local version="$2"
  local attempt=1

  while true; do
    if crate_version_exists "${name}" "${version}"; then
      return 0
    fi

    if ((attempt >= PUBLISH_ATTEMPTS)); then
      echo "crate ${name}@${version} is not visible on crates.io yet" >&2
      exit 1
    fi

    echo "waiting for ${name}@${version} to become visible on crates.io (${attempt}/${PUBLISH_ATTEMPTS})"
    sleep "${PUBLISH_RETRY_DELAY}"
    attempt=$((attempt + 1))
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

NAME="$(crate_name "${ROOT_MANIFEST}")"
VERSION="$(crate_version "${ROOT_MANIFEST}")"

ensure_release_version "${NAME}" "${VERSION}"
VERSION="$(crate_version "${ROOT_MANIFEST}")"
MACROS_NAME=""
MACROS_VERSION=""

if has_macros_crate; then
  MACROS_NAME="$(crate_name "${MACROS_MANIFEST}")"
  MACROS_VERSION="$(crate_version "${MACROS_MANIFEST}")"

  if [[ "${MACROS_VERSION}" != "${VERSION}" ]]; then
    echo "macro crate version ${MACROS_VERSION} must match root version ${VERSION}" >&2
    exit 1
  fi

  if [[ "$(macro_dependency_version)" != "${MACROS_VERSION}" ]]; then
    echo "root dependency on ${MACROS_NAME} must use version ${MACROS_VERSION}" >&2
    exit 1
  fi

  if crate_version_exists "${MACROS_NAME}" "${MACROS_VERSION}"; then
    MACROS_PUBLISHED=1
  fi
fi

if ((PREPARE_ONLY)); then
  echo "crate version is ready: ${NAME}@$(crate_version "${ROOT_MANIFEST}")"
  if has_macros_crate; then
    echo "macro crate version is ready: ${MACROS_NAME}@${MACROS_VERSION}"
  fi
  exit 0
fi

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

if has_macros_crate; then
  cargo package --list --manifest-path "${MACROS_MANIFEST}" $(dirty_flag) >/dev/null
fi

if ! has_macros_crate || ((MACROS_PUBLISHED)); then
  cargo package --list $(dirty_flag) >/dev/null
fi

if ((DRY_RUN)); then
  if has_macros_crate; then
    run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
      cargo publish --manifest-path "${MACROS_MANIFEST}" --dry-run $(dirty_flag)
  fi

  if ! has_macros_crate || ((MACROS_PUBLISHED)); then
    run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
      cargo publish --dry-run $(dirty_flag)
    echo "crate publish dry-run passed"
  else
    echo "macro crate publish dry-run passed"
    echo "main crate dry-run skipped because ${MACROS_NAME}@${MACROS_VERSION} is not visible on crates.io"
    echo "run scripts/publish-crate.sh --execute to publish the macro crate first, then the main crate"
  fi
else
  if has_macros_crate && (( ! MACROS_PUBLISHED )); then
    run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
      cargo publish --manifest-path "${MACROS_MANIFEST}" $(dirty_flag)
    wait_for_crate_version "${MACROS_NAME}" "${MACROS_VERSION}"
  fi

  cargo package --list $(dirty_flag) >/dev/null
  run_with_retry "${PUBLISH_ATTEMPTS}" "${PUBLISH_RETRY_DELAY}" \
    cargo publish $(dirty_flag)
fi
