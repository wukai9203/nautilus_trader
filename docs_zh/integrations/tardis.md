# Tardis

Tardis 为加密货币市场提供细粒度数据，包括逐笔订单簿快照和更新、
成交、未平仓合约、资金费率、期权链和清算数据，覆盖领先的加密交易所。

NautilusTrader 提供与 Tardis API 和数据格式的集成（Integration），实现无缝访问。
此适配器（Adapter）的功能包括：

- `TardisCSVDataLoader`：读取 Tardis 格式的 CSV 文件并将其转换为 Nautilus 数据，支持批量加载和内存高效的流式处理。
- `TardisMachineClient`：支持从 Tardis Machine WebSocket 服务器进行实时流式传输和历史回放（Replay），将消息转换为 Nautilus 数据。
- `TardisHttpClient`：从 Tardis HTTP API 请求金融工具（Instrument）定义元数据，并将其解析为 Nautilus 金融工具定义。
- `TardisDataClient`：提供实时数据客户端，用于订阅来自 Tardis Machine WebSocket 服务器的数据流。
- `TardisInstrumentProvider`：通过 HTTP 金融工具元数据 API 从 Tardis 提供金融工具定义。
- **数据管道函数**：支持从 Tardis Machine 回放历史数据并将其写入 Nautilus Parquet 格式，包括直接目录集成以简化数据管理（见下文）。

:::info
适配器正常运行需要 Tardis API 密钥（API Key）。另请参阅[环境变量](#环境变量)。
:::

## 概述

此适配器使用 Rust 实现，并提供可选的 Python 绑定以便在基于 Python 的工作流中使用。
它不需要任何外部 Tardis 客户端库依赖。

:::info
**无需**为 `tardis` 执行额外的安装步骤。
适配器的核心组件被编译为静态库，并在构建过程中自动链接。
:::

## Tardis 文档

Tardis 提供了广泛的用户[文档](https://docs.tardis.dev/)。
我们建议结合本 NautilusTrader 集成指南一并参阅 Tardis 文档。

## 支持的格式

Tardis 提供*标准化*（Normalized）的市场数据——一种在所有支持的交易所之间保持一致的统一格式。
这种标准化非常有价值，因为它允许单个解析器处理来自任何 [Tardis 支持的交易所](#交易场所)的数据，减少了开发时间和复杂性。
因此，NautilusTrader 不会支持交易所原生市场数据格式，因为在当前阶段为每个交易所实现单独的解析器效率较低。

以下 Tardis 标准化格式受 NautilusTrader 支持：

| Tardis 格式                                                                                                                  | Nautilus 数据类型                                                     |
|:----------------------------------------------------------------------------------------------------------------------------|:---------------------------------------------------------------------|
| [book_change](https://docs.tardis.dev/api/tardis-machine#book_change)                                                       | `OrderBookDelta`                                                     |
| [book_snapshot_*](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit) | `OrderBookDepth10` 或 `OrderBookDeltas`（参见[订单簿快照输出](#订单簿快照输出)） |
| [quote](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit)           | `QuoteTick`                                                          |
| [quote_10s](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit)       | `QuoteTick`                                                          |
| [trade](https://docs.tardis.dev/api/tardis-machine#trade)                                                                   | `Trade`                                                              |
| [trade_bar_*](https://docs.tardis.dev/api/tardis-machine#trade_bar_-aggregation_interval-suffix)                            | `Bar`                                                                |
| [instrument](https://docs.tardis.dev/api/instruments-metadata-api)                                                          | `CurrencyPair`、`CryptoFuture`、`CryptoPerpetual`、`OptionContract` |
| [derivative_ticker](https://docs.tardis.dev/api/tardis-machine#derivative_ticker)                                           | *尚未支持*                                                           |
| [disconnect](https://docs.tardis.dev/api/tardis-machine#disconnect)                                                         | *不适用*                                                             |

**说明：**

- [quote](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit) 是 [book_snapshot_1_0ms](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit) 的别名。
- [quote_10s](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit) 是 [book_snapshot_1_10s](https://docs.tardis.dev/api/tardis-machine#book_snapshot_-number_of_levels-_-snapshot_interval-time_unit) 的别名。
- quote、quote_10s 和单层快照均被解析为 `QuoteTick`。

:::info
另请参阅 Tardis [标准化市场数据 API](https://docs.tardis.dev/api/tardis-machine#normalized-market-data-apis)。
:::

## K 线

适配器将自动把 [Tardis 交易 K 线间隔和后缀](https://docs.tardis.dev/api/tardis-machine#trade_bar_-aggregation_interval-suffix)转换为 Nautilus `BarType`。
包括以下内容：

| Tardis 后缀                                                                                                  | Nautilus K 线聚合方式     |
|:-------------------------------------------------------------------------------------------------------------|:--------------------------|
| [ms](https://docs.tardis.dev/api/tardis-machine#trade_bar_-aggregation_interval-suffix) - 毫秒               | `MILLISECOND`             |
| [s](https://docs.tardis.dev/api/tardis-machine#trade_bar_-aggregation_interval-suffix) - 秒                  | `SECOND`                  |
| [m](https://docs.tardis.dev/api/tardis-machine#trade_bar_-aggregation_interval-suffix) - 分钟                | `MINUTE`                  |
| [ticks](https://docs.tardis.dev/api/tardis-machine#trade_bar_-aggregation_interval-suffix) - tick 数量        | `TICK`                    |
| [vol](https://docs.tardis.dev/api/tardis-machine#trade_bar_-aggregation_interval-suffix) - 成交量大小         | `VOLUME`                  |

## 符号体系和标准化

Tardis 集成确保与 NautilusTrader 的加密交易所适配器无缝兼容，
方法是一致地标准化符号。通常，NautilusTrader 使用 Tardis 提供的交易所原生命名约定。但对于某些交易所，原始符号会被调整以符合 Nautilus 符号标准化，如下所述：

### 通用规则

- 所有符号转换为大写。
- 对于某些交易所，市场类型后缀以连字符附加（参见[交易所特定标准化](#交易所特定标准化)）。
- 原始交易所符号保留在 Nautilus 金融工具定义的 `raw_symbol` 字段中。

### 交易所特定标准化

- **Binance**：Nautilus 为所有永续合约符号附加 `-PERP` 后缀。
- **Bybit**：Nautilus 使用特定的产品类别后缀，包括 `-SPOT`、`-LINEAR`、`-INVERSE`、`-OPTION`。
- **dYdX**：Nautilus 为所有永续合约符号附加 `-PERP` 后缀。
- **Gate.io**：Nautilus 为所有永续合约符号附加 `-PERP` 后缀。

各交易所的详细符号体系文档：

- [Binance 符号体系](./binance.md#symbology)
- [Bybit 符号体系](./bybit.md#symbology)
- [dYdX 符号体系](./dydx.md#symbology)

## 交易场所

Tardis 上的一些交易所被划分为多个交易场所（Venue）。
下表列出了 Nautilus 交易场所与相应 Tardis 交易所之间的映射，以及 Tardis 支持的交易所：

| Nautilus 交易场所       | Tardis 交易所                                                  |
|:------------------------|:--------------------------------------------------------------|
| `ASCENDEX`              | `ascendex`                                                    |
| `BINANCE`               | `binance`、`binance-dex`、`binance-european-options`、`binance-futures`、`binance-jersey`、`binance-options` |
| `BINANCE_DELIVERY`      | `binance-delivery`（*币本位合约*）                             |
| `BINANCE_US`            | `binance-us`                                                  |
| `BITFINEX`              | `bitfinex`、`bitfinex-derivatives`                            |
| `BITFLYER`              | `bitflyer`                                                    |
| `BITGET`                | `bitget`、`bitget-futures`                                    |
| `BITMEX`                | `bitmex`                                                      |
| `BITNOMIAL`             | `bitnomial`                                                   |
| `BITSTAMP`              | `bitstamp`                                                    |
| `BLOCKCHAIN_COM`        | `blockchain-com`                                              |
| `BYBIT`                 | `bybit`、`bybit-options`、`bybit-spot`                        |
| `COINBASE`              | `coinbase`                                                    |
| `COINBASE_INTX`         | `coinbase-international`                                      |
| `COINFLEX`              | `coinflex`（*用于历史研究*）                                   |
| `CRYPTO_COM`            | `crypto-com`、`crypto-com-derivatives`                        |
| `CRYPTOFACILITIES`      | `cryptofacilities`                                            |
| `DELTA`                 | `delta`                                                       |
| `DERIBIT`               | `deribit`                                                     |
| `DYDX`                  | `dydx`                                                        |
| `DYDX_V4`               | `dydx-v4`                                                     |
| `FTX`                   | `ftx`、`ftx-us`（*历史研究*）                                  |
| `GATE_IO`               | `gate-io`、`gate-io-futures`                                  |
| `GEMINI`                | `gemini`                                                      |
| `HITBTC`                | `hitbtc`                                                      |
| `HUOBI`                 | `huobi`、`huobi-dm`、`huobi-dm-linear-swap`、`huobi-dm-options` |
| `HUOBI_DELIVERY`        | `huobi-dm-swap`                                               |
| `HYPERLIQUID`           | `hyperliquid`                                                 |
| `KRAKEN`                | `kraken`                                                      |
| `KUCOIN`                | `kucoin`、`kucoin-futures`                                    |
| `MANGO`                 | `mango`                                                       |
| `OKCOIN`                | `okcoin`                                                      |
| `OKEX`                  | `okex`、`okex-futures`、`okex-options`、`okex-spreads`、`okex-swap` |
| `PHEMEX`                | `phemex`                                                      |
| `POLONIEX`              | `poloniex`                                                    |
| `SERUM`                 | `serum`（*历史研究*）                                          |
| `STAR_ATLAS`            | `star-atlas`                                                  |
| `UPBIT`                 | `upbit`                                                       |
| `WOO_X`                 | `woo-x`                                                       |

## 环境变量

以下环境变量由 Tardis 和 NautilusTrader 使用。

- `TM_API_KEY`：Tardis Machine 的 API 密钥。
- `TARDIS_API_KEY`：NautilusTrader Tardis 客户端的 API 密钥。
- `TARDIS_MACHINE_WS_URL`（可选）：NautilusTrader 中 `TardisMachineClient` 的 WebSocket URL。
- `TARDIS_BASE_URL`（可选）：NautilusTrader 中 `TardisHttpClient` 的基础 URL。
- `NAUTILUS_PATH`（可选）：包含 `catalog/` 子目录的父目录，用于以 Nautilus 目录格式写入回放数据。

## 运行 Tardis Machine 历史回放

[Tardis Machine Server](https://docs.tardis.dev/api/tardis-machine) 是一个可本地运行的服务器，
具有内置数据缓存功能，通过 HTTP 和 WebSocket API 提供逐笔历史数据和整合的实时加密货币市场数据。

你可以使用 Python 或 Rust 执行完整的 Tardis Machine WebSocket 历史数据回放，并将结果输出为 Nautilus Parquet 格式。由于该函数使用 Rust 实现，
无论从 Python 还是 Rust 运行，性能都是一致的，你可以根据偏好的工作流进行选择。

端到端的 `run_tardis_machine_replay` 数据管道函数使用指定的[配置](#配置)执行以下步骤：

- 连接到 Tardis Machine 服务器。
- 从 [Tardis 金融工具元数据](https://docs.tardis.dev/api/instruments-metadata-api) HTTP API 请求并解析所有必要的金融工具定义。
- 从 Tardis Machine 服务器流式传输指定时间范围内所有请求的金融工具和数据类型。
- 对于每个金融工具、数据类型和日期（UTC），生成一个目录兼容格式的 `.parquet` 文件。
- 从 Tardis Machine 服务器断开连接，并终止程序。

**文件命名约定**

文件按天、按金融工具写入，使用 ISO 8601 时间戳范围清楚地标明数据的确切时间跨度：

- **格式**：`{start_timestamp}_{end_timestamp}.parquet`
- **示例**：`2023-10-01T00-00-00-000000000Z_2023-10-01T23-59-59-999999999Z.parquet`
- **结构**：`data/{data_type}/{instrument_id}/{filename}`

此格式与 Nautilus 数据目录完全兼容，支持无缝的查询、整合和数据管理操作。

:::note
你可以在不使用 API 密钥的情况下请求每月第一天的数据。对于所有其他日期，需要 Tardis Machine API 密钥。
:::

此过程已针对直接输出到 Nautilus Parquet 数据目录进行了优化。
确保 `NAUTILUS_PATH` 环境变量设置为包含 `catalog/` 子目录的父目录。
Parquet 文件将按照数据类型和金融工具对应的预期子目录组织在 `<NAUTILUS_PATH>/catalog/data/` 下。

如果配置文件中未指定 `output_path` 且 `NAUTILUS_PATH` 环境变量未设置，系统将默认使用当前工作目录。

### 步骤

首先，确保 `tardis-machine` Docker 容器正在运行。使用以下命令：

```bash
docker run -p 8000:8000 -p 8001:8001 -e "TM_API_KEY=YOUR_API_KEY" -d tardisdev/tardis-machine
```

此命令在没有持久本地缓存的情况下启动 `tardis-machine` 服务器，这可能影响性能。
为了提高性能，建议使用持久卷运行服务器。详情请参阅 [Tardis Docker 文档](https://docs.tardis.dev/api/tardis-machine#docker)。

### 配置

接下来，确保你有一个可用的配置 JSON 文件。

**配置 JSON 格式**

| 字段                    | 类型               | 描述                                                                          | 默认值                                                                                                |
|:------------------------|:-------------------|:-----------------------------------------------------------------------------|:------------------------------------------------------------------------------------------------------|
| `tardis_ws_url`         | 字符串（可选）      | Tardis Machine WebSocket URL。                                               | 如果为 `null`，则使用 `TARDIS_MACHINE_WS_URL` 环境变量。                                               |
| `normalize_symbols`     | 布尔值（可选）      | 是否应用 Nautilus [符号标准化](#符号体系和标准化)。                              | 如果为 `null`，则默认为 `true`。                                                                       |
| `output_path`           | 字符串（可选）      | 写入 Nautilus Parquet 数据的输出目录路径。                                     | 如果为 `null`，则使用 `NAUTILUS_PATH` 环境变量，否则使用当前工作目录。                                    |
| `book_snapshot_output`  | 字符串（可选）      | `book_snapshot_*` 数据的输出格式：`"deltas"` 或 `"depth10"`。参见[订单簿快照输出](#订单簿快照输出)。 | 如果为 `null`，则默认为 `"deltas"`。                                                                   |
| `ws_proxy_url`          | 字符串（可选）      | 可选的 WebSocket 代理 URL。                                                   | 如果为 `null`，则不使用代理。                                                                          |
| `options`               | JSON[]             | [ReplayNormalizedRequestOptions](https://docs.tardis.dev/api/tardis-machine#replay-normalized-options) 对象数组。                                                                  |

示例配置文件 `example_config.json` 可在[这里](https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/adapters/tardis/bin/example_config.json)找到：

```json
{
  "tardis_ws_url": "ws://localhost:8001",
  "output_path": null,
  "options": [
    {
      "exchange": "bitmex",
      "symbols": [
        "xbtusd",
        "ethusd"
      ],
      "data_types": [
        "trade"
      ],
      "from": "2019-10-01",
      "to": "2019-10-02"
    }
  ]
}
```

### 订单簿快照输出

`book_snapshot_output` 配置选项控制 Tardis `book_snapshot_*` 消息（例如 `book_snapshot_5_100ms`、`book_snapshot_10_1s`）的转换和存储方式。

| 值        | Nautilus 类型       | 输出目录              | 描述                                                        |
|:----------|:--------------------|:----------------------|:-----------------------------------------------------------|
| `deltas`  | `OrderBookDeltas`   | `order_book_deltas/`  | 设置了快照标志的单个价格层级更新（默认）                       |
| `depth10` | `OrderBookDepth10`  | `order_book_depths/`  | 包含最多 10 个价格层级的定期深度快照                          |

**何时使用每种格式：**

- **`deltas`（默认）**：当你需要重建完整的订单簿状态或与 `book_change` 数据配合使用时最佳。每个价格层级成为一条单独的增量记录。
- **`depth10`**：当策略需要定期的订单簿快照时最佳。更节省内存，因为每个快照是包含所有层级的单条记录。超过 10 个层级的快照将只保留前 10 个。

**避免文件覆盖：**

当为同一金融工具和日期范围同时下载 `book_snapshot_*` 和 `book_change` 数据时，使用 `depth10` 格式可确保它们被写入不同的目录（`order_book_depths/` 与 `order_book_deltas/`），防止文件覆盖。

使用显式格式的配置示例：

```json
{
  "tardis_ws_url": "ws://localhost:8001",
  "book_snapshot_output": "depth10",
  "options": [
    {
      "exchange": "binance-futures",
      "symbols": ["btcusdt"],
      "data_types": ["book_snapshot_5_100ms", "book_change"],
      "from": "2024-01-01",
      "to": "2024-01-02"
    }
  ]
}
```

### Python 回放

要在 Python 中运行回放，创建类似以下的脚本：

```python
import asyncio

from nautilus_trader.core import nautilus_pyo3


async def run():
    config_filepath = Path("YOUR_CONFIG_FILEPATH")
    await nautilus_pyo3.run_tardis_machine_replay(str(config_filepath.resolve()))


if __name__ == "__main__":
    asyncio.run(run())
```

### Rust 回放

要在 Rust 中运行回放，创建类似以下的二进制文件：

```rust
use std::{env, path::PathBuf};

use nautilus_adapters::tardis::replay::run_tardis_machine_replay_from_config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config_filepath = PathBuf::from("YOUR_CONFIG_FILEPATH");
    run_tardis_machine_replay_from_config(&config_filepath).await;
}
```

确保通过导出以下环境变量来启用 Rust 日志：

```bash
export RUST_LOG=debug
```

可在[这里](https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/adapters/tardis/bin/example_replay.rs)找到一个可运行的示例二进制文件。

也可以使用 cargo 运行：

```bash
cargo run --bin tardis-replay <path_to_your_config>
```

## 加载 Tardis CSV 数据

Tardis 格式的 CSV 数据可以使用 Python 或 Rust 加载。加载器从磁盘读取 CSV 文本数据并将其解析为 Nautilus 数据。由于加载器使用 Rust 实现，无论从 Python 还是 Rust 运行，性能都保持一致，你可以根据偏好的工作流进行选择。

你还可以选择性地为 `load_*` 函数/方法指定 `limit` 参数来控制加载的最大行数。

:::note
加载混合金融工具的 CSV 文件由于精度要求较为困难，不建议使用。请改用单一金融工具的 CSV 文件（见下文）。
:::

### 在 Python 中加载 CSV 数据

你可以使用 `TardisCSVDataLoader` 在 Python 中加载 Tardis 格式的 CSV 数据。
加载数据时，你可以选择性地指定金融工具 ID，但必须同时指定价格精度和数量精度。
提供金融工具 ID 可以提高加载性能，而指定精度是必需的，因为无法从文本数据中推断。

创建类似以下的脚本来加载数据：

```python
from nautilus_trader.adapters.tardis import TardisCSVDataLoader
from nautilus_trader.model import InstrumentId


instrument_id = InstrumentId.from_str("BTC-PERPETUAL.DERIBIT")
loader = TardisCSVDataLoader(
    price_precision=1,
    size_precision=0,
    instrument_id=instrument_id,
)

filepath = Path("YOUR_CSV_DATA_PATH")
limit = None

deltas = loader.load_deltas(filepath, limit)
```

### 在 Rust 中加载 CSV 数据

你可以使用[这里](https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/adapters/tardis/src/csv/mod.rs)找到的加载函数在 Rust 中加载 Tardis 格式的 CSV 数据。
加载数据时，你可以选择性地指定金融工具 ID，但必须同时指定价格精度和数量精度。
提供金融工具 ID 可以提高加载性能，而指定精度是必需的，因为无法从文本数据中推断。

完整示例请参阅[此示例二进制文件](https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/adapters/tardis/bin/example_csv.rs)。

使用类似以下的代码来加载数据：

```rust
use std::path::Path;

use nautilus_adapters::tardis;
use nautilus_model::identifiers::InstrumentId;

#[tokio::main]
async fn main() {
    // 必须指定精度和 CSV 文件路径
    let price_precision = 1;
    let size_precision = 0;
    let filepath = Path::new("YOUR_CSV_DATA_PATH");

    // 可选地指定金融工具 ID 和/或限制数
    let instrument_id = InstrumentId::from("BTC-PERPETUAL.DERIBIT");
    let limit = None;

    // 根据你的工作流考虑传播任何解析错误
    let _deltas = tardis::csv::load_deltas(
        filepath,
        price_precision,
        size_precision,
        Some(instrument_id),
        limit,
    )
    .unwrap();
}
```

## 流式 Tardis CSV 数据

对于大型 CSV 文件的内存高效处理，Tardis 集成提供了流式功能，以可配置的块大小加载和处理数据，而不是一次性将整个文件加载到内存中。这对于处理数 GB 的 CSV 文件而不耗尽系统内存特别有用。

流式功能适用于所有支持的 Tardis 数据类型：

- 订单簿增量（`stream_deltas`）。
- 报价 tick（`stream_quotes`）。
- 成交 tick（`stream_trades`）。
- 订单簿深度快照（`stream_depth10`）。

### 在 Python 中流式 CSV 数据

`TardisCSVDataLoader` 提供流式方法，以迭代器形式产生数据块。每个方法接受一个 `chunk_size` 参数，控制每次从 CSV 文件读取多少条记录：

```python
from nautilus_trader.adapters.tardis import TardisCSVDataLoader
from nautilus_trader.model import InstrumentId

instrument_id = InstrumentId.from_str("BTC-PERPETUAL.DERIBIT")
loader = TardisCSVDataLoader(
    price_precision=1,
    size_precision=0,
    instrument_id=instrument_id,
)

filepath = Path("large_trades_file.csv")
chunk_size = 100_000  # 每块处理 100,000 条记录（默认值）

# 按块流式处理成交 tick
for chunk in loader.stream_trades(filepath, chunk_size):
    print(f"Processing chunk with {len(chunk)} trades")
    # 处理每个块——只有当前块在内存中
    for trade in chunk:
        # 你的处理逻辑
        pass
```

### 流式订单簿数据

订单簿数据的流式处理适用于增量和深度快照：

```python
# 流式订单簿增量
for chunk in loader.stream_deltas(filepath):
    print(f"Processing {len(chunk)} deltas")
    # 处理增量块

# 流式 depth10 快照（指定层数：5 或 25）
for chunk in loader.stream_depth10(filepath, levels=5):
    print(f"Processing {len(chunk)} depth snapshots")
    # 处理深度块
```

### 流式报价数据

报价数据可以类似地流式处理：

```python
# 流式报价 tick
for chunk in loader.stream_quotes(filepath):
    print(f"Processing {len(chunk)} quotes")
    # 处理报价块
```

### 内存效率优势

流式处理方法提供了显著的内存效率优势：

- **可控的内存使用**：一次只加载一个块到内存中。
- **可扩展的处理**：可以处理大于可用 RAM 的文件。
- **可配置的块大小**：根据系统内存和性能需求调整 `chunk_size`（默认 100,000）。

:::warning
当使用精度推断进行流式处理（未提供显式精度）时，推断的精度可能与批量加载整个文件不同。
这是因为精度推断在块边界内工作，不同的块可能包含具有不同精度要求的值。
为了确定性的精度行为，请在调用流式方法时提供显式的 `price_precision` 和 `size_precision` 参数。
:::

### 在 Rust 中流式 CSV 数据

底层流式功能使用 Rust 实现，可以直接使用：

```rust
use std::path::Path;
use nautilus_adapters::tardis::csv::{stream_trades, stream_deltas};
use nautilus_model::identifiers::InstrumentId;

#[tokio::main]
async fn main() {
    let filepath = Path::new("large_trades_file.csv");
    let chunk_size = 100_000;
    let price_precision = Some(1);
    let size_precision = Some(0);
    let instrument_id = Some(InstrumentId::from("BTC-PERPETUAL.DERIBIT"));

    // 按块流式处理成交
    let stream = stream_trades(
        filepath,
        chunk_size,
        price_precision,
        size_precision,
        instrument_id,
    ).unwrap();

    for chunk_result in stream {
        match chunk_result {
            Ok(chunk) => {
                println!("Processing chunk with {} trades", chunk.len());
                // 处理块
            }
            Err(e) => {
                eprintln!("Error processing chunk: {}", e);
                break;
            }
        }
    }
}
```

## 请求金融工具定义

你可以使用 `TardisHttpClient` 在 Python 和 Rust 中请求金融工具定义。
此客户端与 [Tardis 金融工具元数据 API](https://docs.tardis.dev/api/instruments-metadata-api) 交互，请求并解析金融工具元数据为 Nautilus 金融工具。

`TardisHttpClient` 构造函数接受可选参数 `api_key`、`base_url` 和 `timeout_secs`（默认为 60 秒）。

客户端提供方法来获取特定的 `instrument` 或特定交易所上所有可用的 `instruments`。
确保在引用 [Tardis 支持的交易所](https://api.tardis.dev/v1/exchanges)时使用 Tardis 的小写连字符命名约定。

:::note
访问金融工具元数据 API 需要 Tardis API 密钥。
:::

### 在 Python 中请求金融工具

创建类似以下的脚本在 Python 中请求金融工具定义：

```python
import asyncio

from nautilus_trader.core import nautilus_pyo3


async def run():
    http_client = nautilus_pyo3.TardisHttpClient()

    instrument = await http_client.instrument("bitmex", "xbtusd")
    print(f"Received: {instrument}")

    instruments = await http_client.instruments("bitmex")
    print(f"Received: {len(instruments)} instruments")


if __name__ == "__main__":
    asyncio.run(run())
```

### 在 Rust 中请求金融工具

使用类似以下的代码在 Rust 中请求金融工具定义。
完整示例请参阅[此示例二进制文件](https://github.com/nautechsystems/nautilus_trader/blob/develop/crates/adapters/tardis/bin/example_http.rs)。

```rust
use nautilus_adapters::tardis::{enums::Exchange, http::client::TardisHttpClient};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let client = TardisHttpClient::new(None, None, None).unwrap();

    // Nautilus 金融工具定义
    let resp = client.instruments(Exchange::Bitmex).await;
    println!("Received: {resp:?}");

    let resp = client.instrument(Exchange::Bitmex, "ETHUSDT").await;
    println!("Received: {resp:?}");
}
```

## 金融工具提供者

`TardisInstrumentProvider` 通过 HTTP 金融工具元数据 API 从 Tardis 请求和解析金融工具定义。
由于有多个 [Tardis 支持的交易所](#交易场所)，在加载所有金融工具时，
必须使用 `InstrumentProviderConfig` 筛选所需的交易场所：

```python
from nautilus_trader.config import InstrumentProviderConfig

# 参见支持的交易场所 https://nautilustrader.io/docs/nightly/integrations/tardis#venues
venues = {"BINANCE", "BYBIT"}
filters = {"venues": frozenset(venues)}
instrument_provider_config = InstrumentProviderConfig(load_all=True, filters=filters)
```

你也可以按常规方式加载特定的金融工具定义：

```python
from nautilus_trader.config import InstrumentProviderConfig

instrument_ids = [
    InstrumentId.from_str("BTCUSDT-PERP.BINANCE"),  # 将使用 'binance-futures' 交易所
    InstrumentId.from_str("BTCUSDT.BINANCE"),  # 将使用 'binance' 交易所
]
instrument_provider_config = InstrumentProviderConfig(load_ids=instrument_ids)
```

### 期权交易所过滤

金融工具提供者在 `instrument_type` 过滤器未提供或不包含 `"option"` 时，会自动过滤掉期权专用交易所（如 `binance-options`、`binance-european-options`、`bybit-options`、`okex-options` 和 `huobi-dm-options`）。

要显式加载期权金融工具，请在 `instrument_type` 过滤器中包含 `"option"`：

```python
from nautilus_trader.config import InstrumentProviderConfig

venues = {"BINANCE", "BYBIT"}
filters = {
    "venues": frozenset(venues),
    "instrument_type": {"option"},  # 显式请求期权
}
instrument_provider_config = InstrumentProviderConfig(load_all=True, filters=filters)
```

此过滤机制在不需要期权交易所时避免不必要的 API 调用，提高性能并减少 API 使用。

:::note
所有订阅都需要金融工具在缓存中可用。
为简单起见，建议为你打算订阅的交易场所加载所有金融工具。
:::

## 实时数据客户端

`TardisDataClient` 支持将 Tardis Machine 与运行中的 NautilusTrader 系统集成。
它支持订阅以下数据类型：

- `OrderBookDelta`（来自 Tardis 的 L2 粒度，包括所有变更或全深度快照）
- `OrderBookDepth10`（来自 Tardis 的 L2 粒度，提供最多 10 个层级的快照）
- `QuoteTick`
- `TradeTick`
- `Bar`（具有 [Tardis 支持的 K 线聚合](#K-线)的交易 K 线）

### 数据 WebSocket

主 `TardisMachineClient` 数据 WebSocket 管理初始连接阶段期间接收的所有流订阅，
直到 `ws_connection_delay_secs` 指定的持续时间。对于此期间之后的任何额外订阅，
将创建一个新的 `TardisMachineClient`。这种方法通过允许主 WebSocket 在单个流中处理可能数百个订阅（如果它们在启动时提供）来优化性能。

当使用 `ws_connection_delay_secs` 设置初始订阅延迟时，从这些流中取消订阅实际上不会从 Tardis Machine 流中移除订阅，因为 Tardis 不支持选择性取消订阅。但是，该组件仍会按预期从消息总线发布中取消订阅。

在任何初始延迟之后进行的所有订阅将正常运行，在请求时完全从 Tardis Machine 流中取消订阅。

:::tip
如果你预计会频繁订阅和取消订阅数据，建议将 `ws_connection_delay_secs` 设置为零。这将为每个初始订阅创建一个新客户端，允许它们在取消订阅时单独关闭。
:::

## 限制和注意事项

目前已知以下限制和注意事项：

- 不支持历史数据请求，因为每个请求至少需要从 Tardis Machine 进行一天的回放，可能还需要过滤。这种方法既不实用也不高效。

:::info
如需了解更多功能或为 Tardis 适配器做出贡献，请参阅我们的[贡献指南](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::
