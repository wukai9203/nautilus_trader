# Rust

[Rust](https://www.rust-lang.org/learn) 编程语言非常适合实现平台和系统的关键核心（mission-critical core）。
其强类型系统、所有权模型（ownership model）和编译时检查从根本上消除了内存（memory）错误和数据竞争（data race），
同时零成本抽象（zero-cost abstraction）和无垃圾回收器的特性提供了接近 C 语言的性能——这对高频交易工作负载至关重要。

## Cargo 清单约定

- 在 `[dependencies]` 中，先按字母顺序列出内部 crate（`nautilus-*`），插入一个空行，然后按字母顺序列出外部必需依赖，再插入一个空行，最后按字母顺序列出可选依赖（带有 `optional = true` 的）。保留依赖项的行内注释。
- 在每个构建 Python 产物的 `extension-module` 特性列表中添加 `"python"`，使其紧邻 `"pyo3/extension-module"`，以便完整的 Python 栈一目了然。
- 当清单单独分组适配器时（例如 `crates/pyo3`），将 `# Adapters` 块紧接在内部 crate 列表下方，以便下游使用者能快速浏览适配器覆盖范围。
- 始终在 `[dev-dependencies]` 和 `[build-dependencies]` 部分前保留一个空行。
- 当特性或依赖集发生变化时，对相关清单应用相同的布局，以避免 crate 之间的偏差。
- 对 `bin/` 源文件使用 snake_case 文件名（例如 `bin/ws_data.rs`），并在每个 `[[bin]]` 部分中反映这些路径。
- 保持 `[[bin]] name` 条目使用 kebab-case（例如 `name = "hyperliquid-ws-data"`），以便编译后的二进制文件保留其预期的 CLI 名称。

## 版本管理指南

- 对共享依赖使用工作区继承（workspace inheritance）（例如 `serde = { workspace = true }`）。
- 仅对不属于工作区的 crate 特有依赖直接固定版本。
- 将工作区提供的依赖分组放在 crate 专用依赖之前，以便继承关系易于审查。

## 特性标志约定

- 优先使用增量特性标志——启用一个特性不应破坏现有功能。
- 使用描述性的标志名称来说明启用了什么功能。
- 在 crate 级别的文档中记录每个特性，以便使用者了解其切换的内容。
- 常见模式：
  - `high-precision`：切换值类型支撑（64 位或 128 位整数），以支持需要更高精度的领域。
  - `default = []`：保持默认值最小化。
  - `python`：启用 Python 绑定。
  - `extension-module`：构建 Python 扩展模块（始终包含 `python`）。
  - `ffi`：启用 C FFI 绑定。
  - `stubs`：暴露测试桩（testing stubs）。

## 构建配置

为避免开发过程中不必要的重新构建，请在不同构建目标之间对齐 cargo 特性、配置文件（profile）和标志。
Cargo 的构建缓存以特性、配置文件和标志的精确组合为键——任何不匹配都会触发完全重新构建。

### 对齐目标（测试和代码检查）

| 目标                        | 特性                             | 配置文件  | `--all-targets` | `--no-deps` | 用途           |
|-----------------------------|----------------------------------|-----------|-----------------|-------------|----------------|
| `cargo-test`                | `ffi,python,high-precision,defi` | `nextest` | ✓（隐式）       | 不适用      | 运行测试。     |
| `cargo-clippy`（pre-commit）| `ffi,python,high-precision,defi` | `nextest` | ✓               | 不适用      | 检查所有代码。 |

这些目标共享相同的特性集和配置文件，使 cargo 能在代码检查和测试之间复用编译产物，无需重新构建。
`nextest` 配置文件的使用与大多数核心维护者使用 cargo-nextest 运行测试的工作流程保持一致。

### 文档构建

文档使用 `make docs-rust` 单独构建，其运行：

```bash
cargo +nightly doc --all-features --no-deps --workspace
```

此命令使用 nightly 工具链和 `--all-features`，而非上述对齐的特性集，因此不会与测试/代码检查共享构建产物。

### 独立目标（Python 扩展构建）

| 目标          | 特性                                 | 配置文件  | 说明 |
|---------------|--------------------------------------|-----------|------|
| `build`       | 包含 `extension-module` + 子集       | `release` | 需要不同的特性以构建 PyO3 扩展模块。 |
| `build-debug` | 包含 `extension-module` + 子集       | `dev`     | 需要不同的特性以构建 PyO3 扩展模块。 |

Python 扩展构建有意使用不同的特性（`extension-module` 是必需的），因此会触发重新构建。这是预期且不可避免的。

### 需要避免的重新构建触发因素

以下任何不匹配都会导致完全重新构建：

- 不同的特性组合（例如 `--features "a,b"` vs `--features "a,c"`）。
- 不同的 `--no-default-features` 用法（启用/禁用默认特性）。
- 不同的配置文件（例如 `dev` vs `nextest` vs `release`）。

添加新的构建目标或修改现有目标时，请与测试/代码检查组保持对齐，以维持快速增量构建。

## 模块组织

- 保持模块专注于单一职责。
- 定义子模块时使用 `mod.rs` 作为模块根。
- 优先使用相对扁平的层次结构而非深层嵌套，以保持路径可管理。
- 为方便使用，从 crate 根重新导出（re-export）常用项。

## 代码风格和约定

### 文件头要求

所有 Rust 文件必须包含标准化的版权头：

```rust
// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------
```

:::info 自动化执行
`check_copyright_year.sh` pre-commit 钩子会验证版权头是否包含当前年份。
:::

### 代码格式化

运行 `make format` 时，rustfmt 会自动处理导入格式化。
该工具将导入分为几组（标准库、外部 crate、本地导入），并在每组内按字母顺序排序。

在此部分中，遵循以下间距规则：

- 在**函数之间留一个空行**（包括测试）——这提高了可读性，并与 `rustfmt` 的默认行为一致。
- 在**每个文档注释**（`///` 或 `//!`）**上方留一个空行**，使注释与前面的代码块清晰分离。

#### 字符串格式化

优先使用内联格式字符串而非位置参数：

```rust
// 推荐 - 使用变量名的内联格式
anyhow::bail!("Failed to subtract {n} months from {datetime}");

// 而非 - 位置参数
anyhow::bail!("Failed to subtract {} months from {}", n, datetime);
```

这使消息更具可读性和自文档性，尤其是当存在多个变量时。

### 类型限定

在代码中遵循以下类型限定约定：

- **anyhow**：始终完全限定 `anyhow` 宏（`anyhow::bail!`、`anyhow::anyhow!`）和 Result 类型（`anyhow::Result<T>`）。
- **Nautilus 领域类型**：不要完全限定 Nautilus 领域类型。导入后直接使用（例如 `Symbol`、`InstrumentId`、`Price`）。
- **tokio**：通常完全限定 `tokio` 类型，因为它们在标准库和其他 crate 中可能有等效项（例如 `tokio::spawn`、`tokio::time::timeout`）。

```rust
use nautilus_model::identifiers::Symbol;

pub fn process_symbol(symbol: Symbol) -> anyhow::Result<()> {
    if !symbol.is_valid() {
        anyhow::bail!("Invalid symbol: {symbol}");
    }

    tokio::spawn(async move {
        // 异步处理 symbol
    });

    Ok(())
}
```

:::info 自动化执行
`check_anyhow_usage.sh` pre-commit 钩子会自动执行这些 anyhow 约定。
:::

### 日志记录

- 完全限定日志宏以明确后端：
  - 在同步核心 crate 中使用 `log::…`（`log::info!`、`log::warn!` 等）。
  - 在异步运行时、适配器和外围组件中使用 `tracing::…`（`tracing::debug!`、`tracing::info!` 等）。
- 消息以大写字母开头，优先使用完整句子，省略末尾句号（例如 `"Processing batch"`，而非 `"Processing batch."`）。

:::info 自动化执行
`check_logging_macro_usage.sh` pre-commit 钩子会强制使用完全限定的日志宏。
:::

### 错误处理

一致地使用结构化错误处理模式：

1. **主要模式**：对可能失败的函数使用 `anyhow::Result<T>`：

   ```rust
   pub fn calculate_balance(&mut self) -> anyhow::Result<Money> {
       // 实现
   }
   ```

2. **自定义错误类型**：对领域特定错误使用 `thiserror`：

   ```rust
   #[derive(Error, Debug)]
   pub enum NetworkError {
       #[error("Connection failed: {0}")]
       ConnectionFailed(String),
       #[error("Timeout occurred")]
       Timeout,
   }
   ```

3. **错误传播**：使用 `?` 运算符进行简洁的错误传播。

4. **错误创建**：优先使用 `anyhow::bail!` 进行带错误的提前返回：

   ```rust
   // 推荐 - 使用 bail! 进行提前返回
   pub fn process_value(value: i32) -> anyhow::Result<i32> {
       if value < 0 {
           anyhow::bail!("Value cannot be negative: {value}");
       }
       Ok(value * 2)
   }

   // 而非 - 冗长的 return 语句
   if value < 0 {
       return Err(anyhow::anyhow!("Value cannot be negative: {value}"));
   }
   ```

   **注意**：使用 `anyhow::bail!` 进行提前返回，但在闭包上下文中（如 `ok_or_else()`）无法提前返回时使用 `anyhow::anyhow!`。

5. **错误上下文**：`.context()` 消息使用小写字母以支持错误链式调用（专有名词/缩写除外）：

   ```rust
   // 好的 - 小写字母自然衔接
   parse_timestamp(value).context("failed to parse timestamp")?;

   // 例外 - 专有名词保持大写
   connect().context("BitMEX websocket did not become active")?;
   ```

:::info 自动化执行
`check_error_conventions.sh` 和 `check_anyhow_usage.sh` pre-commit 钩子会强制执行这些错误处理模式。
:::

### 异步模式

使用一致的 async/await 模式：

1. **异步函数命名**：不需要特殊后缀；优先使用自然名称。
2. **Tokio 用法**：使用 `tokio::spawn` 进行即发即忘的工作，并记录该后台任务预计何时完成。
3. **错误处理**：从异步函数返回 `anyhow::Result` 以与同步约定保持一致。
4. **取消安全性（cancellation safety）**：说明函数是否是取消安全的，以及取消时哪些不变量仍然成立。
5. **流处理**：使用 `tokio_stream`（或 `futures::Stream`）处理异步迭代器，以明确反压（back-pressure）。
6. **超时模式**：使用超时（`tokio::time::timeout`）包装网络或长时间运行的 await，并传播或处理超时错误。

### 属性模式

一致的属性使用和排序：

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.model")
)]
pub struct Symbol(Ustr);
```

对于具有大量 derive 属性的枚举：

```rust
#[repr(C)]
#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    AsRefStr,
    FromRepr,
    EnumIter,
    EnumString,
)]
#[strum(ascii_case_insensitive)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(eq, eq_int, module = "nautilus_trader.model")
)]
pub enum AccountType {
    /// 仅包含无杠杆现金资产的账户。
    Cash = 1,
    /// 使用账户资产作为抵押品进行保证金交易的账户。
    Margin = 2,
}
```

### 构造函数模式

一致地使用 `new()` vs `new_checked()` 约定：

```rust
/// 创建一个带有正确性检查的新 [`Symbol`] 实例。
///
/// # Errors
///
/// 如果 `value` 不是有效字符串，则返回错误。
///
/// # Notes
///
/// PyO3 需要 `Result` 类型以在 Python 中正确处理错误和打印堆栈跟踪。
pub fn new_checked<T: AsRef<str>>(value: T) -> anyhow::Result<Self> {
    // 实现
}

/// 创建一个新的 [`Symbol`] 实例。
///
/// # Panics
///
/// 如果 `value` 不是有效字符串，则 panic。
pub fn new<T: AsRef<str>>(value: T) -> Self {
    Self::new_checked(value).expect(FAILED)
}
```

始终对与正确性检查相关的 `.expect()` 消息使用 `FAILED` 常量：

```rust
use nautilus_core::correctness::FAILED;
```

### 常量和命名约定

常量使用 SCREAMING_SNAKE_CASE 并配以描述性名称：

```rust
/// 一秒中的纳秒数。
pub const NANOSECONDS_IN_SECOND: u64 = 1_000_000_000;

/// 1 分钟最后价格 K 线的规格。
pub const BAR_SPEC_1_MINUTE_LAST: BarSpecification = BarSpecification {
    step: NonZero::new(1).unwrap(),
    aggregation: BarAggregation::Minute,
    price_type: PriceType::Last,
};
```

### 哈希集合

在性能关键的热路径（hot path）中使用 `ahash` crate 的 `AHashMap` 和 `AHashSet`。
对于非性能关键的代码，优先使用标准 `HashMap`/`HashSet` 以保持简洁：

```rust
// 热路径 - 使用 AHashMap/AHashSet
use ahash::{AHashMap, AHashSet};

let mut symbols: AHashSet<Symbol> = AHashSet::new();
let mut prices: AHashMap<InstrumentId, Price> = AHashMap::new();

// 非热路径 - 标准库 HashMap/HashSet
use std::collections::{HashMap, HashSet};

let mut symbols: HashSet<Symbol> = HashSet::new();
let mut prices: HashMap<InstrumentId, Price> = HashMap::new();
```

**为什么使用 `ahash`？**

- **卓越的性能**：AHash 在可用时使用 AES-NI 硬件指令，与默认的 SipHash 相比提供 2-3 倍更快的哈希速度。
- **低碰撞率**：尽管是非加密哈希，AHash 对典型数据提供了出色的分布和低碰撞率。
- **直接替换**：与标准库集合完全兼容的 API。

**何时使用标准 `HashMap`/`HashSet`：**

- **非性能关键的代码**：对于性能不关键的简单场景（例如工厂注册表、配置映射、测试夹具），标准 `HashMap`/`HashSet` 是可接受的，甚至因其简洁性而更受青睐。
- **需要加密安全性**：当哈希洪泛攻击是隐患时（例如处理网络协议中的不可信用户输入），使用标准 `HashMap`。
- **网络客户端**：对于面向网络的组件，优先使用标准 `HashMap`，因为安全考虑优先于性能收益。
- **外部库边界**：与期望标准 `HashMap` 的外部库交互时使用它（例如 Arrow 序列化元数据）。

### 线程安全哈希映射模式

`AHashMap` 不是线程安全的。将其包装在 `Arc` 中仅能跨线程共享指针，但不能协调修改。仅在映射构建后不可变时使用 `Arc<AHashMap>`，否则需添加适当的同步机制。

```rust
// 避免：多线程修改时的数据竞争
let cache = Arc::new(AHashMap::new());
let cache_clone = Arc::clone(&cache);
tokio::spawn(async move {
    cache_clone.insert(key, value);  // 数据竞争
});
cache.insert(other_key, other_value);  // 数据竞争
```

**模式：**

1. **构建后不可变** — 构建映射一次，然后以只读方式共享：

   ```rust
   let mut map = AHashMap::new();
   map.insert(key1, value1);
   map.insert(key2, value2);
   let shared_map = Arc::new(map);  // 现在不可变

   // 多线程可以安全读取
   let map_clone = Arc::clone(&shared_map);
   tokio::spawn(async move {
       if let Some(value) = map_clone.get(&key1) {
           // 安全的只读访问
       }
   });
   ```

2. **并发读写** — 使用 `DashMap`：

   ```rust
   use dashmap::DashMap;

   let cache: Arc<DashMap<K, V>> = Arc::new(DashMap::new());

   // 多线程可以安全地并发读写
   cache.insert(key, value);
   if let Some(entry) = cache.get(&key) {
       // 安全的并发访问
   }
   ```

   `DashMap` 内部使用分片（sharding）和细粒度锁实现高效的并发访问。

3. **单线程热路径** — 在单线程上下文中使用普通的 `AHashMap`：

   ```rust
   struct Handler {
       instruments: AHashMap<Ustr, InstrumentAny>,
   }

   impl Handler {
       async fn next(&mut self) -> Option<()> {
           // Handler 运行在单个任务上，无并发访问
           self.instruments.insert(key, value);
           Ok(())
       }
   }
   ```

**决策树：**

- 构建后不可变 -> 使用 `Arc<AHashMap<K, V>>`
- 需要并发访问 -> 使用 `Arc<DashMap<K, V>>`
- 单线程访问 -> 使用普通的 `AHashMap<K, V>`

### 重新导出模式

按字母顺序组织重新导出，并将其放在 lib.rs 文件末尾：

```rust
// 重新导出
pub use crate::{
    nanos::UnixNanos,
    time::AtomicTime,
    uuid::UUID4,
};

// 模块级重新导出
pub use crate::identifiers::{
    account_id::AccountId,
    actor_id::ActorId,
    client_id::ClientId,
};
```

### 文档标准

所有文档注释使用第三人称陈述语气（例如 "Returns the account ID" 而非 "Return the account ID"）。

#### 模块级文档

所有模块必须有以简短描述开头的模块级文档：

```rust
//! Functions for correctness checks similar to the *design by contract* philosophy.
//!
//! This module provides validation checking of function or method conditions.
//!
//! A condition is a predicate which must be true just prior to the execution of
//! some section of code - for correct behavior as per the design specification.
```

对于带有特性标志的模块，清晰地记录它们：

```rust
//! # Feature flags
//!
//! This crate provides feature flags to control source code inclusion during compilation,
//! depending on the intended use case:
//!
//! - `ffi`: Enables the C foreign function interface (FFI) from [cbindgen](https://github.com/mozilla/cbindgen).
//! - `python`: Enables Python bindings from [PyO3](https://pyo3.rs).
//! - `extension-module`: Builds as a Python extension module (used with `python`).
//! - `stubs`: Enables type stubs for use in testing scenarios.
```

#### 字段文档

所有结构体和枚举字段必须有以句号结尾的文档：

```rust
pub struct Currency {
    /// The currency code as an alpha-3 string (e.g., "USD", "EUR").
    pub code: Ustr,
    /// The currency decimal precision.
    pub precision: u8,
    /// The ISO 4217 currency code.
    pub iso4217: u16,
    /// The full name of the currency.
    pub name: Ustr,
    /// The currency type, indicating its category (e.g. Fiat, Crypto).
    pub currency_type: CurrencyType,
}
```

#### 函数文档

为所有公共函数记录：

- 目的和行为
- 输入参数用法说明
- 错误条件（如适用）
- Panic 条件（如适用）

```rust
/// Returns a reference to the `AccountBalance` for the specified currency, or `None` if absent.
///
/// # Panics
///
/// Panics if `currency` is `None` and `self.base_currency` is `None`.
pub fn base_balance(&self, currency: Option<Currency>) -> Option<&AccountBalance> {
    // 实现
}
```

#### 错误和 Panic 文档格式

对于单行的错误和 Panic 文档，使用句首大写并遵循以下约定：

```rust
/// Returns a reference to the `AccountBalance` for the specified currency, or `None` if absent.
///
/// # Errors
///
/// Returns an error if the currency conversion fails.
///
/// # Panics
///
/// Panics if `currency` is `None` and `self.base_currency` is `None`.
pub fn base_balance(&self, currency: Option<Currency>) -> anyhow::Result<Option<&AccountBalance>> {
    // 实现
}
```

对于多行的错误和 Panic 文档，使用句首大写、列表项和句号结尾：

```rust
/// Calculates the unrealized profit and loss for the position.
///
/// # Errors
///
/// Returns an error if:
/// - The market price for the instrument cannot be found.
/// - The conversion rate calculation fails.
/// - Invalid position state is encountered.
///
/// # Panics
///
/// This function panics if:
/// - The instrument ID is invalid or uninitialized.
/// - Required market data is missing from the cache.
/// - Internal state consistency checks fail.
pub fn calculate_unrealized_pnl(&self, market_price: Price) -> anyhow::Result<Money> {
    // 实现
}
```

#### Safety 文档格式

对于 Safety 文档，使用 `SAFETY:` 前缀后跟简短描述，说明为什么 unsafe 操作是有效的：

```rust
/// Creates a new instance from raw components without validation.
///
/// # Safety
///
/// The caller must ensure that all input parameters are valid and properly initialized.
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
    // SAFETY: 调用者保证 ptr 有效且 len 正确
    Self {
        data: std::slice::from_raw_parts(ptr, len),
    }
}
```

对于内联 unsafe 块，在 unsafe 代码正上方使用 `SAFETY:` 注释：

```rust
impl Send for MessageBus {
    fn send(&self) {
        // SAFETY: 消息总线不应在线程间传递
        unsafe {
            // 此处为 unsafe 操作
        }
    }
}
```

## Python 绑定

Python 绑定通过 [PyO3](https://pyo3.rs) 提供，允许用户直接在 Python 中导入 NautilusTrader crate，无需 Rust 工具链。

### PyO3 命名约定

通过 PyO3 将 Rust 函数暴露给 Python 时：

1. Rust 符号**必须**以 `py_*` 为前缀，以在 Rust 代码库中明确其用途。
2. 使用 `#[pyo3(name = "…")]` 属性发布**不带** `py_` 前缀的 *Python* 名称，以保持 Python API 的整洁。

```rust
#[pyo3(name = "do_something")]
pub fn py_do_something() -> PyResult<()> {
    // …
}
```

:::info 自动化执行
`check_pyo3_conventions.sh` pre-commit 钩子会强制 PyO3 函数使用 `py_` 前缀。
:::

### 测试约定

- 使用 `mod tests` 作为标准测试模块名称，除非需要专门分区。
- 一致地使用 `#[rstest]` 属性，这种标准化减少了认知负担。
- 在 Rust 测试中**不要**使用 Arrange、Act、Assert 分隔注释。

:::info 自动化执行
`check_testing_conventions.sh` pre-commit 钩子会强制使用 `#[rstest]` 而非 `#[test]`。
:::

#### 参数化测试

一致地使用 `rstest` 属性，对于参数化测试：

```rust
#[rstest]
#[case("AUDUSD", false)]
#[case("AUD/USD", false)]
#[case("CL.FUT", true)]
fn test_symbol_is_composite(#[case] input: &str, #[case] expected: bool) {
    let symbol = Symbol::new(input);
    assert_eq!(symbol.is_composite(), expected);
}
```

#### 测试命名

使用描述场景的测试名称：

```rust
fn test_sma_with_no_inputs()
fn test_sma_with_single_input()
fn test_symbol_is_composite()
```

## Rust-Python 内存管理

使用 PyO3 绑定时，理解和避免 Rust 的 `Arc` 引用计数与 Python 垃圾回收器之间的引用循环（reference cycle）至关重要。
本节记录了在 Rust 回调持有结构体中处理 Python 对象的最佳实践。

### 引用循环问题

**问题**：在回调持有结构体中使用 `Arc<PyObject>` 会创建循环引用：

1. **Rust `Arc` 持有 Python 对象** -> 增加 Python 引用计数。
2. **Python 对象可能引用 Rust 对象** -> 创建循环。
3. **双方都无法被垃圾回收** -> 内存泄漏。

**有问题的模式示例**：

```rust
// 避免：这会创建引用循环
struct CallbackHolder {
    handler: Option<Arc<PyObject>>,  // Arc 包装导致循环
}
```

### 解决方案：基于 GIL 的克隆

**解决方案**：使用普通的 `PyObject` 并通过 `clone_py_object()` 进行适当的基于 GIL 的克隆：

```rust
use nautilus_core::python::clone_py_object;

// 正确：使用不带 Arc 包装的普通 PyObject
struct CallbackHolder {
    handler: Option<PyObject>,  // 无 Arc 包装
}

// 使用 clone_py_object 的手动 Clone 实现
impl Clone for CallbackHolder {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.as_ref().map(clone_py_object),
        }
    }
}
```

### 最佳实践

#### 1. 使用 `clone_py_object()` 克隆 Python 对象

```rust
// 克隆 Python 回调时
let cloned_callback = clone_py_object(&original_callback);

// 在手动 Clone 实现中
self.py_handler.as_ref().map(clone_py_object)
```

#### 2. 从回调持有结构体中移除 `#[derive(Clone)]`

```rust
// 之前：自动 derive 导致 PyObject 问题
#[derive(Clone)]  // 移除此行
struct Config {
    handler: Option<PyObject>,
}

// 之后：使用适当克隆的手动实现
struct Config {
    handler: Option<PyObject>,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            // 正常克隆普通字段
            url: self.url.clone(),
            // 对 Python 对象使用 clone_py_object
            handler: self.handler.as_ref().map(clone_py_object),
        }
    }
}
```

#### 3. 更新函数签名以接受 `PyObject`

```rust
// 之前：函数签名中的 Arc 包装
fn spawn_task(handler: Arc<PyObject>) { ... }  // 避免

// 之后：普通 PyObject
fn spawn_task(handler: PyObject) { ... }  // 推荐
```

#### 4. 创建 Python 回调时避免 `Arc::new()`

```rust
// 之前：包装在 Arc 中
let callback = Arc::new(py_function);  // 避免

// 之后：直接使用
let callback = py_function;  // 推荐
```

### 为什么这有效

`clone_py_object()` 函数：

- **在执行克隆操作前获取 Python GIL**。
- **通过 `clone_ref()` 使用 Python 原生引用计数**。
- **避免干扰 Python GC 的 Rust Arc 包装**。
- **通过适当的 GIL 管理维护线程安全**。

这种方法允许 Rust 和 Python 垃圾回收器正确工作，消除引用循环导致的内存泄漏。

## 常见反模式

1. **避免在热路径中使用 `.clone()`** — 优先借用或通过 `Arc` 共享所有权。
2. **避免在生产代码中使用 `.unwrap()`** — 通常使用 `?` 传播错误或将其映射为领域错误，但对锁中毒（lock poisoning）使用 unwrap 是可接受的，因为它表示应快速中止的严重程序状态。
3. **当 `&str` 足够时避免使用 `String`** — 在紧密循环中最小化分配。
4. **避免暴露内部可变性（interior mutability）** — 将 mutex/`RefCell` 隐藏在安全 API 之后。
5. **避免在 `Result<T, E>` 中使用大型结构体** — 对大型错误载荷使用 Box（`Box<dyn Error + Send + Sync>`）。

## Unsafe Rust

为了实现 Cython 和 Rust 之间的互操作，编写 `unsafe` Rust 代码是必要的。能够跨越安全 Rust 的边界正是使得实现 Rust 语言本身许多最基本特性成为可能的原因，正如 C 和 C++ 被用来实现它们自己的标准库一样。

我们会非常谨慎地使用 Rust 的 `unsafe` 功能——它启用了一小组额外的语言特性，从而改变了接口与调用者之间的契约，将保证正确性的部分责任从 Rust 编译器转移到了我们身上。目标是实现 `unsafe` 功能的优势，同时避免*任何*未定义行为（undefined behavior）。
Rust 语言设计者对未定义行为的定义可以在[语言参考](https://doc.rust-lang.org/stable/reference/behavior-considered-undefined.html)中找到。

### 安全策略

为保持正确性，任何 `unsafe` Rust 的使用都必须遵循我们的策略：

- 如果一个函数调用是 `unsafe` 的，其文档中*必须*有一个 `Safety` 部分，解释该函数为什么是 `unsafe` 的，涵盖函数期望调用者维护的不变量，以及如何履行该契约中的义务。
- 在文档注释的 Safety 部分记录每个函数为什么是 `unsafe` 的，并为所有 `unsafe` 块编写单元测试。
- 始终包含 `SAFETY:` 注释，解释为什么 unsafe 操作是有效的。
- **Crate 级别的 lint** — 每个暴露 FFI 符号的 crate 都启用 `#![deny(unsafe_op_in_unsafe_fn)]`。即使在 `unsafe fn` 内部，每个指针解引用或其他危险操作也必须包装在自己的 `unsafe { … }` 块中。
- **CVec 契约** — 对于跨 FFI 边界的原始向量，请阅读 [FFI 内存契约](ffi.md)。外部代码成为分配的所有者，并且**必须**恰好调用一次匹配的 `vec_drop_*` 函数。

### Unsafe 代码类别

代码库在以下类别中使用 unsafe Rust：

1. **FFI 边界** — 用于 C 互操作的原始指针操作。参见 [FFI 文档](ffi.md)。
2. **内部可变性** — 使用 `UnsafeCell` 实现具有受控访问模式的线程本地注册表。
3. **Unsafe Send/Sync** — 类型本身不是线程安全的，但通过运行时不变量满足 trait 约束（例如架构保证的单线程访问）。

### Unsafe Send/Sync 要求

不安全地实现 `Send` 或 `Sync` 时：

1. 精确记录哪些字段违反了 trait 要求。
2. 解释确保安全的运行时机制（例如单线程事件循环）。
3. 包含一个 `WARNING` 声明，说明违反不变量是未定义行为。
4. 优先使用运行时强制执行（断言、`Result` 返回）而非仅依赖文档保证。

```rust
// SAFETY: 包含非线程安全的 Rc<RefCell<...>>。
// 通过回测引擎架构保证单线程访问。
// WARNING: 实际跨线程发送是未定义行为。
#[allow(unsafe_code)]
unsafe impl Send for BacktestDataClient {}
```

### 纵深防御

当 unsafe 代码依赖于不变量时，添加防御机制：

- **类型验证**：在转换前进行运行时类型检查（例如 `TypeId` 比较）。
- **调试断言**：在调试构建中及早捕获内存损坏。
- **RAII 守卫**：确保在正常返回和 panic 路径上都进行清理。
- **运行时检查**：当不变量被违反时快速失败，而非继续不安全地执行。

## 工具配置

项目使用多种工具保障代码质量：

- **rustfmt**：自动代码格式化（参见 `rustfmt.toml`）。
- **clippy**：代码检查和最佳实践（参见 `clippy.toml`）。
- **cbindgen**：FFI 的 C 头文件生成。

## Rust 版本管理

项目通过 `rust-toolchain.toml` 固定到特定 Rust 版本。

**保持工具链与 CI 同步：**

```bash
rustup update       # 更新到最新稳定版 Rust
rustup show         # 验证正确的工具链处于活动状态
```

如果 pre-commit 在本地通过但在 CI 中失败，清除 pre-commit 缓存并重新运行：

```bash
pre-commit clean    # 清除缓存环境
make pre-commit     # 重新运行所有检查
```

这确保您使用与 CI 相同的 Rust 和 clippy 版本。

## 资源

- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) — Unsafe Rust 的暗黑艺术。
- [The Rust Reference – Unsafety](https://doc.rust-lang.org/stable/reference/unsafety.html)。
- [Safe Bindings in Rust – Russell Johnston](https://www.abubalay.com/blog/2020/08/22/safe-bindings-in-rust)。
- [Google – Rust and C interoperability](https://www.chromium.org/Home/chromium-security/memory-safety/rust-and-c-interoperability/)。

## Cap'n Proto 序列化

`nautilus-serialization` crate 提供了可选的 Cap'n Proto 序列化支持，用于高效的数据交换。
此特性是可选启用的，以避免标准构建需要 Cap'n Proto 编译器。

### 安装 Cap'n Proto

在使用 schema 之前安装 Cap'n Proto 编译器。所需版本在仓库根目录的 `capnp-version` 文件中指定。

有关各平台的详细安装说明，请参阅[环境设置](environment_setup.md#capn-proto)指南。

:::warning
Ubuntu 默认的 `capnproto` 包版本过旧。Linux 用户必须从源代码安装。
:::

验证安装：

```bash
capnp --version  # 应与 capnp-version 中的版本匹配
```

### Schema 开发工作流

Schema 文件位于 `crates/serialization/schemas/capnp/`：

- `common/` - 基础类型、标识符、枚举。
- `commands/` - 交易命令。
- `events/` - 订单和持仓事件。
- `data/` - 市场数据类型。

修改 schema 时：

1. 在相应子目录中编辑 `.capnp` schema 文件。
2. 重新生成 Rust 绑定：

   ```bash
   make regen-capnp
   # 或
   ./scripts/regen_capnp.sh
   ```

3. 审查更改：

   ```bash
   git diff crates/serialization/generated/capnp
   ```

4. 如需要，更新 `crates/serialization/src/capnp/conversions.rs` 中的转换。
5. 运行测试：

   ```bash
   make cargo-test EXTRA_FEATURES="capnp"
   ```

### 生成的代码

生成的 Rust 文件被检入 `crates/serialization/generated/capnp/`，原因如下：

- **docs.rs 兼容性**：文档构建环境缺少 Cap'n Proto 编译器。
- **贡献者便利性**：大多数开发者在标准开发中不需要安装 capnp。
- **构建可重现性**：确保跨环境一致的代码生成。

当启用 `capnp` 特性时，生成的文件会在构建期间通过 `build.rs` 自动创建，但我们将它们提交到仓库以支持在未安装编译器的情况下进行构建。

### 验证 Schema 一致性

在提交 schema 更改之前，确保生成的文件是最新的：

```bash
make check-capnp-schemas
```

此目标会：

1. 重新生成所有 schema 文件。
2. 验证不存在未提交的更改。
3. 如果 schema 不同步则失败。

CI 会自动运行此检查以捕获偏差。

### 使用 capnp 特性进行测试

```bash
# 使用 capnp 运行工作区测试
make cargo-test EXTRA_FEATURES="capnp"

# 使用 capnp 运行特定 crate 测试
make cargo-test-crate-nautilus-serialization FEATURES="capnp"

# 运行特定测试
cargo test -p nautilus-serialization --features capnp test_price_roundtrip
```

### Schema 演进指南

演进 schema 时：

- **仅进行增量更改**：在末尾添加新字段。
- **永远不要移除字段**：在注释中标记已弃用的字段。
- **永远不要复用字段编号**：即使在弃用之后。
- **测试往返兼容性**：确保新旧版本可以互操作。

Cap'n Proto 的演进规则允许 schema 更改而不破坏二进制兼容性，但您必须遵循这些约束以维护前向/后向兼容性。
