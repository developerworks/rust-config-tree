# 트리 API

[English](../en/tree-api.html) | [中文](../zh/tree-api.html) | [日本語](../ja/tree-api.html) | [한국어](tree-api.html) | [Français](../fr/tree-api.html) | [Deutsch](../de/tree-api.html) | [Español](../es/tree-api.html) | [Português](../pt/tree-api.html) | [Svenska](../sv/tree-api.html) | [Suomi](../fi/tree-api.html) | [Nederlands](../nl/tree-api.html)

애플리케이션이 `confique`를 사용하지 않거나 순회 결과에 직접 접근해야 한다면 낮은
수준의 트리 API를 사용하세요.

```rust
use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use rust_config_tree::{ConfigSource, load_config_tree};

fn load_source(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

let tree = load_config_tree("config.yaml", load_source)?;

for node in tree.nodes() {
    println!("{}", node.path().display());
}
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## 순회 규칙

트리 로더는 다음을 수행합니다.

- 소스 경로를 사전식으로 정규화합니다.
- 빈 include 경로를 거부합니다.
- 상대 include를 선언한 파일 기준으로 해석합니다.
- 절대 include 경로를 보존합니다.
- 재귀 include 순환을 감지합니다.
- 다른 include branch를 통해 이미 로드된 파일을 건너뜁니다.

`ConfigTreeOptions`는 같은 레벨 include 순서를 뒤집을 수 있습니다.

```rust
use rust_config_tree::{ConfigTreeOptions, IncludeOrder};

let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
# let _ = options;
```

## 경로 헬퍼

경로 헬퍼는 사전식 처리만 합니다. symbolic link를 해석하지 않고 경로가 존재할
필요도 없습니다.

- `absolutize_lexical(path)`
- `normalize_lexical(path)`
- `resolve_include_path(parent_path, include_path)`
