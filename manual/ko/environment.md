# 환경 변수

[English](../en/environment.html) | [中文](../zh/environment.html) | [日本語](../ja/environment.html) | [한국어](environment.html) | [Français](../fr/environment.html) | [Deutsch](../de/environment.html) | [Español](../es/environment.html) | [Português](../pt/environment.html) | [Svenska](../sv/environment.html) | [Suomi](../fi/environment.html) | [Nederlands](../nl/environment.html)

환경 변수 이름은 `confique`로 스키마에 선언합니다.

```rust
#[derive(Debug, Config)]
struct DatabaseConfig {
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}
```

`rust-config-tree`는 `confique::Config::META`에서 이 이름을 읽고 각 환경 변수를
정확한 필드 경로에 매핑하는 Figment 프로바이더를 만듭니다.

이 crate에는 delimiter 기반 Figment 환경 매핑을 사용하지 마세요.

```rust
// Do not use this pattern for rust-config-tree schemas.
Env::prefixed("APP_").split("_")
Env::prefixed("APP_").split("__")
```

`split("_")`는 underscore를 중첩 키 구분자로 처리합니다. 그러면
`APP_DATABASE_POOL_SIZE`가 `database.pool.size` 같은 경로가 되어 `pool_size` 같은
Rust 필드 이름과 충돌합니다.

`ConfiqueEnvProvider`를 사용하면 이 매핑은 명시적입니다.

```text
APP_DATABASE_POOL_SIZE -> database.pool_size
```

단일 underscore는 환경 변수 이름의 일부로 남습니다. Figment는 중첩 규칙을
추측하지 않습니다.

## Dotenv 로딩

런타임 프로바이더가 평가되기 전에 로더는 루트 설정 파일의 디렉터리에서 위로
올라가며 `.env` 파일을 찾습니다.

기존 프로세스 환경 변수는 보존됩니다. `.env`의 값은 빠진 환경 변수만 채웁니다.

예:

```dotenv
APP_SERVER_PORT=9000
APP_DATABASE_POOL_SIZE=64
```

스키마가 대응하는 `#[config(env = "...")]` 속성을 선언하면 이 변수들은 설정 파일
값을 override합니다.

## 값 파싱

브리지 프로바이더는 Figment가 환경 값을 파싱하게 합니다. `confique`의
`parse_env` hook은 호출하지 않습니다. Figment 환경 값 문법이 타입에 잘 맞지 않는
복잡한 값은 설정 파일에 두세요.
