# 脚本

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

请在仓库根目录运行脚本。

## `publish-pages.sh`

构建所有语言的 mdBook 手册，输出到 `target/mdbook`。

```bash
scripts/publish-pages.sh
```

变更推送到 `main` 后，`.github/workflows/pages.yml` 会部署 GitHub Pages。

## `publish-crate.sh`

运行 crate 发布检查，默认执行 `cargo publish --dry-run`。如果
`package.version` 已经存在于 crates.io，脚本会自动 bump patch 版本。

```bash
scripts/publish-crate.sh
```

当前版本已存在时，可以指定 bump 的版本位：

```bash
scripts/publish-crate.sh --bump minor
scripts/publish-crate.sh --bump major
```

发布到 crates.io：

```bash
scripts/publish-crate.sh --execute
```

真实发布前脚本要求 git 工作区干净。使用 `--no-bump` 可以在版本已存在时直接
失败，而不是自动 bump。

发布步骤会针对临时 crates.io/index 网络失败进行重试。可以用环境变量调整：

```bash
PUBLISH_ATTEMPTS=8 PUBLISH_RETRY_DELAY=15 scripts/publish-crate.sh
```

## `release.sh`

运行完整发布流程：

1. 构建 mdBook Pages artifact。
2. 运行 Rust 检查。
3. 提交并推送代码。
4. 等待 GitHub Pages workflow。
5. 发布 crate。

默认是 dry-run：

```bash
scripts/release.sh
```

执行完整发布：

```bash
scripts/release.sh --execute --message "Release 0.1.3"
```

完整发布脚本会先准备 crate 版本再提交，因此版本 bump 会进入 release commit。

跳过等待 Pages workflow：

```bash
scripts/release.sh --execute --no-wait-pages
```
