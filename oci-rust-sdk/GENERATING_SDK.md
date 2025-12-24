# OCI Rust SDK 자동 생성 가이드

이 가이드는 OCI TypeScript SDK로부터 Rust SDK를 자동으로 생성하는 방법을 설명합니다.

## 목차

- [개요](#개요)
- [파이프라인 아키텍처](#파이프라인-아키텍처)
- [빠른 시작](#빠른-시작)
- [새 서비스 추가하기](#새-서비스-추가하기)
- [고급 사용법](#고급-사용법)
- [문제 해결](#문제-해결)
- [코드 생성 규칙](#코드-생성-규칙)

---

## 개요

OCI Rust SDK는 Oracle의 공식 TypeScript SDK를 소스로 사용하여 Rust 코드를 자동으로 생성합니다.

### 전체 프로세스

```
TypeScript SDK → [Discovery] → [Parser] → [Generator] → Rust SDK
                    (Node.js)    (ts-morph)   (Rust/Tera)
```

### 지원되는 서비스

현재 다음 서비스가 구현되어 있습니다:
- **core** - Compute, VirtualNetwork, BlockStorage (986 모델)
- **audit** - Audit 서비스 (8 모델)
- **containerinstances** - Container Instances (76 모델)
- **osmanagementhub** - OS Management Hub (329 모델)
- **resourcesearch** - Resource Search (10 모델)

---

## 파이프라인 아키텍처

### 1단계: Service Discovery

**도구**: `tools/discovery/discover.ts`
**출력**: `data/services-metadata.json`

OCI TypeScript SDK에서 사용 가능한 모든 서비스를 검색하고 메타데이터를 추출합니다.

```bash
cd tools/discovery
npm run discover > ../../data/services-metadata.json
```

**출력 예시**:
```json
{
  "name": "core",
  "path": "lib/core",
  "clients": ["ComputeClient", "VirtualNetworkClient"],
  "modelCount": 678,
  "priority": 1
}
```

### 2단계: TypeScript Parsing

**도구**: `tools/parser/parse-models.ts`
**출력**: `data/parsed/{service}-models.json`

TypeScript AST를 분석하여 인터페이스, enum, 타입 정보를 추출합니다.

```bash
cd tools/parser
npx ts-node parse-models.ts --service=core > ../../data/parsed/core-models.json
```

**주요 기능**:
- Interface 파싱 (필드, 타입, required/optional)
- Enum 파싱 (variants, values)
- Namespace enum 처리 (`Shape.LifecycleState` → `ShapeLifecycleState`)
- **Rust 키워드 자동 escaping** (`type` → `r#type`)
- JSDoc 문서 보존

### 3단계: Rust Code Generation

**도구**: `tools/generator/`
**출력**: `src/{service}/models/*.rs`

Tera 템플릿 엔진을 사용하여 Rust 코드를 생성합니다.

```bash
cd tools/generator
cargo run --release -- \
  --input ../../data/parsed/core-models.json \
  --output ../../src/core/models
```

**생성되는 파일**:
- `{model_name}.rs` - 각 모델별 Rust 파일
- `mod.rs` - 모듈 export (수동 생성 필요)

### 4단계: Validation & Formatting

자동으로 실행됩니다:
- `cargo check` - 컴파일 검증
- `cargo clippy --fix --allow-dirty` - Lint 자동 수정
- `cargo clippy --all-features` - 최종 검증

---

## 빠른 시작

### 전체 서비스 생성

```bash
# 단일 서비스
./generate-sdk.sh audit

# 여러 서비스
./generate-sdk.sh core,audit,containerinstances

# 서비스 목록 확인
./generate-sdk.sh --list
```

### 생성 후 확인

```bash
# 특정 서비스 컴파일 확인
cargo check --features audit

# 모든 서비스 확인
cargo check --all-features

# Clippy 검증
cargo clippy --all-features
```

---

## 새 서비스 추가하기

### Step 1: TypeScript SDK 확인

```bash
# 서비스가 존재하는지 확인
ls oci-typescript-sdk/lib/
```

서비스 디렉토리 구조:
```
oci-typescript-sdk/lib/{service}/
├── lib/
│   └── model/
│       ├── *.ts (모델 파일들)
│       └── index.ts
└── index.ts
```

### Step 2: 서비스 이름 규칙 확인

**중요**: Rust SDK는 underscore 없는 이름을 사용합니다.

| TypeScript | Rust Feature | 모듈 경로 |
|------------|--------------|-----------|
| `container-instances` | `containerinstances` | `src/containerinstances/` |
| `os-management-hub` | `osmanagementhub` | `src/osmanagementhub/` |
| `resource-search` | `resourcesearch` | `src/resourcesearch/` |

### Step 3: SDK 생성

```bash
# 1. 서비스 생성
./generate-sdk.sh identity

# 2. Cargo.toml에 feature 추가
# (generate-sdk.sh가 자동으로 추가하지만 확인 필요)
```

`Cargo.toml`에 다음을 추가:
```toml
[features]
identity = []
```

### Step 4: src/lib.rs에 모듈 등록

```rust
#[cfg(feature = "identity")]
pub mod identity;
```

### Step 5: mod.rs 생성

**중요**: 생성기가 `mod.rs`를 자동으로 만들지 않으므로 수동 생성이 필요합니다.

```bash
# 자동 생성 스크립트
ls src/identity/models/*.rs | grep -v mod.rs | \
  xargs -I {} basename {} .rs | \
  awk '{print "pub mod "$1";\npub use "$1"::*;"}' > src/identity/models/mod.rs
```

또는 수동으로 `src/identity/models/mod.rs` 생성:
```rust
pub mod user;
pub use user::*;
pub mod group;
pub use group::*;
// ... 모든 모델에 대해 반복
```

### Step 6: 검증

```bash
# 컴파일 확인
cargo check --features identity

# Clippy 확인
cargo clippy --features identity

# 테스트 (있는 경우)
cargo test --features identity
```

---

## 고급 사용법

### 특정 모델만 생성

```bash
cd tools/generator
cargo run --release -- \
  --input ../../data/parsed/core-models.json \
  --output ../../src/core/models \
  --limit 10  # 처음 10개 모델만
```

### Dry-run 모드

```bash
cargo run --release -- \
  --input ../../data/parsed/audit-models.json \
  --output ../../src/audit/models \
  --dry-run  # 파일을 쓰지 않고 미리보기만
```

### 파서 직접 실행

```bash
cd tools/parser

# 서비스 파싱
npx ts-node parse-models.ts --service=identity > ../../data/parsed/identity-models.json

# 결과 확인
cat ../../data/parsed/identity-models.json | head -50
```

### 생성기 직접 실행

```bash
cd tools/generator

# 빌드
cargo build --release

# 실행
./target/release/oci-gen \
  --input ../../data/parsed/audit-models.json \
  --output ../../src/audit/models
```

---

## 문제 해결

### 문제 1: Parser 컴파일 오류

**증상**:
```
error TS1343: The 'import.meta' meta-property is only allowed...
```

**해결**:
`tools/parser/tsconfig.json` 확인:
```json
{
  "compilerOptions": {
    "module": "esnext",
    "moduleResolution": "node"
  }
}
```

### 문제 2: Rust 키워드 오류

**증상**:
```rust
error: expected identifier, found keyword `type`
```

**원인**: TypeScript의 `type` 필드가 Rust 키워드와 충돌

**해결**: 이미 자동으로 처리됨. 만약 오류가 발생한다면:
1. `tools/parser/parse-models.ts`의 `RUST_KEYWORDS` 배열 확인
2. `escapeRustKeyword()` 함수가 올바르게 호출되는지 확인

### 문제 3: mod.rs 누락

**증상**:
```
error[E0583]: file not found for module `models`
```

**해결**:
```bash
ls src/{service}/models/*.rs | grep -v mod.rs | \
  xargs -I {} basename {} .rs | \
  awk '{print "pub mod "$1";\npub use "$1"::*;"}' > src/{service}/models/mod.rs
```

### 문제 4: Namespace Enum 오류

**증상**:
```rust
// 잘못된 생성
pub type: Vec<Shape.LifecycleState>  // 점(.) 사용 불가
```

**해결**: 파서가 자동으로 처리합니다:
- `Shape.LifecycleState` → `ShapeLifecycleState`
- 별도의 enum 파일로 생성: `shape_lifecycle_state.rs`

### 문제 5: Clippy 경고

**증상**:
```
warning: this function has too many arguments
```

**해결**:
```bash
# 자동 수정
cargo clippy --fix --allow-dirty --features {service}

# 또는 generate-sdk.sh가 자동으로 실행
```

---

## 코드 생성 규칙

### Builder Pattern (CLAUDE.md 준수)

#### Required 필드가 있는 경우

```rust
// Required 필드 구조체
pub struct LaunchInstanceDetailsRequired {
    pub compartment_id: String,
    pub shape: String,
}

pub struct LaunchInstanceDetails {
    // Required 필드
    pub compartment_id: String,
    pub shape: String,
    // Optional 필드
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl LaunchInstanceDetails {
    // Required 구조체를 받는 생성자
    pub fn new(required: LaunchInstanceDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,
            shape: required.shape,
            display_name: None,
        }
    }

    // 모든 필드에 대한 set_* 메서드
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    // Optional 필드만 with_* 메서드 (Option 언래핑)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}
```

#### Required 필드가 없는 경우

```rust
pub struct Configuration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_period_days: Option<i64>,
}

impl Configuration {
    // 인자 없는 생성자
    pub fn new() -> Self {
        Self {
            retention_period_days: None,
        }
    }

    pub fn set_retention_period_days(mut self, value: Option<i64>) -> Self {
        self.retention_period_days = value;
        self
    }

    pub fn with_retention_period_days(mut self, value: i64) -> Self {
        self.retention_period_days = Some(value);
        self
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
```

### Enum 생성

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    #[serde(rename = "CREATING")]
    Creating,

    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "DELETING")]
    Deleting,

    /// Unknown value for forward compatibility
    #[serde(other)]
    UnknownValue,
}
```

### Rust 키워드 처리

TypeScript의 예약어가 Rust 키워드인 경우:

```rust
pub struct SomeModel {
    // TypeScript: { type: string }
    // Rust: r# prefix + serde rename
    #[serde(rename = "type")]
    pub r#type: String,
}

impl SomeModel {
    // 메서드명은 r# 없이
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
```

### 타입 매핑

| TypeScript | Rust | Import |
|------------|------|--------|
| `string` | `String` | - |
| `number` | `i64` | - |
| `boolean` | `bool` | - |
| `Date` | `DateTime<Utc>` | `chrono` |
| `any` | `serde_json::Value` | `serde_json` |
| `Array<T>` | `Vec<T>` | - |
| `{ [key: string]: V }` | `HashMap<String, V>` | `std::collections` |
| `string \| null` | `Option<String>` | - |

---

## 생성 파이프라인 확장

### 새로운 타입 추가

`tools/generator/src/type_mapper.rs` 수정:

```rust
pub fn map_type(&self, ts_type: &str, is_optional: bool) -> (String, bool, bool) {
    let (base_type, needs_hashmap, needs_datetime) = match ts_type {
        "string" => ("String".to_string(), false, false),
        // 새 타입 추가
        "BigInt" => ("i128".to_string(), false, false),
        // ...
    };

    if is_optional {
        (format!("Option<{}>", base_type), needs_hashmap, needs_datetime)
    } else {
        (base_type, needs_hashmap, needs_datetime)
    }
}
```

### 템플릿 커스터마이징

`tools/generator/templates/model.rs.tera` 수정:

```rust
// 커스텀 derive 추가
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
                                               // ^^^^^^^^ 추가

// 커스텀 attribute 추가
{% if is_cached_model %}
#[cached]
{% endif %}
pub struct {{ name }} {
```

---

## 성능 최적화

### 병렬 생성

여러 서비스를 동시에 생성:

```bash
# 백그라운드에서 여러 서비스 생성
./generate-sdk.sh core &
./generate-sdk.sh identity &
./generate-sdk.sh objectstorage &
wait

# 모든 작업 완료 후 검증
cargo check --all-features
```

### 증분 생성

이미 생성된 서비스는 건너뛰기:

```bash
# 기존 모델 백업
mv src/core/models src/core/models.backup

# 새로 생성
./generate-sdk.sh core

# 비교
diff -r src/core/models.backup src/core/models
```

---

## CI/CD 통합

### GitHub Actions 예시

```yaml
name: Generate SDK

on:
  push:
    paths:
      - 'oci-typescript-sdk/**'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
        with:
          submodules: recursive

      - name: Setup Node.js
        uses: actions/setup-node@v2
        with:
          node-version: '18'

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Generate SDK
        run: ./generate-sdk.sh core,audit,identity

      - name: Run Tests
        run: cargo test --all-features

      - name: Create PR
        uses: peter-evans/create-pull-request@v4
        with:
          title: 'Auto-generated SDK updates'
```

---

## 참고 자료

- **CLAUDE.md** - Builder pattern 규칙
- **tools/README.md** - 도구 상세 문서
- **GENERATION_REPORT.md** - 생성 결과 보고서
- **VALIDATION_REPORT.md** - 검증 결과

---

## 기여하기

SDK 생성 파이프라인 개선:

1. **Parser 개선**: `tools/parser/parse-models.ts`
   - 새로운 TypeScript 패턴 지원
   - 문서 추출 개선

2. **Generator 개선**: `tools/generator/src/`
   - 새로운 Rust 패턴 생성
   - 타입 매핑 확장

3. **Template 개선**: `tools/generator/templates/`
   - 코드 품질 향상
   - 문서 포맷 개선

---

**최종 업데이트**: 2025-12-24
**버전**: 0.3.0
