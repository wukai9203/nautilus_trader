# Bybit

Bybit 成立于 2018 年，是全球最大的加密货币交易所之一，在加密资产和加密衍生品的日交易量和未平仓合约方面均位居前列。本集成(integration)支持与 Bybit 进行实时市场数据接入和订单执行。

## 示例

你可以在[此处](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/bybit/)找到实时示例脚本。

## 概述

本指南假设交易者正在同时设置实时市场数据源和交易执行。
Bybit 适配器(adapter)包含多个组件，可以根据使用场景组合使用或单独使用。

- `BybitHttpClient`：底层 HTTP API 连接。
- `BybitWebSocketClient`：底层 WebSocket API 连接。
- `BybitInstrumentProvider`：金融工具(instrument)解析和加载功能。
- `BybitDataClient`：市场数据源管理器。
- `BybitExecutionClient`：账户管理和交易执行(execution)网关。
- `BybitLiveDataClientFactory`：Bybit 数据客户端工厂（由交易节点构建器使用）。
- `BybitLiveExecClientFactory`：Bybit 执行客户端工厂（由交易节点构建器使用）。

:::note
大多数用户只需为实时交易节点定义配置(configuration)（如下所示），
不需要直接使用这些底层组件。
:::

## Bybit 文档

Bybit 为用户提供了详尽的文档，可在 [Bybit 帮助中心](https://www.bybit.com/en/help-center)找到。
建议你在使用本 NautilusTrader 集成指南的同时，也参考 Bybit 官方文档。

## 产品

产品是一组相关金融工具类型的总称。

:::note
在 Bybit v5 API 中，产品也被称为 `category`（类别）。
:::

Bybit 支持以下产品类型：

| 产品类型 | 支持 | 备注 |
|---------|------|------|
| 现货(spot)加密货币 | ✓ | 原生现货市场，支持保证金(margin)。 |
| 正向永续合约(linear perpetual) | ✓ | USDT/USDC 保证金永续互换。 |
| 正向期货合约(linear futures) | ✓ | 交割结算的正向期货。 |
| 反向永续合约(inverse perpetual) | ✓ | 币本位保证金永续互换。 |
| 反向期货合约(inverse futures) | ✓ | 币本位交割期货。 |
| 期权合约(option) | ✓ | USDC 结算的期权。 |

## 符号体系

为了区分 Bybit 上不同的产品类型，Nautilus 使用特定的产品类别后缀：

- `-SPOT`：现货加密货币
- `-LINEAR`：永续合约和期货合约
- `-INVERSE`：反向永续和反向期货合约
- `-OPTION`：期权合约

这些后缀必须附加到 Bybit 原始符号字符串后面，以标识金融工具 ID 的具体产品类型。例如：

- 以太坊/泰达币现货货币对使用 `-SPOT` 标识，如 `ETHUSDT-SPOT`。
- BTCUSDT 永续期货合约使用 `-LINEAR` 标识，如 `BTCUSDT-LINEAR`。
- BTCUSD 反向永续期货合约使用 `-INVERSE` 标识，如 `BTCUSD-INVERSE`。

## 订单能力

Bybit 提供了灵活的触发类型组合，使得 Nautilus 支持更广泛的订单(order)类型。
以下所有订单类型均可用作*入场*或*出场*，追踪止损除外（它使用与持仓(position)相关的 API）。

### 订单类型

| 订单类型 | 现货 | 正向 | 反向 | 备注 |
|---------|------|------|------|------|
| `MARKET` | ✓ | ✓ | ✓ | 支持报价数量。 |
| `LIMIT` | ✓ | ✓ | ✓ | |
| `STOP_MARKET` | ✓ | ✓ | ✓ | |
| `STOP_LIMIT` | ✓ | ✓ | ✓ | |
| `MARKET_IF_TOUCHED` | ✓ | ✓ | ✓ | |
| `LIMIT_IF_TOUCHED` | ✓ | ✓ | ✓ | |
| `TRAILING_STOP_MARKET` | - | ✓ | ✓ | *现货不支持*。 |

### 执行指令

| 指令 | 现货 | 正向 | 反向 | 备注 |
|------|------|------|------|------|
| `post_only` | ✓ | ✓ | ✓ | 仅支持 `LIMIT` 订单。 |
| `reduce_only` | - | ✓ | ✓ | *现货不支持*。 |

### 有效时间

| 有效时间 | 现货 | 正向 | 反向 | 备注 |
|---------|------|------|------|------|
| `GTC` | ✓ | ✓ | ✓ | 撤单前有效(Good Till Canceled)。 |
| `GTD` | - | - | - | *不支持*。 |
| `FOK` | ✓ | ✓ | ✓ | 全部成交或撤销(Fill or Kill)。 |
| `IOC` | ✓ | ✓ | ✓ | 立即成交或撤销(Immediate or Cancel)。 |

### 高级订单功能

| 功能 | 现货 | 正向 | 反向 | 备注 |
|------|------|------|------|------|
| 订单修改 | ✓ | ✓ | ✓ | 价格和数量修改。 |
| 括号/OCO 订单 | ✓ | ✓ | ✓ | 仅限 UI；API 用户需手动实现。 |
| 冰山订单 | ✓ | ✓ | ✓ | 每账户最多 10 个，每符号 1 个。 |

### 批量操作

| 操作 | 现货 | 正向 | 反向 | 备注 |
|------|------|------|------|------|
| 批量提交 | ✓ | ✓ | ✓ | 单次请求提交多个订单。 |
| 批量修改 | ✓ | ✓ | ✓ | 单次请求修改多个订单。 |
| 批量取消 | ✓ | ✓ | ✓ | 单次请求取消多个订单。 |

### 持仓管理

| 功能 | 现货 | 正向 | 反向 | 备注 |
|------|------|------|------|------|
| 查询持仓 | - | ✓ | ✓ | 实时持仓更新。 |
| 持仓模式 | - | ✓ | ✓ | 单向 vs 对冲模式。 |
| 杠杆(leverage)控制 | - | ✓ | ✓ | 按符号动态调整杠杆。 |
| 保证金模式 | - | ✓ | ✓ | 全仓 vs 逐仓保证金。 |

### 订单查询

| 功能 | 现货 | 正向 | 反向 | 备注 |
|------|------|------|------|------|
| 查询未结订单 | ✓ | ✓ | ✓ | 列出所有活跃订单。 |
| 查询订单历史 | ✓ | ✓ | ✓ | 历史订单数据。 |
| 订单状态更新 | ✓ | ✓ | ✓ | 实时订单状态变更。 |
| 成交历史 | ✓ | ✓ | ✓ | 执行和成交报告。 |

### 条件订单

| 功能 | 现货 | 正向 | 反向 | 备注 |
|------|------|------|------|------|
| 订单列表 | - | - | - | *不支持*。 |
| OCO 订单 | ✓ | ✓ | ✓ | 仅限 UI；API 用户需手动实现。 |
| 括号订单 | ✓ | ✓ | ✓ | 仅限 UI；API 用户需手动实现。 |
| 条件订单 | ✓ | ✓ | ✓ | 止损和触价限价订单。 |

### 订单参数

提交订单时，可以使用 `params` 字典自定义单个订单：

| 参数 | 类型 | 描述 |
|------|------|------|
| `is_leverage` | `bool` | 仅适用于现货产品。如果为 `True`，则为该订单启用保证金交易（借款）。默认值：`False`。参见 [Bybit 的 isLeverage 文档](https://bybit-exchange.github.io/docs/v5/order/create-order#request-parameters)。 |

#### 示例：现货保证金交易

```python
# 提交启用保证金的现货订单
order = strategy.order_factory.market(
    instrument_id=InstrumentId.from_str("BTCUSDT-SPOT.BYBIT"),
    order_side=OrderSide.BUY,
    quantity=Quantity.from_str("0.1"),
    params={"is_leverage": True}  # 为此订单启用保证金
)
strategy.submit_order(order)
```

:::note
如果 params 中未设置 `is_leverage=True`，即使你在 Bybit 账户上启用了自动借款，现货订单也只会使用你的可用余额，不会借入资金。
:::

有关使用包含 `is_leverage` 在内的订单参数的完整示例，请参见
[bybit_exec_tester.py](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/bybit/bybit_exec_tester.py) 示例。

## 现货保证金借款与还款

NautilusTrader 提供自动化的现货保证金借款还款功能，以防止在 Bybit 上关闭空头持仓后产生利息。

### 背景

当启用保证金交易现货（`is_leverage=True`）时，Bybit 会在你执行空头持仓时自动借入代币。
但是，当你关闭空头持仓（买入订单成交）后，借入的代币**不会自动偿还** —— 它们会继续按小时计息，直到手动偿还为止。
如果不加注意，这可能导致显著的利息成本。

### 自动还款（推荐）

NautilusTrader 会在现货金融工具上的买入订单成交后立即自动偿还现货保证金借款。
此功能通过 `auto_repay_spot_borrows` 配置标志**默认启用**。

**工作原理：**

1. 当现货买入订单成交时，执行客户端会自动尝试偿还该代币的所有未偿借款。
2. 还款使用 Bybit 的 `no-convert-repay` 端点，默认偿还全部未偿借款金额。
3. 如果还款失败（例如 API 错误），会记录错误但不会导致执行客户端崩溃。
4. 在 Bybit 的 UTC 停服窗口期间（见下文），还款会自动跳过。

**示例：**

```python
from nautilus_trader.adapters.bybit import BybitExecClientConfig

config = BybitExecClientConfig(
    api_key="YOUR_API_KEY",
    api_secret="YOUR_API_SECRET",
    product_types=[BybitProductType.SPOT],
    auto_repay_spot_borrows=True,  # 默认为 True
)
```

### 手动保证金操作

策略可以通过 `query_account` 配合 `BybitMarginAction` 枚举直接控制保证金借款和还款：

| 操作 | 描述 |
|------|------|
| `BybitMarginAction.BORROW` | 为保证金交易借入资金。 |
| `BybitMarginAction.REPAY` | 偿还借入的资金。 |
| `BybitMarginAction.GET_BORROW_AMOUNT` | 查询当前借款金额。 |

#### 借款

```python
self.query_account(
    account_id=self.account_id,
    params={"action": BybitMarginAction.BORROW, "coin": "USDT", "amount": 1000},
)
```

#### 还款

```python
# 偿还指定金额
self.query_account(
    account_id=self.account_id,
    params={"action": BybitMarginAction.REPAY, "coin": "USDT", "amount": 500},
)

# 偿还全部（省略 amount）
self.query_account(
    account_id=self.account_id,
    params={"action": BybitMarginAction.REPAY, "coin": "USDT"},
)
```

#### 查询借款金额

```python
self.query_account(
    account_id=self.account_id,
    params={"action": BybitMarginAction.GET_BORROW_AMOUNT, "coin": "USDT"},
)
```

:::note
`account_id` 可以通过 `self.portfolio.account(BYBIT_VENUE).id` 获取，或在策略初始化时通过配置存储。
:::

#### 接收结果

结果以自定义数据的形式发布到消息总线。在策略中订阅以接收它们：

```python
from nautilus_trader.adapters.bybit import BybitMarginAction
from nautilus_trader.adapters.bybit import BybitMarginBorrowResult
from nautilus_trader.adapters.bybit import BybitMarginRepayResult
from nautilus_trader.adapters.bybit import BybitMarginStatusResult
from nautilus_trader.model.data import DataType


class MyStrategy(Strategy):
    def on_start(self):
        self.subscribe_data(DataType(BybitMarginBorrowResult))
        self.subscribe_data(DataType(BybitMarginRepayResult))
        self.subscribe_data(DataType(BybitMarginStatusResult))

    def on_data(self, data):
        if isinstance(data, BybitMarginBorrowResult):
            if data.success:
                self.log.info(f"已借入 {data.amount} {data.coin}")
            else:
                self.log.error(f"借款失败: {data.message}")
        elif isinstance(data, BybitMarginRepayResult):
            if data.success:
                self.log.info(f"已偿还 {data.amount or '全部'} {data.coin}")
            else:
                self.log.error(f"还款失败: {data.message}")
        elif isinstance(data, BybitMarginStatusResult):
            self.log.info(f"{data.coin} 的借款金额: {data.borrow_amount}")
```

### UTC 停服窗口

Bybit 每日在 **04:00-05:30 UTC** 期间屏蔽 `no-convert-repay` 操作，用于利息计算处理。NautilusTrader 会自动检测此窗口并跳过还款尝试，记录一条警告信息。

在停服窗口期间，任何买入订单成交都会触发类似以下的警告：

```
由于 Bybit 停服窗口（每日 04:00-05:30 UTC），跳过 BTC 的借款偿还。需要手动偿还。
```

**重要提示：** 如果你的买入订单在停服窗口期间成交，你需要在 05:30 UTC 之后手动偿还借款以停止利息计算，或者等待下一个停服窗口外的买入订单成交。

### 配置选项

| 选项 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `auto_repay_spot_borrows` | `bool` | `True` | 如果为 `True`，在买入订单成交后自动偿还现货保证金借款。防止借入代币产生利息。停服窗口期间跳过还款。 |

### 重要说明

- 自动还款仅在**现货买入订单**时触发，不适用于衍生品。
- 还款使用 `no-convert-repay` 端点，默认偿还全部未偿借款。
- 该功能优雅地处理 API 错误，记录失败但不会崩溃。
- Bybit 计划在交易所层面推出自动还款模式（月底），届时可能使此功能不再需要。
- 除非在你的 Bybit 账户上启用了自动借款，否则在开立空头持仓前仍需手动借款。

### 现货交易限制

以下限制适用于现货产品，因为交易所端不跟踪持仓：

- 不支持 `reduce_only` 订单。
- 不支持追踪止损订单。

### 追踪止损

Bybit 上的追踪止损在交易所端没有客户订单 ID（但有 `venue_order_id`）。
这是因为追踪止损与金融工具的净持仓相关联。
使用 Bybit 上的追踪止损时请注意以下几点：

- 可以使用 `reduce_only` 指令
- 当与追踪止损关联的持仓被关闭时，追踪止损会在交易所端自动"停用"（关闭）。
- 你无法查询尚未开放的追踪止损订单（此时 `venue_order_id` 未知）。
- 你可以在 GUI 中手动调整触发价格，这将更新 Nautilus 订单。

## 速率限制

每个 HTTP 调用都会消耗全局令牌桶以及相应的配额。当使用量超过桶的限制时，请求会自动排队，因此通常不需要手动节流。

| 键 / 端点 | 限制（请求/秒） | 备注 |
|-----------|----------------|------|
| `bybit:global` | 120 | 交易所全局上限 600 请求 / 5 秒。 |
| `/v5/market/kline` | 20 | 历史数据扫描的节流略低于全局限制。 |
| `/v5/market/trades` | 24 | 与全局配额一致。 |
| `/v5/order/create` | 10 | 标准下单。 |
| `/v5/order/cancel` | 10 | 单笔订单取消。 |
| `/v5/order/create-batch` | 5 | 批量下单端点。 |
| `/v5/order/cancel-batch` | 5 | 批量取消端点。 |
| `/v5/order/cancel-all` | 2 | 全部取消以配合 Bybit 指南。 |

:::warning
当速率限制被超出时，Bybit 会响应错误码 `10016`，如果请求在没有退避的情况下继续发送，可能会暂时封禁 IP。
:::

:::info
有关速率限制的更多详情，请参阅官方文档：<https://bybit-exchange.github.io/docs/v5/rate-limit>。
:::

### 数据客户端

如果未指定产品类型，则所有产品类型都将被加载并可用。

### 执行客户端

适配器会根据配置的产品类型自动确定账户类型：

- **仅现货**：使用 `CASH` 账户类型，启用借款支持
- **衍生品或混合产品**：使用 `MARGIN` 账户类型（UTA - 统一交易账户）

这允许你在单个统一交易账户中同时交易现货和衍生品，这是大多数 Bybit 用户的标准账户类型。

:::info
**统一交易账户 (UTA) 和现货保证金交易**

大多数 Bybit 用户现在都拥有统一交易账户 (UTA)，因为 Bybit 引导新用户使用此账户类型。
经典账户被视为遗留账户。

对于 UTA 账户上的现货保证金交易：

- 借款**不会自动启用** —— 需要显式的 API 配置
- 要通过 API 使用现货保证金，你必须在参数中提交带有 `is_leverage=True` 的订单（参见 [Bybit 文档](https://bybit-exchange.github.io/docs/v5/order/create-order#request-parameters)）
- 如果你的 Bybit 账户启用了自动借款/自动还款，交易所将自动为这些保证金订单借款/还款
- 如果未启用自动借款，你需要通过 Bybit 的界面手动管理借款

**重要提示**：Nautilus Bybit 适配器默认为现货订单设置 `is_leverage=False`，
这意味着除非你显式启用，否则不会使用保证金。
:::

## 手续费币种逻辑

了解 Bybit 如何确定交易手续费的币种对于准确的账务核算和持仓跟踪非常重要。手续费币种规则在现货和衍生品产品之间有所不同。

### 现货交易手续费

对于现货交易，手续费币种取决于订单方向以及手续费是否为返佣（挂单的负手续费）：

#### 正常手续费（正值）

- **买入订单**：手续费以**基础币种**收取（例如 BTCUSDT 的 BTC）
- **卖出订单**：手续费以**计价币种**收取（例如 BTCUSDT 的 USDT）

#### 挂单返佣（负手续费）

当挂单手续费为负值（返佣）时，币种逻辑**反转**：

- **带挂单返佣的买入订单**：返佣以**计价币种**支付（例如 BTCUSDT 的 USDT）
- **带挂单返佣的卖出订单**：返佣以**基础币种**支付（例如 BTCUSDT 的 BTC）

:::note
**吃单订单永远不会反转逻辑**，即使挂单费率为负值。吃单手续费始终遵循正常的手续费币种规则。
:::

#### 示例：BTCUSDT 现货

- **以吃单方式买入 1 BTC（0.1% 手续费）**：支付 0.001 BTC 手续费
- **以吃单方式卖出 1 BTC（0.1% 手续费）**：支付等值 USDT 手续费
- **以挂单方式买入 1 BTC（-0.01% 返佣）**：获得 USDT 返佣（反转）
- **以挂单方式卖出 1 BTC（-0.01% 返佣）**：获得 BTC 返佣（反转）

### 衍生品交易手续费

对于所有衍生品产品（LINEAR、INVERSE、OPTION），手续费始终以**结算币种**收取：

| 产品类型 | 结算币种 | 手续费币种 |
|---------|---------|-----------|
| LINEAR | USDT（通常） | USDT |
| INVERSE | 基础代币（例如 BTCUSD 的 BTC） | 基础代币 |
| OPTION | USDC（旧版）或 USDT（2025 年 2 月后） | USDC/USDT |

### 手续费计算

当 WebSocket 执行消息未提供确切手续费金额（`execFee`）时，适配器按如下方式计算手续费：

#### 现货产品

- **买入订单**：`手续费 = 基础数量 × 费率`
- **卖出订单**：`手续费 = 名义价值 × 费率`（其中 `名义价值 = 数量 × 价格`）

#### 衍生品

- 所有衍生品：`手续费 = 名义价值 × 费率`

### 官方文档

有关 Bybit 手续费结构和币种规则的完整详情，请参阅：

- [Bybit WebSocket 私有执行](https://bybit-exchange.github.io/docs/v5/websocket/private/execution)
- [Bybit 现货手续费币种说明](https://bybit-exchange.github.io/docs/v5/enum#spot-fee-currency-instruction)

## 配置

每个客户端的产品类型必须在配置中指定。

### 数据客户端配置选项

| 选项 | 默认值 | 描述 |
|------|--------|------|
| `api_key` | `None` | API 密钥(API key)；省略时从 `BYBIT_API_KEY`/`BYBIT_TESTNET_API_KEY` 加载。 |
| `api_secret` | `None` | API 密钥对；省略时从 `BYBIT_API_SECRET`/`BYBIT_TESTNET_API_SECRET` 加载。 |
| `product_types` | `None` | 要启用的 `BybitProductType` 值序列；为 `None` 时加载所有产品。 |
| `base_url_http` | `None` | REST 基础 URL 覆盖。 |
| `http_proxy_url` | `None` | 可选的 HTTP 代理 URL。 |
| `ws_proxy_url` | `None` | 可选的 WebSocket 代理 URL（尚未实现）。 |
| `demo` | `False` | 为 `True` 时连接到 Bybit 模拟环境。 |
| `testnet` | `False` | 为 `True` 时连接到 Bybit 测试网(testnet)。 |
| `update_instruments_interval_mins` | `60` | 金融工具目录刷新间隔（分钟）。 |
| `recv_window_ms` | `5,000` | 签名 REST 请求的接收窗口（毫秒）。 |
| `bars_timestamp_on_close` | `True` | K 线时间戳取收盘时间（`True`）或开盘时间（`False`）。 |
| `max_retries` | `None` | REST/WebSocket 恢复的最大重试次数。 |
| `retry_delay_initial_ms` | `None` | 重试之间的初始延迟（毫秒）。 |
| `retry_delay_max_ms` | `None` | 重试之间的最大延迟（毫秒）。 |

### 执行客户端配置选项

| 选项 | 默认值 | 描述 |
|------|--------|------|
| `api_key` | `None` | API 密钥；省略时从 `BYBIT_API_KEY`/`BYBIT_TESTNET_API_KEY` 加载。 |
| `api_secret` | `None` | API 密钥对；省略时从 `BYBIT_API_SECRET`/`BYBIT_TESTNET_API_SECRET` 加载。 |
| `product_types` | `None` | 要启用的 `BybitProductType` 值序列（执行时现货不能与衍生品混合）。 |
| `base_url_http` | `None` | REST 基础 URL 覆盖。 |
| `base_url_ws_private` | `None` | 私有 WebSocket 基础 URL 覆盖。 |
| `base_url_ws_trade` | `None` | 交易 WebSocket 基础 URL 覆盖。 |
| `http_proxy_url` | `None` | 可选的 HTTP 代理 URL。 |
| `ws_proxy_url` | `None` | 可选的 WebSocket 代理 URL（尚未实现）。 |
| `demo` | `False` | 为 `True` 时连接到 Bybit 模拟环境。 |
| `testnet` | `False` | 为 `True` 时连接到 Bybit 测试网。 |
| `use_gtd` | `False` | 为 `True` 时将 GTD 订单重映射为 GTC（Bybit 不原生支持 GTD）。 |
| `use_ws_execution_fast` | `False` | 订阅低延迟执行流。 |
| `use_http_batch_api` | `False` | 使用 Bybit 的 HTTP 批量交易 API（已弃用）。 |
| `use_spot_position_reports` | `False` | 为 `True` 时将现货钱包余额报告为持仓。 |
| `auto_repay_spot_borrows` | `True` | 在买入订单完全成交后自动偿还现货保证金借款（仅现货）。 |
| `ignore_uncached_instrument_executions` | `False` | 忽略尚未缓存的金融工具的执行消息。 |
| `max_retries` | `None` | 订单提交/取消/修改调用的最大重试次数。 |
| `retry_delay_initial_ms` | `None` | 重试之间的初始延迟（毫秒）。 |
| `retry_delay_max_ms` | `None` | 重试之间的最大延迟（毫秒）。 |
| `recv_window_ms` | `5,000` | 签名 REST 请求的接收窗口（毫秒）。 |
| `ws_trade_timeout_secs` | `5.0` | 等待交易 WebSocket 确认的超时时间（秒）。 |
| `ws_auth_timeout_secs` | `5.0` | 等待认证 WebSocket 确认的超时时间（秒）。 |
| `futures_leverages` | `None` | `BybitSymbol` 到杠杆设置的映射。 |
| `position_mode` | `None` | `BybitSymbol` 到持仓模式（单向 vs 对冲）的映射。 |
| `margin_mode` | `None` | 账户的保证金模式设置。 |

最常见的使用场景是配置一个实时 `TradingNode` 以包含 Bybit
数据和执行客户端。为此，在你的客户端配置中添加 `BYBIT` 部分：

```python
from nautilus_trader.adapters.bybit import BYBIT
from nautilus_trader.adapters.bybit import BybitProductType
from nautilus_trader.live.node import TradingNode

config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        BYBIT: {
            "api_key": "YOUR_BYBIT_API_KEY",
            "api_secret": "YOUR_BYBIT_API_SECRET",
            "base_url_http": None,  # 使用自定义端点覆盖
            "product_types": [BybitProductType.LINEAR]
            "testnet": False,
        },
    },
    exec_clients={
        BYBIT: {
            "api_key": "YOUR_BYBIT_API_KEY",
            "api_secret": "YOUR_BYBIT_API_SECRET",
            "base_url_http": None,  # 使用自定义端点覆盖
            "product_types": [BybitProductType.LINEAR]
            "testnet": False,
        },
    },
)
```

然后，创建一个 `TradingNode` 并添加客户端工厂：

```python
from nautilus_trader.adapters.bybit import BYBIT
from nautilus_trader.adapters.bybit import BybitLiveDataClientFactory
from nautilus_trader.adapters.bybit import BybitLiveExecClientFactory
from nautilus_trader.live.node import TradingNode

# 使用配置实例化实时交易节点
node = TradingNode(config=config)

# 向节点注册客户端工厂
node.add_data_client_factory(BYBIT, BybitLiveDataClientFactory)
node.add_exec_client_factory(BYBIT, BybitLiveExecClientFactory)

# 最后构建节点
node.build()
```

### API 凭证

有两种方式向 Bybit 客户端提供凭证。
可以将相应的 `api_key` 和 `api_secret` 值传递给配置对象，或者
设置以下环境变量：

对于 Bybit 实时客户端，你可以设置：

- `BYBIT_API_KEY`
- `BYBIT_API_SECRET`

对于 Bybit 模拟客户端，你可以设置：

- `BYBIT_DEMO_API_KEY`
- `BYBIT_DEMO_API_SECRET`

对于 Bybit 测试网客户端，你可以设置：

- `BYBIT_TESTNET_API_KEY`
- `BYBIT_TESTNET_API_SECRET`

:::tip
我们建议使用环境变量来管理你的凭证。
:::

启动交易节点时，你会立即收到凭证是否有效以及是否具有交易权限的确认。

:::info
如需额外功能或为 Bybit 适配器做出贡献，请参阅我们的
[贡献指南](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::
