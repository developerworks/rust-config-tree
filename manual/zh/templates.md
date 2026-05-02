# 模板生成

[English](../en/templates.md) | [中文](templates.md)

模板由运行时使用的同一个 `confique` schema 生成。`confique` 负责渲染实际
模板内容，包括文档注释、默认值、必填字段和声明的环境变量名。

使用 `write_config_templates`：

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

输出格式由输出路径推断：

- `.yaml` 和 `.yml` 生成 YAML。
- `.toml` 生成 TOML。
- `.json` 和 `.json5` 生成 JSON5-compatible 模板。
- 未知或缺失扩展名生成 YAML。

## 模板 Source 选择

模板生成按这个顺序选择 source tree：

1. 已存在的 config path。
2. 已存在的 output template path。
3. 将 output path 作为新的空 template tree。

这样项目可以从当前配置更新模板、更新已有模板集，或仅从 schema 创建新的
模板集。

## 镜像 Include Tree

如果 source 文件声明了 include，生成的模板会在 output 目录下镜像这些
include path。

```yaml
# config.yaml
include:
  - config/server.yaml
```

生成 `config.example.yaml` 会写入：

```text
config.example.yaml
config/server.yaml
```

相对 include 目标会镜像到 output 文件父目录下。绝对 include 目标保持
绝对路径。

## 自动 Section 拆分

当 source 文件没有 include 时，crate 可以从嵌套 schema section 推导
include 目标。对于包含 `server` section 的 schema，空 root template source
可以生成：

```text
config.example.yaml
config/server.yaml
```

root template 会得到 include block，`config/server.yaml` 只包含 `server`
section。嵌套 section 会继续递归拆分。
