# GitHub Pages

[English](../en/github-pages.html) | [中文](github-pages.html)

本仓库使用 mdBook 和 GitHub Pages 发布手册。

英文手册和中文手册是两个独立的 mdBook 项目。每种语言都有自己的
`SUMMARY.md`，因此左侧目录只显示当前语言的页面：

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
