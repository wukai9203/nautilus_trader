# Kraken

Kraken 成立于 2011 年，是全球最成熟的加密货币交易所之一，也是欧洲欧元交易量最大的交易所。该平台提供广泛数字资产的现货（Spot）和衍生品（Derivatives）交易。本集成（Integration）连接到 Kraken Pro，支持 Kraken 现货和 Kraken 衍生品（期货）市场的实时市场数据接入和订单执行（Execution）。

## 概述

该适配器（Adapter）使用 Rust 实现，提供 Python 绑定以便在基于 Python 的工作流中使用。它不需要外部 Kraken 客户端库——核心组件编译为静态库并在构建过程中自动链接。

本指南假设交易者正在设置实时市场数据推送和交易执行。Kraken 适配器包含多个组件，可以根据使用场景组合或单独使用。

- `KrakenRawHttpClient`：现货和期货的底层 HTTP API 连接。
- `KrakenHttpClient`：带有金融工具（Instrument）缓存和对账支持的高层 HTTP 客户端。
- `KrakenInstrumentProvider`：金融工具解析和加载功能。
- `KrakenDataClient`：市场数据推送管理器。
- `KrakenExecutionClient`：账户管理和交易执行网关。
- `KrakenLiveDataClientFactory`：Kraken 数据客户端工厂（由交易节点构建器使用）。
- `KrakenLiveExecClientFactory`：Kraken 执行客户端工厂（由交易节点构建器使用）。

:::note
大多数用户只需为实时交易节点定义配置（如下所示），无需直接使用这些底层组件。
:::

## 示例

您可以在[此处](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/kraken/)找到实时示例脚本。

## Kraken 文档

Kraken 为用户提供了详尽的文档：

- [Kraken API 文档](https://docs.kraken.com/api/)
- [Kraken 现货 REST API](https://docs.kraken.com/api/docs/guides/spot-rest-intro)
- [Kraken 期货 REST API](https://docs.kraken.com/api/docs/futures-api)

请结合本 NautilusTrader 集成指南参考 Kraken 文档。

## 产品

Kraken 支持两个主要产品类别：

| 产品类型                | 支持 | 备注                                                    |
|------------------------|------|--------------------------------------------------------|
| 现货                    | ✓   | 支持保证金（Margin）的标准加密货币交易对。                   |
| 期货（永续）             | ✓   | 反向（`PI_`）和 USD 保证金（`PF_`）永续合约。              |
| 期货（交割/弹性）        | ✓   | 固定到期（`FI_`）和弹性（`FF_`）合约。                     |

:::note
**双产品部署**：当同时配置 `SPOT` 和 `FUTURES` 产品类型时，适配器会查询两个 API 并合并账户状态。这确保执行引擎能够查看两个市场的抵押品。
:::

## K 线数据流

### 支持的时间间隔

Kraken 适配器通过 WebSocket 支持现货市场的实时 K 线（OHLC）数据流。以下时间间隔可用：

| 时间间隔    | BarType 规格           |
|------------|------------------------|
| 1 分钟     | `1-MINUTE-LAST`        |
| 5 分钟     | `5-MINUTE-LAST`        |
| 15 分钟    | `15-MINUTE-LAST`       |
| 30 分钟    | `30-MINUTE-LAST`       |
| 1 小时     | `1-HOUR-LAST`          |
| 4 小时     | `4-HOUR-LAST`          |
| 1 天       | `1-DAY-LAST`           |
| 1 周       | `1-WEEK-LAST`          |
| 15 天      | `15-DAY-LAST`          |

:::note
**期货限制**：Kraken 期货不支持通过 WebSocket 进行 K 线数据流推送。请改用 `request_bars()` 获取历史 K 线数据。
:::

### K 线发射延迟

Kraken 的 WebSocket OHLC 频道在每笔交易时推送*当前*（未完成的）K 线更新。与某些交易所（如 Binance）不同，Kraken 不提供 "is_closed" 指标来表示 K 线已完成。

为避免发射部分/未完成的 K 线，适配器缓冲当前 K 线，仅在下一个 K 线周期开始时（即收到具有新 `interval_begin` 时间戳的消息时）发射它。这意味着：

- K 线的发射延迟最多为一个 K 线周期。
- 对于 1 分钟 K 线，最大延迟约为 1 分钟。
- 发射的 K 线数据是完整且最终的。

我们选择这种方式而非基于定时器的发射，因为：

- 基于定时器的发射可能错过 K 线关闭前的最后一次更新。
- Kraken 的更新不保证在精确的间隔边界到达。
- 缓冲以延迟为代价确保数据完整性。

:::warning
如果 K 线延迟对您的策略至关重要，请考虑使用逐笔成交数据并使用 `BarAggregator` 在本地聚合 K 线。
:::

:::tip
对于大多数使用场景，我们建议使用 `INTERNAL` K 线聚合（订阅逐笔成交并在本地聚合 K 线）而非 `EXTERNAL` 交易所提供的 K 线：

- K 线完成后立即发射，无缓冲延迟。
- 跨所有交易所行为一致，简化多交易场所策略。

:::

## 符号体系

### 现货市场

NautilusTrader 对 Kraken 现货金融工具符号使用 ISO 4217-A3 格式，提供跨交易所的标准化表示。适配器在内部处理到 Kraken 原生格式的转换。

**金融工具 ID 格式：**

```python
InstrumentId.from_str("BTC/USD.KRAKEN")   # 现货 BTC/USD
InstrumentId.from_str("ETH/USD.KRAKEN")   # 现货 ETH/USD
InstrumentId.from_str("SOL/USD.KRAKEN")   # 现货 SOL/USD
InstrumentId.from_str("BTC/USDT.KRAKEN")  # 现货 BTC/USDT
```

:::note
Kraken 的原生 API 使用不同的资产代码（例如，比特币使用 `XBT`，ETH/USD 使用 `XETHZUSD`）。适配器自动在 NautilusTrader 的标准化格式和 Kraken 的原生格式之间进行转换。
:::

### 期货市场

Kraken 期货金融工具使用带有前缀的特定命名约定：

- `PI_` - 永续反向合约（例如 `PI_XBTUSD`）
- `PF_` - 永续固定保证金合约（例如 `PF_XBTUSD`）
- `FI_` - 固定到期反向合约（例如 `FI_XBTUSD_230929`）
- `FF_` - 弹性期货合约

**金融工具 ID 格式：**

```python
InstrumentId.from_str("PI_XBTUSD.KRAKEN")  # 永续反向 BTC
InstrumentId.from_str("PI_ETHUSD.KRAKEN")  # 永续反向 ETH
InstrumentId.from_str("PF_XBTUSD.KRAKEN")  # 永续固定保证金 BTC
```

## 订单功能

### 订单类型

| 订单类型               | 现货 | 期货 | 备注                                            |
|------------------------|------|------|------------------------------------------------|
| `MARKET`               | ✓    | ✓   | 以市场价格立即执行。                               |
| `LIMIT`                | ✓    | ✓   | 以指定价格或更优价格执行。                          |
| `STOP_MARKET`          | ✓    | ✓   | 条件市价单（止损）。                               |
| `MARKET_IF_TOUCHED`    | ✓    | ✓   | 条件市价单（止盈）。                               |
| `STOP_LIMIT`           | ✓    | ✓   | 条件限价单（止损限价）。                            |
| `LIMIT_IF_TOUCHED`     | ✓    | -   | *期货：尚未实现*。                                 |

### 有效时间

| 有效时间 | 现货 | 期货 | 备注                                               |
|---------|------|------|---------------------------------------------------|
| `GTC`   | ✓    | ✓   | 撤销前有效（Good Till Canceled）。                    |
| `GTD`   | ✓    | -   | 到期前有效（仅现货，需要 `expire_time`）。              |
| `IOC`   | ✓    | ✓   | 立即成交或撤销（Immediate or Cancel）。               |
| `FOK`   | -    | -   | *Kraken 不支持*。                                   |

:::note
**市价单**本质上是立即执行的，不支持有效时间设置。`IOC` 仅适用于限价类订单。
:::

### 执行指令

| 指令           | 现货 | 期货 | 备注                                       |
|---------------|------|------|--------------------------------------------|
| `post_only`   | ✓    | ✓   | 适用于限价单。                                |
| `reduce_only` | -    | ✓   | 仅期货。减少持仓，不反转。                      |

### 批量操作

| 操作          | 现货 | 期货 | 备注                                        |
|--------------|------|------|---------------------------------------------|
| 批量提交      | -    | -   | *尚未实现*。                                  |
| 批量修改      | -    | -   | *尚未实现*（仅期货）。                          |
| 批量取消      | ✓    | ✓   | 自动分块为每批 50 个。                          |

:::note
**取消所有订单**：

- 不支持按订单方向筛选；无论方向如何，所有订单都将被取消。
- 现货：取消所有交易对的所有未结订单。
- 期货：需要 `instrument_id`；仅取消该交易对的订单。

:::

### 持仓管理

| 功能          | 现货 | 期货 | 备注                                                     |
|--------------|------|------|----------------------------------------------------------|
| 查询持仓      | ✓*   | ✓   | *现货：需通过 `use_spot_position_reports` 启用。见下文。     |
| 持仓模式      | -    | -   | 每个金融工具单一持仓。                                      |
| 杠杆控制      | -    | ✓   | 按账户等级配置。                                            |
| 保证金模式    | -    | ✓   | 期货使用全仓保证金。                                        |

### 订单查询

| 功能             | 现货 | 期货 | 备注                                        |
|-----------------|------|------|---------------------------------------------|
| 查询未结订单     | ✓    | ✓   | 列出所有活动订单。                              |
| 查询历史订单     | ✓    | ✓   | 支持分页的历史订单数据。                         |
| 订单状态更新     | ✓    | ✓   | 通过 WebSocket 实时更新订单状态。                |
| 交易历史         | ✓    | ✓   | 成交和填充报告。                                |

### 条件订单

| 功能            | 现货 | 期货 | 备注                                    |
|----------------|------|------|-----------------------------------------|
| 订单列表        | -    | -   | *不支持*。                                |
| OCO 订单       | -    | -   | *不支持*。                                |
| 组合订单        | -    | -   | *不支持*。                                |
| 条件订单        | ✓    | ✓   | 止损和止盈订单。                           |

## 对账

Kraken 适配器为现货和期货市场提供全面的对账（Reconciliation）功能，允许交易者在启动时或运行期间将本地状态与交易所状态同步。

### 现货对账

**订单状态报告：**

- 未结订单：获取所有当前活动的订单。
- 已关闭订单：获取支持分页的历史订单。
- 时间范围查询：支持按开始/结束时间戳筛选。

**成交报告：**

- 交易历史：获取支持分页的成交历史。
- 时间范围查询：支持按开始/结束时间戳筛选。
- 所有成交类型：市价单、限价单和条件订单成交。

### 期货对账

**订单状态报告：**

- 未结订单：获取所有当前活动的期货订单。
- 历史订单：当 `open_only=False` 时获取已关闭和已成交的订单。
- 订单事件：通过 `/api/history/v2/orders` 端点获取完整的订单生命周期历史。

**成交报告：**

- 成交历史：获取所有成交报告。
- 时间筛选：按开始/结束时间戳进行客户端筛选（解析 RFC3339 时间戳）。
- 所有成交类型：包含手续费信息的 Maker 和 Taker 成交。

**持仓状态报告：**

- 未结持仓：获取所有活动的期货持仓。
- 实时数据：包含未实现资金费用、平均价格和持仓规模。

:::note
**期货时间筛选**：Kraken 期货成交端点不支持服务端时间范围筛选。适配器通过解析 `fillTime` 字段并与请求的开始/结束时间戳进行比较来实现客户端筛选。
:::

### 现货持仓报告

Kraken 适配器可以选择性地将钱包余额报告为现货金融工具的持仓状态报告。此功能默认禁用，必须通过配置显式启用。

**工作原理：**

- 启用后，钱包余额被转换为 `PositionStatusReport` 对象。
- 正余额报告为多头（`LONG`）持仓。
- 仅报告与配置的报价货币匹配的金融工具（默认：`USDT`）。
- 这防止了同一资产在多个报价货币下出现重复报告（例如 BTC/USD、BTC/USDT、BTC/EUR）。

**配置：**

```python
exec_clients={
    KRAKEN: {
        "use_spot_position_reports": True,
        "spot_positions_quote_currency": "USDT",  # 默认值
    },
}
```

:::warning
**谨慎使用**：启用现货持仓报告可能导致意外行为，如果您的策略未设计为处理现货持仓。例如，预期平仓的策略可能会尝试卖出您的钱包持有量。
:::

## 速率限制

适配器实现了自动速率限制以符合 Kraken 的 API 要求。

| 端点类型             | 限制（请求/秒） | 备注                                |
|---------------------|----------------|-------------------------------------|
| 现货 REST（全局）    | 5              | 现货 API 的全局速率限制。              |
| 期货 REST（全局）    | 5              | 期货 API 的全局速率限制。              |

:::info
Kraken 使用基于计数器的速率限制系统，限制因等级而异：

- **入门等级**：最大计数器 15，衰减 -0.33/秒
- **中级等级**：最大计数器 20，衰减 -0.5/秒
- **专业等级**：最大计数器 20，衰减 -1/秒

账本/交易历史调用计数器增加 +2；其他调用增加 +1。
:::

:::warning
Kraken 可能会临时封锁超出速率限制的 IP 地址。适配器会在接近限制时自动排队请求。
:::

## 配置

每个客户端的产品类型必须在配置中指定。

### 数据客户端配置选项

| 选项                              | 默认值     | 描述                                                             |
|---------------------------------|-----------|------------------------------------------------------------------|
| `api_key`                       | `None`    | API 密钥；省略时从环境变量加载（见下文）。                            |
| `api_secret`                    | `None`    | API 密钥；省略时从环境变量加载（见下文）。                            |
| `environment`                   | `mainnet` | 交易环境（`mainnet` 或 `demo`）；demo 仅适用于期货。                 |
| `product_types`                 | `(SPOT,)` | 产品类型元组（例如 `(KrakenProductType.SPOT,)`）。                  |
| `base_url_http_spot`            | `None`    | Kraken 现货 REST 基础 URL 覆盖。                                  |
| `base_url_http_futures`         | `None`    | Kraken 期货 REST 基础 URL 覆盖。                                  |
| `base_url_ws_spot`              | `None`    | Kraken 现货 WebSocket URL 覆盖。                                  |
| `base_url_ws_futures`           | `None`    | Kraken 期货 WebSocket URL 覆盖。                                  |
| `http_proxy_url`                | `None`    | 可选的 HTTP 代理 URL。                                            |
| `ws_proxy_url`                  | `None`    | WebSocket 代理 URL（*尚未实现*）。                                  |
| `update_instruments_interval_mins` | `60`   | 重新加载金融工具的间隔（分钟）；设为 `None` 可禁用。                  |
| `max_retries`                   | `None`    | REST 请求的最大重试次数。                                          |
| `retry_delay_initial_ms`        | `None`    | 重试间的初始延迟（毫秒）。                                          |
| `retry_delay_max_ms`            | `None`    | 重试间的最大延迟（毫秒）。                                          |
| `http_timeout_secs`             | `None`    | HTTP 请求超时时间（秒）。                                          |
| `ws_heartbeat_secs`             | `30`      | WebSocket 心跳间隔（秒）。                                        |
| `max_requests_per_second`       | `None`    | 覆盖速率限制（默认 5 请求/秒）；适用于更高等级账户。                   |

### 执行客户端配置选项

| 选项                              | 默认值     | 描述                                                             |
|---------------------------------|-----------|------------------------------------------------------------------|
| `api_key`                       | `None`    | API 密钥；省略时从环境变量加载（见下文）。                            |
| `api_secret`                    | `None`    | API 密钥；省略时从环境变量加载（见下文）。                            |
| `environment`                   | `mainnet` | 交易环境（`mainnet` 或 `demo`）；demo 仅适用于期货。                 |
| `product_types`                 | `(SPOT,)` | 产品类型元组；`SPOT` 使用现金账户，`FUTURES` 使用保证金账户。          |
| `base_url_http_spot`            | `None`    | Kraken 现货 REST 基础 URL 覆盖。                                  |
| `base_url_http_futures`         | `None`    | Kraken 期货 REST 基础 URL 覆盖。                                  |
| `base_url_ws_spot`              | `None`    | Kraken 现货 WebSocket URL 覆盖。                                  |
| `base_url_ws_futures`           | `None`    | Kraken 期货 WebSocket URL 覆盖。                                  |
| `http_proxy_url`                | `None`    | 可选的 HTTP 代理 URL。                                            |
| `ws_proxy_url`                  | `None`    | WebSocket 代理 URL（*尚未实现*）。                                  |
| `max_retries`                   | `None`    | 订单提交/取消调用的最大重试次数。                                    |
| `retry_delay_initial_ms`        | `None`    | 重试间的初始延迟（毫秒）。                                          |
| `retry_delay_max_ms`            | `None`    | 重试间的最大延迟（毫秒）。                                          |
| `http_timeout_secs`             | `None`    | HTTP 请求超时时间（秒）。                                          |
| `ws_heartbeat_secs`             | `30`      | WebSocket 心跳间隔（秒）。                                        |
| `max_requests_per_second`       | `None`    | 覆盖速率限制（默认 5 请求/秒）；适用于更高等级账户。                   |
| `use_spot_position_reports`     | `False`   | 将钱包余额报告为持仓（见下文）。                                     |
| `spot_positions_quote_currency` | `"USDT"`  | 现货持仓报告的报价货币筛选。                                        |

### 模拟环境设置

要使用 Kraken 期货模拟（模拟交易）进行测试：

1. 在 [https://demo-futures.kraken.com](https://demo-futures.kraken.com) 注册并生成 API 凭证。
2. 使用您的模拟凭证设置环境变量：
   - `KRAKEN_FUTURES_DEMO_API_KEY`
   - `KRAKEN_FUTURES_DEMO_API_SECRET`
3. 使用 `environment=KrakenEnvironment.DEMO` 和 `product_types=(KrakenProductType.FUTURES,)` 配置适配器。

```python
from nautilus_trader.adapters.kraken import KRAKEN
from nautilus_trader.adapters.kraken import KrakenEnvironment
from nautilus_trader.adapters.kraken import KrakenProductType

config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        KRAKEN: {
            "environment": KrakenEnvironment.DEMO,
            "product_types": (KrakenProductType.FUTURES,),
        },
    },
    exec_clients={
        KRAKEN: {
            "environment": KrakenEnvironment.DEMO,
            "product_types": (KrakenProductType.FUTURES,),
        },
    },
)
```

### 生产配置

最常见的使用场景是配置实时 `TradingNode` 以包含 Kraken 数据和执行客户端。将 `KRAKEN` 部分添加到您的客户端配置中：

```python
from nautilus_trader.adapters.kraken import KRAKEN
from nautilus_trader.adapters.kraken import KrakenEnvironment
from nautilus_trader.adapters.kraken import KrakenProductType
from nautilus_trader.live.node import TradingNode

config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        KRAKEN: {
            "environment": KrakenEnvironment.MAINNET,
            "product_types": (KrakenProductType.SPOT,),
        },
    },
    exec_clients={
        KRAKEN: {
            "environment": KrakenEnvironment.MAINNET,
            "product_types": (KrakenProductType.SPOT,),
        },
    },
)
```

### 双产品配置（现货 + 期货）

当同时交易现货和期货市场时，包含两种产品类型：

```python
config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        KRAKEN: {
            "environment": KrakenEnvironment.MAINNET,
            "product_types": (KrakenProductType.SPOT, KrakenProductType.FUTURES),
        },
    },
    exec_clients={
        KRAKEN: {
            "environment": KrakenEnvironment.MAINNET,
            "product_types": (KrakenProductType.SPOT, KrakenProductType.FUTURES),
        },
    },
)
```

然后，创建 `TradingNode` 并添加客户端工厂：

```python
from nautilus_trader.adapters.kraken import KRAKEN
from nautilus_trader.adapters.kraken import KrakenLiveDataClientFactory
from nautilus_trader.adapters.kraken import KrakenLiveExecClientFactory
from nautilus_trader.live.node import TradingNode

# 使用配置实例化实时交易节点
node = TradingNode(config=config)

# 向节点注册客户端工厂
node.add_data_client_factory(KRAKEN, KrakenLiveDataClientFactory)
node.add_exec_client_factory(KRAKEN, KrakenLiveExecClientFactory)

# 最后构建节点
node.build()
```

### API 凭证

有两种方式向 Kraken 客户端提供凭证。将对应的 `api_key` 和 `api_secret` 值传递给配置对象，或设置以下环境变量：

| 环境变量                           | 描述                              |
|-----------------------------------|-----------------------------------|
| `KRAKEN_SPOT_API_KEY`             | Kraken 现货的 API 密钥（主网）。     |
| `KRAKEN_SPOT_API_SECRET`          | Kraken 现货的 API 密钥（主网）。     |
| `KRAKEN_FUTURES_API_KEY`          | Kraken 期货的 API 密钥（主网）。     |
| `KRAKEN_FUTURES_API_SECRET`       | Kraken 期货的 API 密钥（主网）。     |
| `KRAKEN_FUTURES_DEMO_API_KEY`     | Kraken 期货的 API 密钥（模拟）。     |
| `KRAKEN_FUTURES_DEMO_API_SECRET`  | Kraken 期货的 API 密钥（模拟）。     |

:::note
**模拟环境**：只有 Kraken 期货提供模拟环境（`https://demo-futures.kraken.com`）用于无真实资金的测试。Kraken 现货没有测试网——`environment` 设置仅影响期货连接。
:::

:::tip
我们建议使用环境变量来管理您的凭证。
:::

启动交易节点时，您将立即收到凭证是否有效以及是否具有交易权限的确认。

## 贡献

:::info
如需额外功能或为 Kraken 适配器做贡献，请参阅我们的
[贡献指南](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::
