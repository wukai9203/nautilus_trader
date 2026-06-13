# Binance

Binance 成立于 2017 年，是全球最大的加密货币交易所之一，以日交易量和加密资产及加密衍生品的未平仓合约量衡量。

本集成（integration）支持以下产品的实时市场数据接入和订单执行（execution）：

- **Binance Spot**（包括 Binance US）
- **Binance USDT 保证金期货（USDT-Margined Futures）**（永续合约和交割合约）
- **Binance 币本位期货（Coin-Margined Futures）**

## 示例

你可以在[此处](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/binance/)找到实盘示例脚本。

## 概览

本指南假设交易者正在设置实时市场数据源和交易执行。
Binance 适配器（adapter）包含多个组件，可根据使用场景组合或单独使用。

- `BinanceHttpClient`：底层 HTTP API 连接。
- `BinanceWebSocketClient`：底层 WebSocket API 连接。
- `BinanceInstrumentProvider`：金融工具（instrument）解析和加载功能。
- `BinanceSpotDataClient`/`BinanceFuturesDataClient`：市场数据源管理器。
- `BinanceSpotExecutionClient`/`BinanceFuturesExecutionClient`：账户管理和交易执行网关。
- `BinanceLiveDataClientFactory`：Binance 数据客户端工厂（由交易节点构建器使用）。
- `BinanceLiveExecClientFactory`：Binance 执行客户端工厂（由交易节点构建器使用）。

:::note
大多数用户只需为实盘交易节点定义配置（configuration）（如下所示），无需直接使用这些底层组件。
:::

### 产品支持

| 产品类型                             | 支持 | 备注                               |
|------------------------------------------|-----------|-------------------------------------|
| 现货市场（Spot Markets，含 Binance US）          | ✓         |                                     |
| 保证金账户（Margin Accounts，全仓和逐仓）       | -         | 保证金交易尚未实现。     |
| USDT 保证金期货（永续和交割）  | ✓         |                                     |
| 币本位期货                    | ✓         |                                     |

:::note
保证金交易（全仓和逐仓）目前尚未实现。
欢迎通过 [GitHub issue #2631](https://github.com/nautechsystems/nautilus_trader/issues/#2631) 或提交 pull request 来添加保证金交易功能。
:::

:::info 产品类型说明：保证金账户 vs. 期货合约

这三类产品在 Binance 体系中是完全独立的账户和 API，本质上有根本区别：

**保证金账户（Margin Account）= 现货市场的杠杆借贷**

你借入代币，在**现货市场**直接买卖真实资产。你实际持有代币，到期需归还借款并支付**按小时计算的利息**。

- **全仓保证金（Cross Margin）**：账户内所有资产共同作为抵押品，一个仓位爆仓会波及所有持仓。
- **逐仓保证金（Isolated Margin）**：每个交易对独立设置抵押品上限，爆仓只损失分配给该仓位的资金。

**USDT 保证金期货（USDT-M Futures）= 以 USDT 结算的衍生品合约**

你持有的是"涨跌赔付合约"，而非真实代币。保证金和盈亏均以 **USDT** 计价结算。持仓成本为**资金费率**（多空双方定期互付）而非借款利息。

**币本位期货（Coin-M Futures）= 以加密货币本身结算的衍生品合约**

与 USDT-M 期货逻辑相同，但保证金和盈亏以 **BTC / ETH 等加密货币**计价结算。常见于矿工或长期持币者，无需将资产换成 USDT 即可对冲风险。

| 维度 | 保证金账户 | USDT-M 期货 | 币本位期货 |
|------|-----------|------------|-----------|
| 产品性质 | 现货 + 借贷 | 衍生品合约 | 衍生品合约 |
| 是否持有真实资产 | 是 | 否 | 否 |
| 保证金 / 结算货币 | 借入的代币 | USDT | BTC / ETH 等 |
| 持仓成本 | 借款利息（按小时） | 资金费率 | 资金费率 |
| NautilusTrader 支持 | ❌ 未实现 | ✅ | ✅ |
:::

## 数据类型

为了向交易者提供完整的 API 功能，本集成包含若干自定义数据（data）类型：

- `BinanceTicker`：表示 Binance 24 小时行情订阅返回的数据，包含全面的价格和统计信息。
- `BinanceBar`：表示 Binance K 线的历史请求或实时订阅数据，附带额外的成交量指标。
- `BinanceFuturesMarkPriceUpdate`：表示 Binance Futures 订阅的标记价格更新。

完整定义请参阅 Binance [API 参考](../api_reference/adapters/binance.md)。

## 交易代码规则

按照 Nautilus 统一的交易代码策略，在可能的情况下使用 Binance 原生交易代码，包括现货资产和期货合约。由于 NautilusTrader 支持多交易场所（venue）+ 多账户交易，因此有必要明确区分作为现货和保证金交易对的 `BTCUSDT` 与 `BTCUSDT` 永续期货合约（Binance 原生系统中两者使用*相同*的代码）。

因此，Nautilus 为所有永续合约代码添加 `-PERP` 后缀。
例如，在 Binance Futures 中，`BTCUSDT` 永续期货合约在 Nautilus 系统边界内的代码为 `BTCUSDT-PERP`。

## 订单能力

以下表格详细说明了不同 Binance 账户类型支持的订单（order）类型、执行指令和有效期选项：

### 订单类型

| 订单类型             | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                   |
|------------------------|------|--------|--------------|--------------|-------------------------|
| `MARKET`               | ✓    | ✓      | ✓            | ✓            | 报价数量支持：仅限现货/保证金。 |
| `LIMIT`                | ✓    | ✓      | ✓            | ✓            |                         |
| `STOP_MARKET`          | -    | ✓      | ✓            | ✓            | 现货不支持。 |
| `STOP_LIMIT`           | ✓    | ✓      | ✓            | ✓            |                         |
| `MARKET_IF_TOUCHED`    | -    | -      | ✓            | ✓            | 仅限期货。           |
| `LIMIT_IF_TOUCHED`     | ✓    | ✓      | ✓            | ✓            |                         |
| `TRAILING_STOP_MARKET` | -    | -      | ✓            | ✓            | 仅限期货。           |

### 执行指令

| 指令   | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                 |
|---------------|------|--------|--------------|--------------|---------------------------------------|
| `post_only`   | ✓    | ✓      | ✓            | ✓            | 请参阅下方限制条件。               |
| `reduce_only` | -    | -      | ✓            | ✓            | 仅限期货；对冲模式下禁用。 |

#### Post-only 限制

仅*限价*订单类型支持 `post_only`。

| 订单类型               | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                                      |
|--------------------------|------|--------|--------------|--------------|------------------------------------------------------------|
| `LIMIT`                  | ✓    | ✓      | ✓            | ✓            | 现货/保证金使用 `LIMIT_MAKER`，期货使用 `GTX` TIF。 |
| `STOP_LIMIT`             | -    | -      | ✓            | ✓            | 现货/保证金不支持。                             |

### 有效期

| 有效期 | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                           |
|---------------|------|--------|--------------|--------------|-------------------------------------------------|
| `GTC`         | ✓    | ✓      | ✓            | ✓            | 撤销前有效（Good Till Canceled）。                             |
| `GTD`         | ✓*   | ✓*     | ✓            | ✓            | *现货/保证金会转换为 GTC 并发出警告。 |
| `FOK`         | ✓    | ✓      | ✓            | ✓            | 全部成交或取消（Fill or Kill）。                                   |
| `IOC`         | ✓    | ✓      | ✓            | ✓            | 立即成交或取消（Immediate or Cancel）。                            |

### 高级订单功能

| 功能            | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                        |
|--------------------|------|--------|--------------|--------------|----------------------------------------------|
| 订单修改 | ✓    | ✓      | ✓            | ✓            | 仅支持 `LIMIT` 订单的价格和数量修改。  |
| Bracket/OCO 订单 | ✓    | ✓      | ✓            | ✓            | 用于止损/止盈的一取消另一（One-Cancels-Other）订单。 |
| 冰山订单     | ✓    | ✓      | ✓            | ✓            | 大额订单拆分为可见部分。    |

### 批量操作

| 操作          | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                        |
|--------------------|------|--------|--------------|--------------|----------------------------------------------|
| 批量提交       | ✓    | ✓      | ✓            | ✓            | 单次请求提交多个订单。    |
| 批量修改       | -    | -      | ✓            | ✓            | 单次请求修改多个订单。仅限期货。 |
| 批量取消       | ✓    | ✓      | ✓            | ✓            | 单次请求取消多个订单。    |

### 持仓管理

| 功能              | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                      |
|---------------------|------|--------|--------------|--------------|---------------------------------------------|
| 查询持仓（position）     | -    | ✓      | ✓            | ✓            | 实时持仓更新。                 |
| 持仓模式       | -    | -      | ✓            | ✓            | 单向 vs 对冲模式（持仓 ID）。       |
| 杠杆控制    | -    | ✓      | ✓            | ✓            | 按交易对动态调整杠杆。     |
| 保证金模式         | -    | ✓      | ✓            | ✓            | 按交易对设置全仓 vs 逐仓保证金。        |

### 风险事件

| 功能              | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                       |
|----------------------|------|--------|--------------|--------------|---------------------------------------------|
| 强平处理 | -    | -      | ✓            | ✓            | 交易所强制平仓。          |
| ADL 处理         | -    | -      | ✓            | ✓            | 自动减仓（Auto-Deleveraging）事件。                   |

Binance Futures 可能在风险事件中触发交易所生成的订单：

- **强平**：当保证金不足以维持持仓时，Binance 会以破产价格强制平仓。这些订单的客户端 ID 以 `autoclose-` 开头。
- **ADL（自动减仓）**：当保险基金耗尽时，Binance 会平掉盈利持仓以弥补亏损。这些订单使用客户端 ID `adl_autoclose`。
- **交割**：季度合约交割使用以 `settlement_autoclose-` 开头的客户端 ID。

适配器通过客户端 ID 模式和执行类型（`CALCULATED`）检测这些特殊订单类型，然后：

1. 记录带有订单详情的警告日志以便监控。
2. 生成 `OrderStatusReport` 以初始化缓存。
3. 生成包含正确成交详情和 TAKER 流动性方向的 `FillReport`。

这确保了强平和 ADL 事件能正确反映在投资组合状态和盈亏计算中。

### 订单查询

| 功能              | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                      |
|---------------------|------|--------|--------------|--------------|---------------------------------------------|
| 查询未完成订单   | ✓    | ✓      | ✓            | ✓            | 列出所有活跃订单。                     |
| 查询历史订单 | ✓    | ✓      | ✓            | ✓            | 历史订单数据。                      |
| 订单状态更新| ✓    | ✓      | ✓            | ✓            | 实时订单状态变更。              |
| 成交历史       | ✓    | ✓      | ✓            | ✓            | 执行和成交报告。                 |

### 条件订单

| 功能              | 现货 | 保证金 | USDT 期货 | 币本位期货 | 备注                                       |
|---------------------|------|--------|--------------|--------------|----------------------------------------------|
| 订单列表         | -    | -      | -            | -            | *不支持*。                             |
| OCO 订单          | ✓    | ✓      | ✓            | ✓            | 用于止损/止盈的一取消另一订单。 |
| Bracket 订单      | ✓    | ✓      | ✓            | ✓            | 止损 + 止盈组合。        |
| 条件订单  | ✓    | ✓      | ✓            | ✓            | 停损和触价订单。           |

### 订单参数

通过在调用 `Strategy.submit_order` 时提供 `params` 字典来自定义单个订单。Binance 执行客户端目前识别以下参数：

| 参数       | 类型   | 账户类型     | 描述 |
|-----------------|--------|-------------------|-------------|
| `price_match`   | `str`  | USDT/COIN Futures | 设置 Binance 的 `priceMatch` 模式之一（参见下方价格匹配部分），将价格选择委托给交易所。不能与 `post_only` 或冰山（`display_qty`）指令组合使用。 |

### 价格匹配

Binance Futures 通过 `priceMatch` 参数支持 BBO（最优买卖报价）价格匹配，将价格选择委托给交易所。此功能允许限价订单动态加入订单簿的最优价位，无需手动指定确切价格水平。

使用 `price_match` 时，你提交一个带有参考价格（用于本地风险检查）的限价订单，但 Binance 会根据当前市场状态和所选的价格匹配模式确定实际挂单价格。

#### 有效的价格匹配值

Binance Futures 有效的 `priceMatch` 值：

| 值         | 行为                                                      |
|---------------|----------------------------------------------------------------|
| `OPPONENT`    | 加入对手方最优价格。          |
| `OPPONENT_5`  | 加入对手方价格，但允许最多 5 个 tick 的偏移。  |
| `OPPONENT_10` | 加入对手方价格，但允许最多 10 个 tick 的偏移。 |
| `OPPONENT_20` | 加入对手方价格，但允许最多 20 个 tick 的偏移。 |
| `QUEUE`       | 加入同方向最优价格（保持 maker）。             |
| `QUEUE_5`     | 加入同方向队列，但偏移最多 5 个 tick。             |
| `QUEUE_10`    | 加入同方向队列，但偏移最多 10 个 tick。            |
| `QUEUE_20`    | 加入同方向队列，但偏移最多 20 个 tick。            |

:::info
更多详情请参阅[官方文档](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api)。
:::

#### 事件序列

当提交带有 `price_match` 的订单时，会发生以下事件序列：

1. **订单提交**：Nautilus 向 Binance 发送带有 `priceMatch` 参数的订单，但在 API 请求中省略限价。
2. **订单接受**：Binance 接受订单，并根据当前市场和指定的价格匹配模式确定实际挂单价格。
3. **OrderAccepted 事件**：订单确认后，Nautilus 生成 `OrderAccepted` 事件。
4. **OrderUpdated 事件**：如果 Binance 接受的价格与原始参考价格不同，Nautilus 会立即生成带有实际挂单价格的 `OrderUpdated` 事件。
5. **价格同步**：Nautilus 缓存中的订单限价现已与 Binance 接受的实际价格同步。

这确保了你系统中的订单价格准确反映 Binance 的接受价格，这对持仓管理、风险计算和策略逻辑至关重要。

#### 示例

```python
order = strategy.order_factory.limit(
    instrument_id=InstrumentId.from_str("BTCUSDT-PERP.BINANCE"),
    order_side=OrderSide.BUY,
    quantity=Quantity.from_int(1),
    price=Price.from_str("65000"),  # 用于本地风险检查的参考价格
)

strategy.submit_order(
    order,
    params={"price_match": "QUEUE"},
)
```

:::note
提交后，如果 Binance 以不同的价格接受了订单（例如 64,995.50），你将先收到 `OrderAccepted` 事件，随后收到带有新价格的 `OrderUpdated` 事件。
:::

### 追踪止损

对于 Binance 上的追踪止损市价订单：

- 使用 `activation_price`（可选）指定追踪机制的激活价格
- 省略时，Binance 使用提交时的当前市场价格
- 使用 `trailing_offset` 设置回调率（以基点为单位）

:::warning
不要对追踪止损订单使用 `trigger_price` -- 这会导致错误。请改用 `activation_price`。
:::

## 订单簿

订单簿可根据订阅设置维护全量或部分深度。
WebSocket 流的更新频率在现货和期货交易所之间有所不同，Nautilus 会使用可用的最高流速率：

- **现货**：100ms
- **期货**：0ms（*无节流*）

每个交易者实例的每个金融工具限制一个订单簿。
由于流订阅可能不同，Binance 数据客户端将使用最新的订单簿数据（增量或快照）订阅。

订单簿快照重建将在以下情况触发：

- 订单簿数据的初始订阅。
- 数据 WebSocket 重新连接。

事件序列如下：

- 增量数据开始缓冲。
- 请求并等待快照。
- 快照响应解析为 `OrderBookDeltas`。
- 快照增量发送到 `DataEngine`。
- 迭代缓冲的增量，丢弃序列号不大于快照中最后一个增量的数据。
- 停止缓冲增量数据。
- 剩余增量发送到 `DataEngine`。

## Binance 数据差异

`QuoteTick` 对象的 `ts_event` 字段值在现货和期货交易所之间有所不同，前者不提供事件时间戳，因此使用 `ts_init`（这意味着 `ts_event` 和 `ts_init` 相同）。

## Binance 特定数据

随着适配器逐步支持更多功能，可以订阅 Binance 特定的数据流。

:::note
K 线不被视为"Binance 特定"数据，可以通过常规方式订阅。
随着更多需要标记价格和资金费率更新等功能的适配器被构建，这些方法最终可能会成为一等公民（不再需要如下的自定义/通用订阅）。
:::

### `BinanceFuturesMarkPriceUpdate`

你可以通过以下方式在 actor 或策略中订阅 `BinanceFuturesMarkPriceUpdate`（包含资金费率信息）数据流：

```python
from nautilus_trader.adapters.binance import BinanceFuturesMarkPriceUpdate
from nautilus_trader.model import DataType
from nautilus_trader.model import ClientId

# 在你的 `on_start` 方法中
self.subscribe_data(
    data_type=DataType(BinanceFuturesMarkPriceUpdate, metadata={"instrument_id": self.instrument.id}),
    client_id=ClientId("BINANCE"),
)
```

这将使你的 actor/策略把接收到的 `BinanceFuturesMarkPriceUpdate` 对象传递到 `on_data` 方法中。你需要检查类型，因为此方法是所有自定义/通用数据的灵活处理器。

```python
from nautilus_trader.core import Data

def on_data(self, data: Data):
    # 首先检查数据类型
    if isinstance(data, BinanceFuturesMarkPriceUpdate):
        # 对数据进行处理
```

## 速率限制

Binance 使用基于时间窗口的速率限制系统，在固定时间窗口内跟踪请求权重（例如每分钟在 :00 秒重置）。适配器使用令牌桶速率限制器来近似此行为，有助于降低配额违规风险，同时维持正常交易操作的高吞吐量。

| 键 / 端点           | 限制（权重/分钟） | 备注                                                 |
|--------------------------|--------------------| ------------------------------------------------------|
| `binance:global`         | 现货: 6,000<br>期货: 2,400 | 应用于每个请求的默认桶。   |
| `/api/v3/order`          | 3,000              | 现货下单。                                 |
| `/api/v3/allOrders`      | 150                | 现货全部订单端点（20 倍权重乘数）。     |
| `/api/v3/klines`         | 600                | 现货历史 K 线。                               |
| `/fapi/v1/order`         | 1,200              | 期货下单。                              |
| `/fapi/v1/allOrders`     | 60                 | 期货历史订单（20 倍乘数）。           |
| `/fapi/v1/commissionRate`| 120                | 期货手续费率查询（20 倍乘数）。       |
| `/fapi/v1/klines`        | 600                | 期货历史 K 线。                            |

Binance 动态分配请求权重（例如 `/klines` 权重随 `limit` 参数变化）。上述配额反映的是静态限制，但客户端每次调用仍只消耗一个令牌，因此拉取长历史数据可能需要手动控制节奏以符合实时 `X-MBX-USED-WEIGHT-*` 头信息。

:::warning
当你超过允许的权重时，Binance 会返回 HTTP 429，反复突发请求可能触发临时 IP 封禁，因此请在批次之间留足余量。
:::

:::info
更多速率限制详情请参阅官方文档：<https://binance-docs.github.io/apidocs/futures/en/#limits>。
:::

## 配置

### 数据客户端配置选项

| 选项                             | 默认值 | 描述 |
|------------------------------------|---------|-------------|
| `venue`                            | `BINANCE` | 注册客户端时使用的交易场所标识符。 |
| `api_key`                          | `None`  | Binance API 密钥（API key）；省略时从环境变量加载。 |
| `api_secret`                       | `None`  | Binance API 密钥（secret）；省略时从环境变量加载。 |
| `key_type`                         | `HMAC`  | 加密密钥类型（`HMAC`、`RSA` 或 `ED25519`）。 |
| `account_type`                     | `SPOT`  | 数据端点的账户类型（现货、保证金、USDT 期货、币本位期货）。 |
| `base_url_http`                    | `None`  | HTTP REST 基础 URL 覆盖。 |
| `base_url_ws`                      | `None`  | WebSocket 基础 URL 覆盖。 |
| `proxy_url`                        | `None`  | HTTP 请求的可选代理 URL。 |
| `us`                               | `False` | 为 `True` 时将请求路由到 Binance US 端点。 |
| `testnet`                          | `False` | 为 `True` 时使用 Binance 测试网（testnet）端点。 |
| `update_instruments_interval_mins` | `60`    | 金融工具目录刷新间隔（分钟）。 |
| `use_agg_trade_ticks`              | `False` | 为 `True` 时订阅聚合交易 tick 而非原始交易。 |

### 执行客户端配置选项

| 选项                               | 默认值 | 描述 |
|--------------------------------------|---------|-------------|
| `venue`                              | `BINANCE` | 注册客户端时使用的交易场所标识符。 |
| `api_key`                            | `None`  | Binance API 密钥；省略时从环境变量加载。 |
| `api_secret`                         | `None`  | Binance API secret；省略时从环境变量加载。 |
| `key_type`                           | `HMAC`  | 加密密钥类型（`HMAC`、`RSA` 或 `ED25519`）。 |
| `account_type`                       | `SPOT`  | 下单的账户类型（现货、保证金、USDT 期货、币本位期货）。 |
| `base_url_http`                      | `None`  | HTTP REST 基础 URL 覆盖。 |
| `base_url_ws`                        | `None`  | WebSocket 基础 URL 覆盖。 |
| `proxy_url`                          | `None`  | HTTP 请求的可选代理 URL。 |
| `us`                                 | `False` | 为 `True` 时将请求路由到 Binance US 端点。 |
| `testnet`                            | `False` | 为 `True` 时使用 Binance 测试网端点。 |
| `use_gtd`                            | `True`  | 为 `False` 时将 GTD 订单重映射为 GTC 以进行本地到期管理。 |
| `use_reduce_only`                    | `True`  | 为 `True` 时将 `reduce_only` 指令传递给 Binance。 |
| `use_position_ids`                   | `True`  | 启用 Binance 对冲持仓 ID；设为 `False` 使用虚拟对冲。 |
| `use_trade_lite`                     | `False` | 使用包含衍生费用的 TRADE_LITE 执行事件。 |
| `treat_expired_as_canceled`          | `False` | 为 `True` 时将 `EXPIRED` 执行类型视为 `CANCELED`。 |
| `recv_window_ms`                     | `5,000` | 签名 REST 请求的接收窗口（毫秒）。 |
| `max_retries`                        | `None`  | 订单提交/取消/修改调用的最大重试次数。 |
| `retry_delay_initial_ms`             | `None`  | 重试尝试之间的初始延迟（毫秒）。 |
| `retry_delay_max_ms`                 | `None`  | 重试尝试之间的最大延迟（毫秒）。 |
| `futures_leverages`                  | `None`  | 期货账户的 `BinanceSymbol` 到初始杠杆的映射。 |
| `futures_margin_types`               | `None`  | `BinanceSymbol` 到期货保证金类型（逐仓/全仓）的映射。 |
| `listen_key_ping_max_failures`       | `3`     | 触发恢复前允许的连续 listen key ping 失败次数。 |
| `log_rejected_due_post_only_as_warning` | `True` | 为 `True` 时将 post-only 拒绝记录为警告；否则记录为错误。 |

最常见的用例是配置实盘 `TradingNode` 以包含 Binance 数据和执行客户端。为此，在客户端配置中添加 `BINANCE` 部分：

```python
from nautilus_trader.adapters.binance import BINANCE
from nautilus_trader.live.node import TradingNode

config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        BINANCE: {
            "api_key": "YOUR_BINANCE_API_KEY",
            "api_secret": "YOUR_BINANCE_API_SECRET",
            "account_type": "spot",  # {spot, margin, usdt_future, coin_future}
            "base_url_http": None,  # 使用自定义端点覆盖
            "base_url_ws": None,  # 使用自定义端点覆盖
            "us": False,  # 是否用于 Binance US
        },
    },
    exec_clients={
        BINANCE: {
            "api_key": "YOUR_BINANCE_API_KEY",
            "api_secret": "YOUR_BINANCE_API_SECRET",
            "account_type": "spot",  # {spot, margin, usdt_future, coin_future}
            "base_url_http": None,  # 使用自定义端点覆盖
            "base_url_ws": None,  # 使用自定义端点覆盖
            "us": False,  # 是否用于 Binance US
        },
    },
)
```

然后，创建 `TradingNode` 并添加客户端工厂：

```python
from nautilus_trader.adapters.binance import BINANCE
from nautilus_trader.adapters.binance import BinanceLiveDataClientFactory
from nautilus_trader.adapters.binance import BinanceLiveExecClientFactory
from nautilus_trader.live.node import TradingNode

# 使用配置实例化实盘交易节点
node = TradingNode(config=config)

# 向节点注册客户端工厂
node.add_data_client_factory(BINANCE, BinanceLiveDataClientFactory)
node.add_exec_client_factory(BINANCE, BinanceLiveExecClientFactory)

# 最后构建节点
node.build()
```

### 密钥类型

Binance 支持多种加密密钥类型用于 API 认证：

- **HMAC**（默认）：使用 HMAC-SHA256 和你的 API secret
- **RSA**：使用 RSA 签名和你的私钥
- **Ed25519**：使用 Ed25519 签名和你的私钥

你可以在配置中指定密钥类型：

```python
from nautilus_trader.adapters.binance import BinanceKeyType

config = TradingNodeConfig(
    data_clients={
        BINANCE: {
            "api_key": "YOUR_BINANCE_API_KEY",
            "api_secret": "YOUR_BINANCE_API_SECRET",  # 用于 HMAC
            "key_type": BinanceKeyType.ED25519,  # 或 RSA、HMAC（默认）
            "account_type": "spot",
        },
    },
)
```

:::note
Ed25519 密钥必须以 base64 编码的 ASN.1/DER 格式提供。实现会自动从 DER 结构中提取 32 字节的种子。
:::

### API 凭证

有多种方式可以向 Binance 客户端提供凭证。
可以将对应的值传递给配置对象，或设置以下环境变量：

对于 Binance 实盘客户端（现货/保证金和期货共享），可以设置：

- `BINANCE_API_KEY`
- `BINANCE_API_SECRET`（适用于所有密钥类型）

对于 Binance 现货/保证金测试网客户端，可以设置：

- `BINANCE_TESTNET_API_KEY`
- `BINANCE_TESTNET_API_SECRET`（适用于所有密钥类型）

对于 Binance 期货测试网客户端，可以设置：

- `BINANCE_FUTURES_TESTNET_API_KEY`
- `BINANCE_FUTURES_TESTNET_API_SECRET`（适用于所有密钥类型）

启动交易节点时，你将立即收到凭证是否有效以及是否具有交易权限的确认。

### 账户类型

所有 Binance 账户类型都将支持实盘交易。使用 `BinanceAccountType` 枚举设置 `account_type`。账户类型选项包括：

- `SPOT`
- `MARGIN`（持仓间共享的保证金）
- `ISOLATED_MARGIN`（分配给单个持仓的保证金）
- `USDT_FUTURES`（以 USDT 或 BUSD 稳定币作为抵押品）
- `COIN_FUTURES`（以其他加密货币作为抵押品）

:::tip
我们推荐使用环境变量管理你的凭证。
:::

### 基础 URL 覆盖

可以覆盖 HTTP REST 和 WebSocket API 的默认基础 URL。这对于出于性能原因配置 API 集群，或当 Binance 为你提供了专用端点时非常有用。

### Binance US

通过将配置中的 `us` 选项设为 `True`（默认为 `False`）来支持 Binance US 账户。US 账户可用的所有功能应与标准 Binance 行为一致。

### 测试网

也可以配置一个或两个客户端连接到 Binance 测试网。
将 `testnet` 选项设为 `True`（默认为 `False`）：

```python
from nautilus_trader.adapters.binance import BINANCE

config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        BINANCE: {
            "api_key": "YOUR_BINANCE_TESTNET_API_KEY",
            "api_secret": "YOUR_BINANCE_TESTNET_API_SECRET",
            "account_type": "spot",  # {spot, margin, usdt_future}
            "testnet": True,  # 是否使用测试网
        },
    },
    exec_clients={
        BINANCE: {
            "api_key": "YOUR_BINANCE_TESTNET_API_KEY",
            "api_secret": "YOUR_BINANCE_TESTNET_API_SECRET",
            "account_type": "spot",  # {spot, margin, usdt_future}
            "testnet": True,  # 是否使用测试网
        },
    },
)
```

### 聚合交易

Binance 提供聚合交易数据端点作为交易的替代数据源。与默认的交易端点相比，聚合交易数据端点可以返回 `start_time` 和 `end_time` 之间的所有 tick。

要使用聚合交易及端点功能，将 `use_agg_trade_ticks` 选项设为 `True`（默认为 `False`）。

### 手续费率查询

默认情况下，Binance Futures 金融工具使用基于你 VIP 等级的费率层级表。
对于具有负 maker 费率的做市商账户或需要精确费率时，可以启用按交易对查询手续费率：

```python
from nautilus_trader.adapters.binance import BinanceInstrumentProviderConfig

instrument_provider=BinanceInstrumentProviderConfig(
    load_all=True,
    query_commission_rates=True,  # 按交易对查询精确费率
)
```

启用后，适配器在加载金融工具时并行查询 Binance 的 `/fapi/v1/commissionRate` 端点获取每个交易对的费率。这对以下情况特别有用：

- 具有负 maker 费率的做市商账户。
- 具有自定义费率安排的账户。
- 确保盈亏计算使用精确的手续费率。

适配器使用带有适当速率限制（考虑端点权重 20 后为 120 请求/分钟）的并行请求。如果查询失败，会自动回退到费率层级表。

### 解析器警告

某些 Binance 金融工具如果包含超出平台处理能力的巨大字段值，则无法被解析为 Nautilus 对象。
在这些情况下，采用*警告并继续*的方式（该金融工具将不可用）。

这些警告可能会产生不必要的日志噪音，因此可以配置提供者不记录警告，如下面的客户端配置示例：

```python
from nautilus_trader.config import InstrumentProviderConfig

instrument_provider=InstrumentProviderConfig(
    load_all=True,
    log_warnings=False,
)
```

### 期货对冲模式

Binance Futures 对冲模式是一种持仓模式，交易者可以同时开设多头和空头持仓，以降低风险并从市场波动中获利。

要使用 Binance Futures 对冲模式，需要遵循以下三个步骤：

- 1. 在启动策略之前，确保已在 Binance 上配置了对冲模式。
- 2. 在 BinanceExecClientConfig 中将 `use_reduce_only` 选项设为 `False`（默认为 `True`）。

    ```python
    from nautilus_trader.adapters.binance import BINANCE

    config = TradingNodeConfig(
        ...,  # 省略
        data_clients={
            BINANCE: BinanceDataClientConfig(
                api_key=None,  # 'BINANCE_API_KEY' 环境变量
                api_secret=None,  # 'BINANCE_API_SECRET' 环境变量
                account_type=BinanceAccountType.USDT_FUTURES,
                base_url_http=None,  # 使用自定义端点覆盖
                base_url_ws=None,  # 使用自定义端点覆盖
            ),
        },
        exec_clients={
            BINANCE: BinanceExecClientConfig(
                api_key=None,  # 'BINANCE_API_KEY' 环境变量
                api_secret=None,  # 'BINANCE_API_SECRET' 环境变量
                account_type=BinanceAccountType.USDT_FUTURES,
                base_url_http=None,  # 使用自定义端点覆盖
                base_url_ws=None,  # 使用自定义端点覆盖
                use_reduce_only=False,  # 对冲模式必须禁用
            ),
        }
    )
    ```

- 3. 提交订单时，在 `position_id` 中使用后缀（`LONG` 或 `SHORT`）来指示持仓方向。

    ```python
    class EMACrossHedgeMode(Strategy):
        ...,  # 省略
        def buy(self) -> None:
            """
            用户的简单买入方法（示例）。
            """
            order: MarketOrder = self.order_factory.market(
                instrument_id=self.instrument_id,
                order_side=OrderSide.BUY,
                quantity=self.instrument.make_qty(self.trade_size),
                # time_in_force=TimeInForce.FOK,
            )

            # LONG 后缀被 Binance 适配器识别为多头持仓。
            position_id = PositionId(f"{self.instrument_id}-LONG")
            self.submit_order(order, position_id)

        def sell(self) -> None:
            """
            用户的简单卖出方法（示例）。
            """
            order: MarketOrder = self.order_factory.market(
                instrument_id=self.instrument_id,
                order_side=OrderSide.SELL,
                quantity=self.instrument.make_qty(self.trade_size),
                # time_in_force=TimeInForce.FOK,
            )
            # SHORT 后缀被 Binance 适配器识别为空头持仓。
            position_id = PositionId(f"{self.instrument_id}-SHORT")
            self.submit_order(order, position_id)
    ```

:::info
如需更多功能或为 Binance 适配器做出贡献，请参阅我们的[贡献指南](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::
