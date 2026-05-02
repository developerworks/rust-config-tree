# GitHub Pages

[English](../en/github-pages.md) | [中文](github-pages.md)

本仓库使用 mdBook 和 GitHub Pages 发布手册。

mdBook source 位于 `manual/`，由 `book.toml` 配置：

```text
book.toml
manual/
  SUMMARY.md
  en/
    introduction.md
    quick-start.md
    ...
  zh/
    introduction.md
    quick-start.md
    ...
```

本地构建：

```bash
mdbook build
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
3. 构建手册。
4. 将 `target/mdbook` 上传为 Pages artifact。
5. 将 artifact 部署到 GitHub Pages。

发布 URL：

```text
https://developerworks.github.io/rust-config-tree/
```
