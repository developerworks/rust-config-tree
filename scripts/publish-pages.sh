#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT_DIR}/target/mdbook"

command -v mdbook >/dev/null 2>&1 || {
  echo "mdbook is required" >&2
  exit 1
}

rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}"

mdbook build -d "${OUT_DIR}/en" "${ROOT_DIR}/manual/en"
mdbook build -d "${OUT_DIR}/zh" "${ROOT_DIR}/manual/zh"
mdbook build -d "${OUT_DIR}/ja" "${ROOT_DIR}/manual/ja"
mdbook build -d "${OUT_DIR}/ko" "${ROOT_DIR}/manual/ko"
mdbook build -d "${OUT_DIR}/fr" "${ROOT_DIR}/manual/fr"
mdbook build -d "${OUT_DIR}/de" "${ROOT_DIR}/manual/de"
mdbook build -d "${OUT_DIR}/es" "${ROOT_DIR}/manual/es"
mdbook build -d "${OUT_DIR}/pt" "${ROOT_DIR}/manual/pt"
mdbook build -d "${OUT_DIR}/sv" "${ROOT_DIR}/manual/sv"
mdbook build -d "${OUT_DIR}/fi" "${ROOT_DIR}/manual/fi"
mdbook build -d "${OUT_DIR}/nl" "${ROOT_DIR}/manual/nl"

cat >"${OUT_DIR}/index.html" <<'HTML'
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>rust-config-tree Manual</title>
    <style>
      :root {
        color-scheme: light dark;
        font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }

      body {
        margin: 0;
        min-height: 100vh;
        display: grid;
        place-items: center;
        padding: 2rem;
      }

      main {
        max-width: 42rem;
      }

      h1 {
        margin: 0 0 0.75rem;
        font-size: 2rem;
      }

      p {
        margin: 0 0 1.5rem;
        line-height: 1.6;
      }

      nav {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem;
      }

      a {
        display: inline-flex;
        align-items: center;
        min-height: 2.5rem;
        padding: 0 1rem;
        border: 1px solid currentColor;
        border-radius: 0.375rem;
        color: inherit;
        text-decoration: none;
      }
    </style>
  </head>
  <body>
    <main>
      <h1>rust-config-tree Manual</h1>
      <p>Select a language to read the project manual.</p>
      <nav aria-label="Manual language">
        <a href="./en/">English</a>
        <a href="./zh/">中文</a>
        <a href="./ja/">日本語</a>
        <a href="./ko/">한국어</a>
        <a href="./fr/">Français</a>
        <a href="./de/">Deutsch</a>
        <a href="./es/">Español</a>
        <a href="./pt/">Português</a>
        <a href="./sv/">Svenska</a>
        <a href="./fi/">Suomi</a>
        <a href="./nl/">Nederlands</a>
      </nav>
    </main>
  </body>
</html>
HTML

touch "${OUT_DIR}/.nojekyll"
