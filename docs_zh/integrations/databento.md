# Databento

NautilusTrader 提供了一个用于集成（Integration） Databento API 和 [Databento 二进制编码（DBN）](https://databento.com/docs/standards-and-conventions/databento-binary-encoding)格式数据的适配器（Adapter）。
由于 Databento 纯粹是一个市场数据提供商，因此没有提供执行（Execution）客户端——不过仍然可以设置具有模拟执行的沙盒环境。
也可以将 Databento 数据与 Interactive Brokers 执行配合使用，或者计算传统资产类别信号用于加密货币交易。

此适配器的功能包括：

- 从 DBN 文件加载历史数据（Historical Data）并解码为 Nautilus 对象，用于回测（Backtest）或写入数据目录。
- 请求历史数据并解码为 Nautilus 对象，以支持实时交易和回测。
- 订阅实时数据源并解码为 Nautilus 对象，以支持实时交易和沙盒环境。

:::tip
[Databento](https://databento.com/signup) 目前为新账户注册提供 125 美元的免费数据额度（仅限历史数据）。

精心请求的话，这足够用于测试和评估目的。
我们建议你使用 [/metadata.get_cost](https://databento.com/docs/api-reference-historical/metadata/metadata-get-cost) 端点。
:::

## 概述

适配器实现将 [databento-rs](https://crates.io/crates/databento) crate 作为依赖项，
这是 Databento 提供的官方 Rust 客户端库。

:::info
**无需**额外安装 `databento`，因为适配器的核心组件被编译为静态库，并在构建过程中自动链接。
:::

以下适配器类可用：

- `DatabentoDataLoader`：从文件加载 Databento 二进制编码（DBN）数据。
- `DatabentoInstrumentProvider`：与 Databento API（HTTP）集成，提供最新或历史金融工具（Instrument）定义。
- `DatabentoHistoricalClient`：与 Databento API（HTTP）集成，用于历史市场数据请求。
- `DatabentoLiveClient`：与 Databento API（原始 TCP）集成，用于订阅实时数据源。
- `DatabentoDataClient`：提供 `LiveMarketDataClient` 实现，用于实时运行交易节点。

:::info
与其他集成适配器一样，大多数用户将为实时交易节点定义配置（如下所述），
不一定需要直接使用这些底层组件。
:::

## 示例

你可以在[这里](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/databento/)找到实时示例脚本。

## Databento 文档

Databento 为新用户提供了广泛的文档，可在 [Databento 新用户指南](https://databento.com/docs/quickstart/new-user-guides)中找到。
我们建议结合本 NautilusTrader 集成指南一并参阅 Databento 文档。

## Databento 二进制编码（DBN）

Databento 二进制编码（DBN）是一种极快的消息编码和存储格式，用于标准化（Normalization）的市场数据。
[DBN 规范](https://databento.com/docs/standards-and-conventions/databento-binary-encoding)包含一个简单的自描述元数据头和一组固定的结构体定义，
强制执行标准化市场数据的统一方式。

该集成提供了一个解码器，可以将 DBN 格式数据转换为 Nautilus 对象。

同一个 Rust 实现的 Nautilus 解码器用于：

- 从磁盘加载和解码 DBN 文件。
- 实时解码历史和实时数据。

## 支持的 Schema

以下 Databento schema 受 NautilusTrader 支持：

| Databento schema                                                               | Nautilus 数据类型                  | 描述                            |
|:------------------------------------------------------------------------------|:----------------------------------|:--------------------------------|
| [MBO](https://databento.com/docs/schemas-and-data-formats/mbo)                | `OrderBookDelta`                  | 按订单聚合（L3）。              |
| [MBP_1](https://databento.com/docs/schemas-and-data-formats/mbp-1)            | `(QuoteTick, TradeTick \| None)`  | 按价格聚合（L1）。              |
| [MBP_10](https://databento.com/docs/schemas-and-data-formats/mbp-10)          | `OrderBookDepth10`                | 市场深度（L2）。                |
| [BBO_1S](https://databento.com/docs/schemas-and-data-formats/bbo-1s)          | `QuoteTick`                       | 1 秒最优买卖价。                |
| [BBO_1M](https://databento.com/docs/schemas-and-data-formats/bbo-1m)          | `QuoteTick`                       | 1 分钟最优买卖价。              |
| [CMBP_1](https://databento.com/docs/schemas-and-data-formats/cmbp-1)          | `(QuoteTick, TradeTick \| None)`  | 跨交易场所整合的 MBP。          |
| [CBBO_1S](https://databento.com/docs/schemas-and-data-formats/cbbo-1s)        | `QuoteTick`                       | 整合的 1 秒 BBO。               |
| [CBBO_1M](https://databento.com/docs/schemas-and-data-formats/cbbo-1m)        | `QuoteTick`                       | 整合的 1 分钟 BBO。             |
| [TCBBO](https://databento.com/docs/schemas-and-data-formats/tcbbo)            | `(QuoteTick, TradeTick)`          | 交易采样的整合 BBO。            |
| [TBBO](https://databento.com/docs/schemas-and-data-formats/tbbo)              | `(QuoteTick, TradeTick)`          | 交易采样的最优买卖价。          |
| [TRADES](https://databento.com/docs/schemas-and-data-formats/trades)          | `TradeTick`                       | 成交 tick。                     |
| [OHLCV_1S](https://databento.com/docs/schemas-and-data-formats/ohlcv-1s)      | `Bar`                             | 1 秒 K 线。                     |
| [OHLCV_1M](https://databento.com/docs/schemas-and-data-formats/ohlcv-1m)      | `Bar`                             | 1 分钟 K 线。                   |
| [OHLCV_1H](https://databento.com/docs/schemas-and-data-formats/ohlcv-1h)      | `Bar`                             | 1 小时 K 线。                   |
| [OHLCV_1D](https://databento.com/docs/schemas-and-data-formats/ohlcv-1d)      | `Bar`                             | 日 K 线。                       |
| [OHLCV_EOD](https://databento.com/docs/schemas-and-data-formats/ohlcv-eod)    | `Bar`                             | 收盘 K 线。                     |
| [DEFINITION](https://databento.com/docs/schemas-and-data-formats/definition)  | `Instrument`（多种类型）           | 金融工具定义。                   |
| [IMBALANCE](https://databento.com/docs/schemas-and-data-formats/imbalance)    | `DatabentoImbalance`              | 拍卖不平衡数据。                |
| [STATISTICS](https://databento.com/docs/schemas-and-data-formats/statistics)  | `DatabentoStatistics`             | 市场统计数据。                   |
| [STATUS](https://databento.com/docs/schemas-and-data-formats/status)          | `InstrumentStatus`                | 市场状态更新。                   |

### Schema 选择建议

- **TBBO 和 TCBBO**：交易采样数据源，将每笔交易与交易发生*之前*的 BBO 配对（TBBO 按交易场所，TCBBO 跨交易场所整合）。当你需要将交易与同时期的报价对齐而无需管理两个流时使用。
- **MBP-1 和 CMBP-1（L1）**：事件级更新；仅在交易事件时发送成交数据。适合完整的最优价事件流。对于报价+成交对齐，优先选择 TBBO/TCBBO；否则使用 TRADES。
- **MBP-10（L2）**：前 10 个价格层级加成交数据。适合需要深度感知但不需要逐笔订单细节的策略；比 MBO 轻量，同时具备你需要的大部分结构，包括每个价格层级的订单数量。
- **MBO（L3）**：逐笔订单事件，支持队列位置建模和精确的订单簿重建。数据量/成本最高；在节点初始化时开始订阅以确保正确的回放上下文。
- **BBO_1S/BBO_1M 和 CBBO_1S/CBBO_1M**：固定间隔（1 秒/1 分钟）采样的最优价报价，不含成交数据。最适合监控/价差/低成本信号生成；不适合精细的微观结构分析。
- **TRADES**：仅成交数据。与 MBP-1（`include_trades=True`）配对使用，或使用 TBBO/TCBBO 获取与成交对齐的报价上下文。
- **OHLCV_（包括 OHLCV_EOD）**：从成交数据派生的聚合 K 线。优先用于较长时间框架的分析/回测；确保 K 线时间戳表示收盘时间（设置 `bars_timestamp_on_close=True`）。
- **Imbalance / Statistics / Status**：交易场所运营数据；通过 `subscribe_data` 使用携带 `instrument_id` 元数据的 `DataType` 进行订阅。

:::tip
**整合 schema**（CMBP_1、CBBO_1S、CBBO_1M、TCBBO）将数据跨多个交易场所聚合，
提供统一的市场视图。这对于跨交易场所分析以及需要全面市场画面时特别有用。
:::

:::info
另请参阅 Databento [Schemas and data formats](https://databento.com/docs/schemas-and-data-formats) 指南。
:::

## 实时订阅的 Schema 选择

下表显示了 Nautilus 订阅方法如何映射到 Databento schema：

| Nautilus 订阅方法                  | 默认 Schema  | 可用的 Databento Schema                                                       | Nautilus 数据类型  |
|:----------------------------------|:-------------|:-----------------------------------------------------------------------------|:-------------------|
| `subscribe_quote_ticks()`         | `mbp-1`      | `mbp-1`、`bbo-1s`、`bbo-1m`、`cmbp-1`、`cbbo-1s`、`cbbo-1m`、`tbbo`、`tcbbo` | `QuoteTick`        |
| `subscribe_trade_ticks()`         | `trades`     | `trades`、`tbbo`、`tcbbo`、`mbp-1`、`cmbp-1`                                  | `TradeTick`        |
| `subscribe_order_book_depth()`    | `mbp-10`     | `mbp-10`                                                                     | `OrderBookDepth10` |
| `subscribe_order_book_deltas()`   | `mbo`        | `mbo`                                                                        | `OrderBookDeltas`  |
| `subscribe_bars()`                | 视情况而定    | `ohlcv-1s`、`ohlcv-1m`、`ohlcv-1h`、`ohlcv-1d`                               | `Bar`              |

:::note
以下示例假设你在 `Strategy` 或 `Actor` 类上下文中，其中 `self` 可以访问订阅方法。
记得导入必要的类型：

```python
from nautilus_trader.adapters.databento import DATABENTO_CLIENT_ID
from nautilus_trader.model import BarType
from nautilus_trader.model.enums import BookType
from nautilus_trader.model.identifiers import InstrumentId
```

:::

### 报价订阅（MBP / L1）

```python
# 默认 MBP-1 报价（可能包含成交数据）
self.subscribe_quote_ticks(instrument_id, client_id=DATABENTO_CLIENT_ID)

# 显式 MBP-1 schema
self.subscribe_quote_ticks(
    instrument_id=instrument_id,
    params={"schema": "mbp-1"},
    client_id=DATABENTO_CLIENT_ID,
)

# 1 秒 BBO 快照（仅报价，无成交数据）
self.subscribe_quote_ticks(
    instrument_id=instrument_id,
    params={"schema": "bbo-1s"},
    client_id=DATABENTO_CLIENT_ID,
)

# 跨交易场所整合的报价
self.subscribe_quote_ticks(
    instrument_id=instrument_id,
    params={"schema": "cbbo-1s"},  # 或 "cmbp-1" 用于整合的 MBP
    client_id=DATABENTO_CLIENT_ID,
)

# 交易采样的 BBO（包含报价和成交数据）
self.subscribe_quote_ticks(
    instrument_id=instrument_id,
    params={"schema": "tbbo"},  # 将同时接收 QuoteTick 和 TradeTick 到消息总线
    client_id=DATABENTO_CLIENT_ID,
)
```

### 成交订阅

```python
# 仅成交 tick
self.subscribe_trade_ticks(instrument_id, client_id=DATABENTO_CLIENT_ID)

# 来自 MBP-1 数据源的成交（仅在发生交易事件时）
self.subscribe_trade_ticks(
    instrument_id=instrument_id,
    params={"schema": "mbp-1"},
    client_id=DATABENTO_CLIENT_ID,
)

# 交易采样数据（包含交易时刻的报价）
self.subscribe_trade_ticks(
    instrument_id=instrument_id,
    params={"schema": "tbbo"},  # 同时在交易事件时提供报价
    client_id=DATABENTO_CLIENT_ID,
)
```

### 订单簿深度订阅（MBP / L2）

```python
# 订阅前 10 个层级的市场深度
self.subscribe_order_book_depth(
    instrument_id=instrument_id,
    depth=10  # 自动选择 MBP-10 schema
)

# depth 参数对于 Databento 必须为 10
# 将接收 OrderBookDepth10 更新
```

### 订单簿增量订阅（MBO / L3）

```python
# 订阅完整的订单簿更新（按订单聚合）
self.subscribe_order_book_deltas(
    instrument_id=instrument_id,
    book_type=BookType.L3_MBO  # 使用 MBO schema
)

# 注意：对于 Databento，MBO 订阅必须在节点启动时进行
# 以确保从会话开始时正确回放
```

### K 线订阅

```python
# 订阅 1 分钟 K 线（自动使用 ohlcv-1m schema）
self.subscribe_bars(
    bar_type=BarType.from_str(f"{instrument_id}-1-MINUTE-LAST-EXTERNAL")
)

# 订阅 1 秒 K 线（自动使用 ohlcv-1s schema）
self.subscribe_bars(
    bar_type=BarType.from_str(f"{instrument_id}-1-SECOND-LAST-EXTERNAL")
)

# 订阅 1 小时 K 线（自动使用 ohlcv-1h schema）
self.subscribe_bars(
    bar_type=BarType.from_str(f"{instrument_id}-1-HOUR-LAST-EXTERNAL")
)

# 订阅日 K 线（自动使用 ohlcv-1d schema）
self.subscribe_bars(
    bar_type=BarType.from_str(f"{instrument_id}-1-DAY-LAST-EXTERNAL")
)

# 使用收盘 schema 订阅日 K 线（仅对 DAY 聚合有效）
self.subscribe_bars(
    bar_type=BarType.from_str(f"{instrument_id}-1-DAY-LAST-EXTERNAL"),
    params={"schema": "ohlcv-eod"},  # 覆盖为使用收盘 K 线
)
```

### 自定义数据类型订阅

对于专用的 Databento 数据类型（如 imbalance 和 statistics），使用通用的 `subscribe_data` 方法：

```python
from nautilus_trader.adapters.databento import DATABENTO_CLIENT_ID
from nautilus_trader.adapters.databento import DatabentoImbalance
from nautilus_trader.adapters.databento import DatabentoStatistics
from nautilus_trader.model import DataType

# 订阅 imbalance 数据
self.subscribe_data(
    data_type=DataType(DatabentoImbalance, metadata={"instrument_id": instrument_id}),
    client_id=DATABENTO_CLIENT_ID,
)

# 订阅 statistics 数据
self.subscribe_data(
    data_type=DataType(DatabentoStatistics, metadata={"instrument_id": instrument_id}),
    client_id=DATABENTO_CLIENT_ID,
)

# 订阅金融工具状态更新
from nautilus_trader.model.data import InstrumentStatus
self.subscribe_data(
    data_type=DataType(InstrumentStatus, metadata={"instrument_id": instrument_id}),
    client_id=DATABENTO_CLIENT_ID,
)
```

## 金融工具 ID 和符号体系

Databento 市场数据包含一个 `instrument_id` 字段，这是一个由原始来源交易场所分配或由 Databento 在标准化过程中内部分配的整数。

需要注意的是，这与 Nautilus 的 `InstrumentId` 不同，
后者是由符号 + 交易场所组成的字符串，以句号分隔，即 `"{symbol}.{venue}"`。

Nautilus 解码器将使用 Databento 的 `raw_symbol` 作为 Nautilus 的 `symbol`，并使用来自 Databento 金融工具定义消息的 [ISO 10383 MIC](https://www.iso20022.org/market-identifier-codes)（市场标识码）作为 Nautilus 的 `venue`。

Databento 数据集通过*数据集 ID* 标识，这与交易场所标识符不同。你可以在[这里](https://databento.com/docs/api-reference-historical/basics/datasets)阅读更多关于 Databento 数据集命名约定的信息。

特别需要注意的是 CME Globex MDP 3.0 数据（`GLBX.MDP3` 数据集 ID），以下交易所都归入 `GLBX` 交易场所下。这些映射可以从金融工具的 `exchange` 字段确定：

- `CBCM`：XCME-XCBT 跨交易所价差
- `NYUM`：XNYM-DUMX 跨交易所价差
- `XCBT`：芝加哥期货交易所（CBOT）
- `XCEC`：商品交易中心（COMEX）
- `XCME`：芝加哥商品交易所（CME）
- `XFXS`：CME FX Link 价差
- `XNYM`：纽约商品交易所（NYMEX）

:::info
其他交易场所 MIC 可以在 [metadata.list_publishers](https://databento.com/docs/api-reference-historical/metadata/metadata-list-publishers) 端点响应的 `venue` 字段中找到。
:::

## 时间戳

Databento 数据包含多种时间戳字段，包括（但不限于）：

- `ts_event`：撮合引擎接收时间戳，表示为自 UNIX 纪元以来的纳秒数。
- `ts_in_delta`：撮合引擎发送时间戳，表示为 `ts_recv` 之前的纳秒数。
- `ts_recv`：采集服务器接收时间戳，表示为自 UNIX 纪元以来的纳秒数。
- `ts_out`：Databento 发送时间戳。

Nautilus 数据至少包含两个时间戳（由 `Data` 契约要求）：

- `ts_event`：数据事件发生的 UNIX 时间戳（纳秒）。
- `ts_init`：数据实例创建的 UNIX 时间戳（纳秒）。

在从 Databento 解码和标准化为 Nautilus 时，我们通常将 Databento 的 `ts_recv` 值赋给 Nautilus 的 `ts_event` 字段，因为该时间戳更加可靠和一致，并且保证按金融工具单调递增。
例外是 `DatabentoImbalance` 和 `DatabentoStatistics` 数据类型，它们包含所有时间戳的字段，因为这些类型是专门为适配器定义的。

:::info
更多信息请参阅以下 Databento 文档：

- [Databento standards and conventions - timestamps](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#timestamps)
- [Databento timestamping guide](https://databento.com/docs/architecture/timestamping-guide)

:::

## 数据类型

以下部分讨论 Databento schema 到 Nautilus 数据类型的等价关系和注意事项。

:::info
参见 Databento [schemas and data formats](https://databento.com/docs/schemas-and-data-formats)。
:::

### 金融工具定义

Databento 提供单一 schema 来覆盖所有金融工具类别，这些被解码为相应的 Nautilus `Instrument` 类型。

以下 Databento 金融工具类别受 NautilusTrader 支持：

| Databento 金融工具类别    | 代码  | Nautilus 金融工具类型         |
|--------------------------|-------|------------------------------|
| Stock                    | `K`   | `Equity`                     |
| Future                   | `F`   | `FuturesContract`            |
| Call                     | `C`   | `OptionContract`             |
| Put                      | `P`   | `OptionContract`             |
| Future spread            | `S`   | `FuturesSpread`              |
| Option spread            | `T`   | `OptionSpread`               |
| Mixed spread             | `M`   | `OptionSpread`               |
| FX spot                  | `X`   | `CurrencyPair`               |
| Bond                     | `B`   | 尚未支持                      |

### MBO（按订单聚合）

这是 Databento 提供的最高粒度数据，代表完整的订单簿深度。某些消息还提供成交信息，因此在解码 MBO 消息时，Nautilus 将产生一个 `OrderBookDelta`，以及可选的一个 `TradeTick`。

Nautilus 实时数据客户端将缓冲 MBO 消息，直到看到 `F_LAST` 标志。然后将一个离散的 `OrderBookDeltas` 容器对象传递给已注册的处理器。

订单簿快照也被缓冲到离散的 `OrderBookDeltas` 容器对象中，这在回放启动序列期间发生。

### MBP-1（按价格聚合，最优价）

此 schema 仅代表最优价（报价*和*成交）。与 MBO 消息类似，某些消息携带成交信息，因此在解码 MBP-1 消息时，Nautilus 将产生一个 `QuoteTick`，如果消息是成交，*还会*产生一个 `TradeTick`。

### TBBO 和 TCBBO（最优价加成交）

TBBO（带成交的最优报价）和 TCBBO（带成交的整合最优报价）schema 在每条消息中同时提供报价和成交数据。使用这些 schema 订阅报价时，你将自动接收 `QuoteTick` 和 `TradeTick` 数据，比分别订阅报价和成交更高效。TCBBO 提供跨交易场所整合的数据。

### OHLCV（K 线聚合）

Databento K 线聚合消息的时间戳位于 K 线间隔的**开盘**时刻。
Nautilus 解码器将 `ts_event` 时间戳标准化到 K 线的**收盘**时刻（原始 `ts_event` + K 线间隔）。

### Imbalance 和 Statistics

Databento 的 `imbalance` 和 `statistics` schema 无法表示为内置的 Nautilus 数据类型，因此在 Rust 中定义了特定类型 `DatabentoImbalance` 和 `DatabentoStatistics`。
Python 绑定通过 PyO3（Rust）提供，因此这些类型的行为与内置 Nautilus 数据类型略有不同，其中所有属性都是 PyO3 提供的对象，不直接兼容可能期望 Cython 提供类型的某些方法。可以使用 PyO3 -> 遗留 Cython 对象转换方法，这些方法可在 API 参考中找到。

以下是将 PyO3 `Price` 转换为 Cython `Price` 的通用模式：

```python
price = Price.from_raw(pyo3_price.raw, pyo3_price.precision)
```

此外，请求和订阅这些数据类型需要使用自定义数据类型的底层通用方法。以下示例订阅 `AAPL.XNAS` 金融工具（在纳斯达克交易所交易的 Apple Inc）的 `imbalance` schema：

```python
from nautilus_trader.adapters.databento import DATABENTO_CLIENT_ID
from nautilus_trader.adapters.databento import DatabentoImbalance
from nautilus_trader.model import DataType

instrument_id = InstrumentId.from_str("AAPL.XNAS")
self.subscribe_data(
    data_type=DataType(DatabentoImbalance, metadata={"instrument_id": instrument_id}),
    client_id=DATABENTO_CLIENT_ID,
)
```

或者请求前一天的 `ES.FUT` 父符号（CME Globex 交易所上所有活跃的 E-mini S&P 500 期货合约）的 `statistics` schema：

```python
from nautilus_trader.adapters.databento import DATABENTO_CLIENT_ID
from nautilus_trader.adapters.databento import DatabentoStatistics
from nautilus_trader.model import DataType

instrument_id = InstrumentId.from_str("ES.FUT.GLBX")
metadata = {
    "instrument_id": instrument_id,
    "start": "2024-03-06",
}
self.request_data(
    data_type=DataType(DatabentoStatistics, metadata=metadata),
    client_id=DATABENTO_CLIENT_ID,
)
```

## 性能考虑

使用 Databento DBN 数据进行回测时，有两种选择：

- 将数据存储为 DBN（`.dbn.zst`）格式文件，每次运行时解码为 Nautilus 对象。
- 将 DBN 文件一次性转换为 Nautilus 对象并写入数据目录（以 Nautilus Parquet 格式存储在磁盘上）。

虽然 DBN -> Nautilus 解码器使用 Rust 实现并已优化，
但回测的最佳性能通过将 Nautilus 对象写入数据目录来实现，这只执行一次解码步骤。

[DataFusion](https://arrow.apache.org/datafusion/) 提供了一个查询引擎后端，可高效加载和流式传输磁盘上的 Nautilus Parquet 数据，实现极高的吞吐量（比每次回测运行时将 DBN -> Nautilus 实时转换快至少一个数量级）。

:::note
性能基准测试目前正在开发中。
:::

## 加载 DBN 数据

你可以使用 `DatabentoDataLoader` 类加载 DBN 文件并将记录转换为 Nautilus 对象。这样做有两个主要目的：

- 将转换后的数据直接传递给 `BacktestEngine.add_data` 用于回测。
- 将转换后的数据传递给 `ParquetDataCatalog.write_data` 供后续与 `BacktestNode` 流式使用。

### DBN 数据到 BacktestEngine

此代码片段演示了如何加载 DBN 数据并传递给 `BacktestEngine`。
由于 `BacktestEngine` 需要添加金融工具，我们将使用 `TestInstrumentProvider` 提供的测试金融工具（你也可以传递从 DBN 文件解析的金融工具对象）。
数据是一个月的 TSLA（特斯拉公司）在纳斯达克交易所的成交数据：

```python
# 添加金融工具
TSLA_NASDAQ = TestInstrumentProvider.equity(symbol="TSLA")
engine.add_instrument(TSLA_NASDAQ)

# 解码数据为遗留 Cython 对象
loader = DatabentoDataLoader()
trades = loader.from_dbn_file(
    path=TEST_DATA_DIR / "databento" / "temp" / "tsla-xnas-20240107-20240206.trades.dbn.zst",
    instrument_id=TSLA_NASDAQ.id,
)

# 添加数据
engine.add_data(trades)
```

### DBN 数据到 ParquetDataCatalog

此代码片段演示了如何加载 DBN 数据并写入 `ParquetDataCatalog`。
我们为 `as_legacy_cython` 标志传递 false 值，这将确保 DBN 记录被解码为 PyO3（Rust）对象。值得注意的是，遗留 Cython 对象也可以传递给 `write_data`，但这些需要在底层转换回 pyo3 对象（因此传递 PyO3 对象是一种优化）。

### 加载金融工具

**重要**：将市场数据（MBO、trades、quotes、bars 等）加载到目录时，必须首先从 DEFINITION schema 文件加载相应的金融工具定义。
目录需要金融工具存在后才能存储市场数据。市场数据文件（MBO、TRADES 等）不包含金融工具定义。

```python
# 初始化目录接口
# （将使用 `NAUTILUS_PATH` 环境变量作为路径）
catalog = ParquetDataCatalog.from_env()

loader = DatabentoDataLoader()

# 步骤 1：首先加载金融工具定义
# 你必须从 Databento 获取你的金融工具的 DEFINITION schema 文件
instruments = loader.from_dbn_file(
    path=TEST_DATA_DIR / "databento" / "temp" / "tsla-xnas-definition.dbn.zst",
    as_legacy_cython=False,  # 使用 PyO3 以获得最佳性能
)

# 将金融工具写入目录
catalog.write_data(instruments)

# 步骤 2：现在加载并写入市场数据
instrument_id = InstrumentId.from_str("TSLA.XNAS")

# 解码成交数据为 pyo3 对象
trades = loader.from_dbn_file(
    path=TEST_DATA_DIR / "databento" / "temp" / "tsla-xnas-20240107-20240206.trades.dbn.zst",
    instrument_id=instrument_id,
    as_legacy_cython=False,  # 这是写入目录的优化
)

# 写入市场数据
catalog.write_data(trades)
```

#### 为回测加载多种数据类型

在为具有多种数据类型（例如 MBO 订单簿数据）的回测准备目录时，始终先加载金融工具：

```python
from nautilus_trader.adapters.databento.loaders import DatabentoDataLoader
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.persistence.catalog import ParquetDataCatalog

catalog = ParquetDataCatalog.from_env()
loader = DatabentoDataLoader()

# 步骤 1：从 DEFINITION 文件加载金融工具定义
instruments = loader.from_dbn_file(
    path="equity-definitions.dbn.zst",
    as_legacy_cython=False,
)
catalog.write_data(instruments)

# 步骤 2：加载市场数据（MBO、trades、quotes 等）
instrument_id = InstrumentId.from_str("AAPL.XNAS")

# 加载 MBO 订单簿增量
deltas = loader.from_dbn_file(
    path="aapl-mbo.dbn.zst",
    instrument_id=instrument_id,  # 可选但可提高性能
    as_legacy_cython=False,
)
catalog.write_data(deltas)

# 加载成交数据
trades = loader.from_dbn_file(
    path="aapl-trades.dbn.zst",
    instrument_id=instrument_id,
    as_legacy_cython=False,
)
catalog.write_data(trades)

# 验证金融工具已在目录中
print(catalog.instruments())  # 应显示你加载的金融工具
```

:::tip
你可以通过调用 `catalog.instruments()` 来验证金融工具是否正确加载，它返回目录中所有金融工具的列表。如果返回空列表，你需要先加载 DEFINITION 文件。
:::

:::info
要从 Databento 获取 DEFINITION schema 文件，请使用 Databento API 或 CLI 下载你的符号和日期范围的金融工具定义。
详情请参阅 [Databento 文档](https://databento.com/docs/api-reference-historical/timeseries/timeseries-get-range)了解如何请求定义数据。
:::

:::info
另请参阅[数据概念指南](../concepts/data.md)。
:::

### 历史加载器选项

`from_dbn_file` 方法支持几个重要参数：

- `instrument_id`：传递此参数可通过跳过符号查找来提高解码速度。
- `price_precision`：覆盖金融工具的默认价格精度。
- `include_trades`：对于 MBP-1/CMBP-1 schema，设置为 `True` 将在存在成交数据时同时发送 `QuoteTick` 和 `TradeTick` 对象。
- `as_legacy_cython`：加载 IMBALANCE 或 STATISTICS schema 时设置为 `False`（必需），或在写入目录时设置为 `False` 以提高性能。

:::warning
IMBALANCE 和 STATISTICS schema 要求 `as_legacy_cython=False`，因为这些是仅 PyO3 的类型。设置 `as_legacy_cython=True` 将引发 `ValueError`。
:::

### 加载整合数据

整合 schema 跨多个交易场所聚合数据：

```python
# 加载整合的 MBP-1 报价
loader = DatabentoDataLoader()
cmbp_quotes = loader.from_dbn_file(
    path="consolidated.cmbp-1.dbn.zst",
    instrument_id=InstrumentId.from_str("AAPL.XNAS"),
    include_trades=True,  # 如果可用，同时获取报价和成交数据
    as_legacy_cython=True,
)

# 加载整合的 BBO 报价
cbbo_quotes = loader.from_dbn_file(
    path="consolidated.cbbo-1s.dbn.zst",
    instrument_id=InstrumentId.from_str("AAPL.XNAS"),
    as_legacy_cython=False,  # 使用 PyO3 以获得更好性能
)

# 加载 TCBBO（交易采样整合 BBO）- 同时提供报价和成交
# 注意：include_trades=True 加载报价，include_trades=False 加载成交
tcbbo_quotes = loader.from_dbn_file(
    path="consolidated.tcbbo.dbn.zst",
    instrument_id=InstrumentId.from_str("AAPL.XNAS"),
    include_trades=True,  # 加载报价
    as_legacy_cython=True,
)

tcbbo_trades = loader.from_dbn_file(
    path="consolidated.tcbbo.dbn.zst",
    instrument_id=InstrumentId.from_str("AAPL.XNAS"),
    include_trades=False,  # 加载成交
    as_legacy_cython=True,
)
```

:::tip
**成本优化**：避免对同一金融工具同时订阅 TBBO/TCBBO 和单独的成交订阅，因为这些 schema 已包含成交数据。这可以防止重复并降低成本。
:::

## 实时客户端架构

`DatabentoDataClient` 是一个 Python 类，包含其他 Databento 适配器类。
每个 Databento 数据集有两个 `DatabentoLiveClient`：

- 一个用于 MBO（订单簿增量）实时数据源
- 一个用于所有其他实时数据源

:::warning
目前有一个限制，即一个数据集的所有 MBO（订单簿增量）订阅必须在节点启动时进行，
然后才能从会话开始回放数据。如果之后有新的订阅到达，将记录错误（并忽略该订阅）。

对于任何其他 Databento schema 没有此限制。
:::

一个 `DatabentoHistoricalClient` 实例在 `DatabentoInstrumentProvider` 和 `DatabentoDataClient` 之间共享，
用于进行历史金融工具定义和数据请求。

## 配置

最常见的用例是配置一个实时 `TradingNode` 以包含 Databento 数据客户端。为此，在你的客户端配置中添加 `DATABENTO` 部分：

```python
from nautilus_trader.adapters.databento import DATABENTO
from nautilus_trader.live.node import TradingNode

config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        DATABENTO: {
            "api_key": None,  # 'DATABENTO_API_KEY' 环境变量
            "http_gateway": None,  # 覆盖默认的 HTTP 历史网关
            "live_gateway": None,  # 覆盖默认的原始 TCP 实时网关
            "instrument_provider": InstrumentProviderConfig(load_all=True),
            "instrument_ids": None,  # 启动时加载的 Nautilus 金融工具 ID
            "parent_symbols": None,  # 启动时加载的 Databento 父符号
        },
    },
    ..., # 省略
)
```

然后，创建一个 `TradingNode` 并添加客户端工厂：

```python
from nautilus_trader.adapters.databento.factories import DatabentoLiveDataClientFactory
from nautilus_trader.live.node import TradingNode

# 使用配置实例化实时交易节点
node = TradingNode(config=config)

# 向节点注册客户端工厂
node.add_data_client_factory(DATABENTO, DatabentoLiveDataClientFactory)

# 最后构建节点
node.build()
```

### 配置参数

Databento 数据客户端提供以下配置选项：

| 选项                      | 默认值   | 描述 |
|---------------------------|---------|------|
| `api_key`                 | `None`  | Databento API 密钥。为 `None` 时回退到 `DATABENTO_API_KEY` 环境变量。 |
| `http_gateway`            | `None`  | 历史 HTTP 网关覆盖，用于测试自定义端点。 |
| `live_gateway`            | `None`  | 原始 TCP 实时网关覆盖，通常仅用于测试。 |
| `use_exchange_as_venue`   | `True`  | 如果为 `True`，使用交易所 MIC 作为 Nautilus 交易场所（例如 `XCME`）。为 `False` 时保留默认的 GLBX 映射。 |
| `timeout_initial_load`    | `15.0`  | 等待每个数据集的金融工具定义加载完成的秒数。 |
| `mbo_subscriptions_delay` | `3.0`   | 启用 MBO/L3 流之前的缓冲秒数，以便初始快照可以按顺序回放。 |
| `bars_timestamp_on_close` | `True`  | 在收盘时为 K 线打时间戳（`ts_event`/`ts_init`）。设置为 `False` 则在开盘时打时间戳。 |
| `reconnect_timeout_mins`  | `10`    | 放弃之前尝试重连的分钟数。设置为 `None` 则无限重试（谨慎使用）。参见下方[连接稳定性](#连接稳定性)。 |
| `venue_dataset_map`       | `None`  | Nautilus 交易场所到 Databento 数据集代码的可选映射。 |
| `parent_symbols`          | `None`  | 可选映射 `{dataset: {parent symbols}}`，用于预加载定义树（例如 `{"GLBX.MDP3": {"ES.FUT", "ES.OPT"}}`）。 |
| `instrument_ids`          | `None`  | 启动时预加载定义的 Nautilus `InstrumentId` 值序列。 |
| `http_proxy_url`          | `None`  | 可选的 HTTP 代理 URL。 |
| `ws_proxy_url`            | `None`  | 可选的 WebSocket 代理 URL。 |

:::tip
我们建议使用环境变量来管理你的凭证。
:::

### 连接稳定性

Databento 实时客户端实现了自动重连以处理连接中断。系统通过以下方式保持弹性：

- **网络中断**：临时的连接问题。
- **网关重启**：Databento 每周日执行定期维护（参见[维护计划](https://databento.com/docs/api-reference-live/basics#maintenance-schedule)）。
- **市场收盘**：非交易时段的会话结束。

#### 重连策略

客户端根据超时配置使用不同的退避策略：

**有超时**（默认 10 分钟）：

- 指数退避上限 **60 秒**，实现快速恢复。
- 模式：1 秒、2 秒、4 秒、8 秒、16 秒、32 秒、60 秒、60 秒...（+/- 1 秒抖动）。
- 优化为在超时窗口内快速重连。

**无超时**（`reconnect_timeout_mins=None`）：

- 指数退避上限 **10 分钟**，实现耐心的、对基础设施友好的恢复。
- 模式：1 秒、2 秒、4 秒、8 秒、16 秒、32 秒、64 秒、128 秒、256 秒、512 秒、600 秒、600 秒...（+/- 1 秒抖动）。
- 适合无人值守系统在夜间收盘和定期维护期间持续运行。

所有重连包括：

- **抖动**：随机延迟（最多 1 秒），防止同步重连风暴。
- **自动重新订阅**：重连后恢复所有活跃订阅。
- **周期重置**：每次成功会话（>60 秒）重置超时计时器。

#### 超时配置

`reconnect_timeout_mins` 参数控制客户端尝试重连的时长：

**默认（10 分钟）**：适合大多数使用场景。

- 处理瞬态网络问题。
- 安全度过计划中的网关重启。
- 防止在市场收盘期间浪费资源。
- 较长时间的中断需要手动干预。

:::warning
设置 `reconnect_timeout_mins=None` 会导致无限重试。仅用于必须在无人值守情况下安全度过夜间市场收盘的系统。这可能掩盖持续性的配置或认证问题。
:::

#### 定期维护

Databento 每周日在以下时间重启其实时网关（所有客户端将断开连接）：

| 数据集              | 维护时间（UTC）    |
|--------------------|-------------------|
| CME Globex         | 09:30             |
| 所有 ICE 交易场所   | 09:45             |
| 所有其他数据集      | 10:30             |

默认的 10 分钟超时可处理典型的维护重启。对于在维护窗口期间运行的无人值守系统，考虑使用 `reconnect_timeout_mins=None` 或更长的超时。详情请参阅 [Databento Maintenance Schedule](https://databento.com/docs/api-reference-live/basics/maintenance-schedule)。

:::info
如需了解更多功能或为 Databento 适配器做出贡献，请参阅我们的[贡献指南](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::
