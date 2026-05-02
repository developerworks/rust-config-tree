# GitHub Pages

[English](../en/github-pages.html) | [中文](github-pages.html) | [日本語](../ja/github-pages.html) | [한국어](../ko/github-pages.html) | [Français](../fr/github-pages.html) | [Deutsch](../de/github-pages.html) | [Español](../es/github-pages.html) | [Português](../pt/github-pages.html) | [Svenska](../sv/github-pages.html) | [Suomi](../fi/github-pages.html) | [Nederlands](../nl/github-pages.html)

本仓库使用 mdBook 和 GitHub Pages 发布手册。

每种语言的手册都是独立的 mdBook 项目。每种语言都有自己的 `SUMMARY.md`，
因此左侧目录只显示当前语言的页面：

```text
manual/
  en/
    book.toml
    SUMMARY.md
    introduction.md
    quick-start.md
    ...
  zh/
    book.toml
    SUMMARY.md
    introduction.md
    quick-start.md
    ...
  ja/
    book.toml
    SUMMARY.md
    introduction.md
    quick-start.md
    ...
  ko/
  fr/
  de/
  es/
  pt/
  sv/
  fi/
  nl/
```

本地构建：

```bash
scripts/publish-pages.sh
```

生成站点写入：

```text
target/mdbook
```

## 发布 Workflow

`.github/workflows/pages.yml` 中的 workflow 会在 push 到 `main` 时运行，也
支持手动触发。它会：

1. Checkout 仓库。
2. 安装 mdBook。
3. 运行 `scripts/publish-pages.sh`。
4. 将 `target/mdbook` 上传为 Pages artifact。
5. 将 artifact 部署到 GitHub Pages。

发布 URL：

```text
https://developerworks.github.io/rust-config-tree/
```

## Crate 发布

完整的提交、推送、Pages 部署和 crate 发布流程：

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

在仓库根目录使用 crate 发布辅助脚本：

```bash
scripts/publish-crate.sh
```

默认模式会运行检查和 `cargo publish --dry-run`。如果当前版本已经存在于
crates.io，脚本会自动 bump patch 版本。检查通过后发布到 crates.io：

```bash
scripts/publish-crate.sh --execute
```

脚本用法汇总在 `scripts/README.md`。
