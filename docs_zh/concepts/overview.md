# 概述 (Overview)

## 简介

NautilusTrader 是一个开源、高性能 (high-performance)、生产级的算法交易平台，
为量化交易者 (quantitative traders) 提供使用事件驱动 (event-driven) 引擎 (engine) 在历史数据上回测 (backtest) 自动化交易策略 (strategy) 投资组合 (portfolio) 的能力，并且能够将相同的策略部署到实盘交易 (live trading) 中，无需更改代码。

该平台以 *AI 优先* 为理念，旨在在高性能且稳健的 Python 原生环境中开发和部署算法交易策略。这有助于解决保持 Python 研究/回测环境与生产实盘交易环境一致性的对等性挑战。

NautilusTrader 的设计、架构 (architecture) 和实现理念将软件正确性和安全性置于最高优先级，旨在支持 Python 原生的、关键任务级的交易系统回测和实盘部署工作负载。

该平台同时具备通用性和资产类别无关性 -- 任何 REST API 或 WebSocket 数据流都可以通过模块化的适配器 (adapter) 进行集成。它支持跨广泛资产类别和金融工具 (instrument) 类型的高频交易，包括外汇、股票、期货、期权、加密货币、DeFi 和博彩 -- 支持同时在多个交易场所 (venue) 上无缝运作。

## 特性

- **快速**: 核心使用 Rust 编写，采用 [tokio](https://crates.io/crates/tokio) 实现异步网络通信。
- **可靠**: Rust 驱动的类型安全和线程安全，可选 Redis 支持的状态持久化。
- **可移植**: 操作系统无关，可在 Linux、macOS 和 Windows 上运行。支持 Docker 部署。
- **灵活**: 模块化适配器意味着任何 REST API 或 WebSocket 数据流都可以集成。
- **高级**: 有效期类型 `IOC`、`FOK`、`GTC`、`GTD`、`DAY`、`AT_THE_OPEN`、`AT_THE_CLOSE`，高级订单 (order) 类型和条件触发器。执行 (execution) 指令 `post-only`、`reduce-only` 和冰山订单。关联订单包括 `OCO`、`OUO`、`OTO`。
- **可定制**: 添加用户自定义组件 (component)，或利用[缓存 (cache)](cache.md) 和[消息总线 (message bus)](message_bus.md) 从零构建完整系统。
- **回测**: 使用历史报价 (quote) Tick、成交 (trade) Tick、K线 (Bar)、订单簿和自定义数据，支持多交易场所、多金融工具和多策略的纳秒级精度同步运行。
- **实盘**: 回测和实盘部署之间使用完全相同的策略实现。
- **多交易场所**: 多交易场所功能支持做市和统计套利策略。
- **AI 训练**: 回测引擎速度足够快，可用于训练 AI 交易代理 (RL/ES)。

![Nautilus](https://github.com/nautechsystems/nautilus_trader/blob/develop/assets/nautilus-art.png?raw=true "nautilus")
> *nautilus - 源自古希腊语 'sailor'（水手）和 naus 'ship'（船）。*
>
> *鹦鹉螺壳由模块化的腔室组成，其增长因子近似于对数螺旋。
> 这个理念可以转化为设计和架构的美学。*

## 为什么选择 NautilusTrader？

- **高性能事件驱动 Python**: 原生二进制核心组件。
- **回测与实盘交易的对等性**: 相同的策略代码。
- **降低运营风险 (risk)**: 增强的风险管理功能、逻辑准确性和类型安全。
- **高度可扩展**: 消息总线、自定义组件和 Actor、自定义数据、自定义适配器。

传统上，交易策略研究和回测可能使用 Python 的向量化方法进行，
然后策略需要使用 C++、C#、Java 或其他静态类型语言以更加事件驱动的方式重新实现。
原因在于向量化回测代码无法表达实时交易中细粒度的时间和事件依赖复杂性，
而编译型语言由于其天然的更高性能和类型安全，已被证明更为适合。

NautilusTrader 在这里的关键优势之一是，这个重新实现的步骤现在被规避了 -- 因为平台的关键核心组件全部使用 [Rust](https://www.rust-lang.org/) 或 [Cython](https://cython.org/) 编写。
这意味着我们在使用正确的工具做正确的事，系统编程语言编译出高性能的二进制文件，
CPython C 扩展模块则提供了 Python 原生环境，适合专业的量化交易者和交易公司使用。

## 使用场景

该软件包有三个主要使用场景：

- 在历史数据上回测交易系统（`backtest`）。
- 使用实时数据和虚拟执行模拟交易系统（`sandbox`）。
- 将交易系统部署到真实账户或模拟账户进行实盘交易（`live`）。

项目代码库提供了实现上述功能的软件层框架。你可以在相应命名的子包中找到默认的 `backtest` 和 `live` 系统实现。`sandbox` 环境可以使用 sandbox 适配器构建。

:::note

- 所有示例都将使用这些默认的系统实现。
- 我们认为交易策略是端到端交易系统的子组件，这些系统包括应用层和基础设施层。

:::

## 分布式

该平台设计为易于集成到更大的分布式系统中。
为此，几乎所有配置 (configuration) 和领域对象都可以使用 JSON、MessagePack 或 Apache Arrow (Feather) 进行序列化 (serialization)，以便通过网络通信。

## 公共核心

公共系统核心被所有节点 (node) [环境上下文](/concepts/architecture.md#environment-contexts)（`backtest`、`sandbox` 和 `live`）所使用。
用户定义的 `Actor`、`Strategy` 和 `ExecAlgorithm` 组件在这些环境上下文中被一致地管理。

## 回测

回测可以通过以下方式实现：首先将数据直接提供给 `BacktestEngine`，或通过更高层级的 `BacktestNode` 和 `ParquetDataCatalog` 提供数据，然后以纳秒级精度将数据通过系统运行。

## 实盘交易

`TradingNode` 可以从多个数据和执行客户端接收数据和事件，支持模拟/纸上交易账户和真实账户。通过在单个[事件循环](https://docs.python.org/3/library/asyncio-eventloop.html)上异步运行可以实现高性能，
还可以利用 [uvloop](https://github.com/MagicStack/uvloop) 实现（适用于 Linux 和 macOS）进一步提升性能。

## 领域模型

该平台具有全面的交易领域模型，包括各种值类型（如 `Price` 和 `Quantity`），以及更复杂的实体（如 `Order` 和 `Position` 对象），这些对象用于聚合多个事件以确定状态。

## 时间戳

平台内所有时间戳均以 UTC 纳秒精度记录。

时间戳字符串遵循 ISO 8601 (RFC 3339) 格式，具有 9 位（纳秒）或 3 位（毫秒）的小数精度，（但主要使用纳秒），始终保留所有数字包括尾随零。
这些可以在日志 (logging) 消息和对象的调试/显示输出中看到。

时间戳字符串由以下部分组成：

- 完整日期组件始终存在：`YYYY-MM-DD`。
- 日期和时间组件之间使用 `T` 分隔符。
- 始终使用纳秒精度（9 位小数），或在某些情况下（如 GTD 过期时间）使用毫秒精度（3 位小数）。
- 始终使用 UTC 时区，以 `Z` 后缀标识。

示例：`2024-01-05T15:30:45.123456789Z`

有关完整规范，请参阅 [RFC 3339: Date and Time on the Internet](https://datatracker.ietf.org/doc/html/rfc3339)。

## UUID

平台使用通用唯一标识符 (UUID) 第 4 版 (RFC 4122) 作为唯一标识符。
我们的高性能实现利用 `uuid` crate 在从字符串解析时进行正确性验证，确保输入的 UUID 符合规范。

有效的 UUID v4 由以下部分组成：

- 32 个十六进制数字，分为 5 组显示。
- 各组之间用连字符分隔：`8-4-4-4-12` 格式。
- 第 4 版标识（由第三组以 "4" 开头表示）。
- RFC 4122 变体标识（由第四组以 "8"、"9"、"a" 或 "b" 开头表示）。

示例：`2d89666b-1a1e-4a75-b193-4eb3b454c757`

有关完整规范，请参阅 [RFC 4122: A Universally Unique Identifier (UUID) URN Namespace](https://datatracker.ietf.org/doc/html/rfc4122)。

## 数据 (data) 类型

以下市场数据类型可以请求历史数据，也可以在交易场所/数据提供商提供且已在集成适配器中实现的情况下，作为实时流进行订阅。

- `OrderBookDelta` (L1/L2/L3)
- `OrderBookDeltas` (容器类型)
- `OrderBookDepth10` (每侧固定 10 档深度)
- `QuoteTick`
- `TradeTick`
- `Bar`
- `Instrument`
- `InstrumentStatus`
- `InstrumentClose`

以下 `PriceType` 选项可用于 K线聚合：

- `BID`
- `ASK`
- `MID`
- `LAST`

## K线聚合

以下 `BarAggregation` 方法可用：

- `MILLISECOND`
- `SECOND`
- `MINUTE`
- `HOUR`
- `DAY`
- `WEEK`
- `MONTH`
- `YEAR`
- `TICK`
- `VOLUME`
- `VALUE`（又称美元 K线）
- `RENKO`（基于价格的砖形图）
- `TICK_IMBALANCE`
- `TICK_RUNS`
- `VOLUME_IMBALANCE`
- `VOLUME_RUNS`
- `VALUE_IMBALANCE`
- `VALUE_RUNS`

当前已实现的聚合方法：

- `MILLISECOND`
- `SECOND`
- `MINUTE`
- `HOUR`
- `DAY`
- `WEEK`
- `MONTH`
- `YEAR`
- `TICK`
- `VOLUME`
- `VALUE`
- `RENKO`

上述列出但未在已实现列表中重复出现的聚合方法已计划但尚未可用。

:::note K 线聚合方法分类

NautilusTrader 支持的聚合方法可按触发维度分为四类：

| 类别 | 方法 | 触发条件 |
|------|------|---------|
| **时间驱动** | `MILLISECOND`、`SECOND`、`MINUTE`、`HOUR`、`DAY`、`WEEK`、`MONTH`、`YEAR` | 固定时间间隔结束时触发 |
| **成交量驱动** | `TICK`、`VOLUME`、`VALUE`（美元 K 线） | 累积到指定 Tick 数 / 成交量 / 成交金额时触发 |
| **价格驱动** | `RENKO` | 价格移动指定幅度时触发（砖形图） |
| **信息驱动（计划中）** | `TICK_IMBALANCE`、`TICK_RUNS`、`VOLUME_IMBALANCE`、`VOLUME_RUNS`、`VALUE_IMBALANCE`、`VALUE_RUNS` | 基于订单流不平衡程度触发 |

信息驱动类型来自量化金融研究（微观结构分析），适合检测市场参与者的知情/非知情交易行为。这些方法已列在规格中但尚未实现，将在未来版本中逐步添加。
:::

价格类型和 K线聚合可以通过 `BarSpecification` 以任意方式组合，步长 >= 1。
这提供了最大的灵活性，现在允许为实盘交易聚合替代 K线。

## 账户类型

以下账户类型在实盘和回测环境中均可用：

- `Cash` 单币种（基础货币）
- `Cash` 多币种
- `Margin` 单币种（基础货币）
- `Margin` 多币种
- `Betting` 单币种

## 订单类型

以下订单类型可用（取决于交易场所是否支持）：

- `MARKET`
- `LIMIT`
- `STOP_MARKET`
- `STOP_LIMIT`
- `MARKET_TO_LIMIT`
- `MARKET_IF_TOUCHED`
- `LIMIT_IF_TOUCHED`
- `TRAILING_STOP_MARKET`
- `TRAILING_STOP_LIMIT`

## 值类型

以下值类型由 128 位或 64 位原始整数值支持，具体取决于编译时使用的[精度模式](../getting_started/installation.md#precision-mode)。

- `Price`
- `Quantity`
- `Money`

### 高精度模式 (128 位)

当 `high-precision` 特性标志被**启用**时（默认），值使用以下规格：

| 类型         | 原始存储 | 最大精度 | 最小值              | 最大值             |
|:-------------|:---------|:---------|:--------------------|:-------------------|
| `Price`      | `i128`   | 16       | -17,014,118,346,046 | 17,014,118,346,046 |
| `Money`      | `i128`   | 16       | -17,014,118,346,046 | 17,014,118,346,046 |
| `Quantity`   | `u128`   | 16       | 0                   | 34,028,236,692,093 |

### 标准精度模式 (64 位)

当 `high-precision` 特性标志被**禁用**时，值使用以下规格：

| 类型         | 原始存储 | 最大精度 | 最小值              | 最大值             |
|:-------------|:---------|:---------|:--------------------|:-------------------|
| `Price`      | `i64`    | 9        | -9,223,372,036      | 9,223,372,036      |
| `Money`      | `i64`    | 9        | -9,223,372,036      | 9,223,372,036      |
| `Quantity`   | `u64`    | 9        | 0                   | 18,446,744,073     |

:::tip 精度模式选择建议

| 场景 | 推荐模式 | 原因 |
|------|---------|------|
| 加密货币交易（精度可能 > 9 位） | **128 位**（默认） | 部分极低价代币的价格精度超过 9 位 |
| 外汇/股票交易（精度 ≤ 5 位） | 64 位可用 | 外汇报价通常 4-5 位小数，64 位完全足够 |
| AI 强化学习训练（速度优先） | 64 位 | 更低内存占用，算术运算更快 |
| 高量值合约（如 BTC 大额交易） | **128 位** | 避免大数值与高精度同时出现时的整数溢出 |

大多数用户应保持默认的 128 位高精度模式。仅在明确了解精度限制且有充分性能理由时才切换到 64 位模式。
:::
