# Coinbase International

[Coinbase International Exchange](https://www.coinbase.com/en/international-exchange) 为非美国机构客户提供加密货币永续期货（Perpetual Futures）和现货（Spot）市场的访问。
该交易所（Exchange）为欧洲和国际交易者提供杠杆加密衍生品交易服务，而这些服务在某些地区通常受到限制或不可用。

Coinbase International 提供高标准的客户保护、强大的风险管理框架和高性能的交易技术，包括：

- 24/7/365 全天候实时风险管理。
- 来自外部做市商的流动性（无自营交易）。
- 动态保证金要求和抵押品评估。
- 符合严格合规标准的清算框架。
- 资本充足的交易所以应对极端市场事件。
- 与顶级全球监管机构的合作。

:::info
详情请参阅 [Introducing Coinbase International Exchange](https://www.coinbase.com/en-au/blog/introducing-coinbase-international-exchange) 博客文章。
:::

## 安装

:::note
无需额外安装 `coinbase_intx`；适配器（Adapter）的核心组件由 Rust 编写，在构建过程中会自动编译和链接。
:::

## 示例

你可以在[这里](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/coinbase_intx)找到实时示例脚本。
这些示例演示了如何设置实时市场数据（Data）源和执行（Execution）客户端，以在 Coinbase International 上进行交易。

## 概述

Coinbase International 交易所支持以下产品：

- 永续期货合约
- 现货加密货币

本指南假设交易者正在同时设置实时市场数据源和交易执行。
Coinbase International 适配器包含多个组件，可以根据使用场景一起使用或单独使用。这些组件协同工作，连接到 Coinbase International 的 API 以获取市场数据和执行交易。

- `CoinbaseIntxHttpClient`：REST API 连接。
- `CoinbaseIntxWebSocketClient`：WebSocket API 连接。
- `CoinbaseIntxInstrumentProvider`：金融工具（Instrument）解析和加载功能。
- `CoinbaseIntxDataClient`：市场数据源管理器。
- `CoinbaseIntxExecutionClient`：账户管理和交易执行网关。
- `CoinbaseIntxLiveDataClientFactory`：Coinbase International 数据客户端工厂。
- `CoinbaseIntxLiveExecClientFactory`：Coinbase International 执行客户端工厂。

:::note
大多数用户将为实时交易节点定义配置（如下所述），不一定需要直接使用上述组件。
:::

## Coinbase 文档

Coinbase International 为用户提供了广泛的 API 文档，可在 [Coinbase Developer Platform](https://docs.cdp.coinbase.com/intx/docs/welcome) 中找到。
我们建议结合本 NautilusTrader 集成（Integration）指南一并参阅 Coinbase International 文档。

## 数据

### 金融工具

启动时，适配器会自动从 Coinbase International REST API 加载所有可用的金融工具，并订阅 `INSTRUMENTS` WebSocket 频道以获取更新。这确保了缓存和需要最新定义进行解析的客户端始终拥有最新的金融工具数据。

可用的金融工具类型包括：

- `CurrencyPair`（现货加密货币）
- `CryptoPerpetual`

:::note
指数产品尚未实现。
:::

以下数据类型可用：

- `OrderBookDelta`（L2 按价格聚合的市场深度）
- `QuoteTick`（L1 最优买卖价）
- `TradeTick`
- `Bar`
- `MarkPriceUpdate`
- `IndexPriceUpdate`

:::note
历史数据（Historical Data）请求尚未实现。
:::

### WebSocket 市场数据

数据客户端连接到 Coinbase International 的 WebSocket 数据源，以流式传输实时市场数据。
WebSocket 客户端支持自动重连，并在重新连接后重新订阅活跃的订阅。

## 执行

**该适配器设计为每个执行客户端对应一个 Coinbase International 投资组合。**

### 选择投资组合

要查看可用的投资组合及其 ID，请使用 REST 客户端运行以下脚本：

```bash
python nautilus_trader/adapters/coinbase_intx/scripts/list_portfolios.py
```

将输出类似以下的投资组合详情列表：

```bash
[{'borrow_disabled': False,
  'cross_collateral_enabled': False,
  'is_default': False,
  'is_lsp': False,
  'maker_fee_rate': '-0.00008',
  'name': 'hrp5587988499',
  'portfolio_id': '3mnk59ap-1-22',  # 你的投资组合 ID
  'portfolio_uuid': 'dd0958ad-0c9d-4445-a812-1870fe40d0e1',
  'pre_launch_trading_enabled': False,
  'taker_fee_rate': '0.00012',
  'trading_lock': False,
  'user_uuid': 'd4fbf7ea-9515-1068-8d60-4de91702c108'}]
```

### 配置投资组合

要指定用于交易的投资组合，请将 `COINBASE_INTX_PORTFOLIO_ID` 环境变量设置为所需的 `portfolio_id`。如果使用多个执行客户端，你也可以在每个客户端的执行配置中定义 `portfolio_id`。

## 订单功能

Coinbase International 提供市价单、限价单和止损单类型，支持广泛的策略。

### 订单类型

| 订单类型                 | 衍生品 | 现货 | 备注                                      |
|------------------------|--------|------|-------------------------------------------|
| `MARKET`               | ✓      | ✓    | 必须使用 `IOC` 或 `FOK` 有效期            |
| `LIMIT`                | ✓      | ✓    |                                           |
| `STOP_MARKET`          | ✓      | ✓    |                                           |
| `STOP_LIMIT`           | ✓      | ✓    |                                           |
| `MARKET_IF_TOUCHED`    | -      | -    | *不支持*。                                 |
| `LIMIT_IF_TOUCHED`     | -      | -    | *不支持*。                                 |
| `TRAILING_STOP_MARKET` | -      | -    | *不支持*。                                 |

### 执行指令

| 指令          | 衍生品 | 现货 | 备注                                              |
|---------------|--------|------|---------------------------------------------------|
| `post_only`   | ✓      | ✓    | 确保订单仅提供流动性。                              |
| `reduce_only` | ✓      | ✓    | 确保订单仅减少现有持仓。                            |

### 有效期选项

| 有效期   | 衍生品 | 现货 | 备注                                              |
|----------|--------|------|---------------------------------------------------|
| `GTC`    | ✓      | ✓    | 撤销前有效（Good Till Canceled）。                  |
| `GTD`    | ✓      | ✓    | 指定日期前有效（Good Till Date）。                  |
| `FOK`    | ✓      | ✓    | 全部成交或取消（Fill or Kill）。                    |
| `IOC`    | ✓      | ✓    | 立即成交或取消（Immediate or Cancel）。             |

### 高级订单功能

| 功能             | 衍生品 | 现货 | 备注                                        |
|------------------|--------|------|---------------------------------------------|
| 订单修改         | ✓      | ✓    | 支持价格和数量修改。                          |
| 括号/OCO 订单    | ?      | ?    | 需要进一步调研。                              |
| 冰山订单         | ✓      | ✓    | 通过 FIX 协议可用。                           |

### 批量操作

| 操作             | 衍生品 | 现货 | 备注                                        |
|------------------|--------|------|---------------------------------------------|
| 批量提交         | -      | -    | *不支持*。                                   |
| 批量修改         | -      | -    | *不支持*。                                   |
| 批量取消         | -      | -    | *不支持*。                                   |

### 持仓管理

| 功能             | 衍生品 | 现货 | 备注                                        |
|------------------|--------|------|---------------------------------------------|
| 查询持仓         | ✓      | -    | 衍生品实时持仓更新。                          |
| 持仓模式         | -      | -    | 仅支持单一持仓模式。                          |
| 杠杆控制         | ✓      | -    | 按投资组合设置杠杆。                          |
| 保证金模式       | ✓      | -    | 仅支持全仓保证金。                            |

### 订单查询

| 功能             | 衍生品 | 现货 | 备注                                        |
|------------------|--------|------|---------------------------------------------|
| 查询未成交订单   | ✓      | ✓    | 列出所有活跃订单。                            |
| 查询订单历史     | ✓      | ✓    | 历史订单数据。                                |
| 订单状态更新     | ✓      | ✓    | 通过 FIX drop copy 实时更新。                 |
| 交易历史         | ✓      | ✓    | 执行和成交报告。                              |

### 条件订单

| 功能             | 衍生品 | 现货 | 备注                                        |
|------------------|--------|------|---------------------------------------------|
| 订单列表         | -      | -    | *不支持*。                                   |
| OCO 订单         | ?      | ?    | 需要进一步调研。                              |
| 括号订单         | ?      | ?    | 需要进一步调研。                              |
| 条件订单         | ✓      | ✓    | 止损和止损限价订单。                          |

### FIX Drop Copy 集成

Coinbase International 适配器包含一个 FIX（金融信息交换协议）[drop copy](https://docs.cdp.coinbase.com/intx/docs/fix-msg-drop-copy) 客户端。
这提供了来自 Coinbase 撮合引擎的可靠、低延迟执行更新。

:::note
这种方式是必要的，因为 WebSocket 数据源不提供执行消息，而且它比轮询 REST API 能提供更快、更可靠的订单执行更新。
:::

FIX 客户端：

- 在交易节点启动时自动建立安全的 TCP/TLS 连接并登录。
- 处理连接监控，在连接中断时自动重连和重新登录。
- 在交易节点停止时正确注销并关闭连接。

客户端处理多种类型的执行消息：

- 订单状态报告（已取消、已过期、已触发）。
- 成交报告（部分成交和完全成交）。

FIX 凭证使用与 REST 和 WebSocket 客户端相同的 API 凭证自动管理。
除了提供有效的 API 凭证外，无需额外配置。

:::note
REST 客户端在订单提交时处理 `REJECTED` 和 `ACCEPTED` 状态的执行消息。
:::

### 账户和持仓管理

启动时，执行客户端会请求并加载你当前的账户和执行状态，包括：

- 所有资产的可用余额。
- 未成交订单。
- 未平仓持仓。

这为你的交易策略提供了在下新订单之前对账户的完整了解。

## 配置

### 策略

:::warning
Coinbase International 对客户端订单 ID 有严格的规范。
Nautilus 可以通过使用 UUID4 值作为客户端订单 ID 来满足该规范。
为此，请在策略配置中设置 `use_uuid_client_order_ids=True` 选项（否则，订单提交将触发 API 错误）。

详情请参阅 Coinbase International [Create order](https://docs.cdp.coinbase.com/intx/reference/createorder) REST API 文档。
:::

### 数据客户端配置选项

| 选项              | 默认值           | 描述 |
|-------------------|-----------------|------|
| `venue`           | `COINBASE_INTX` | 为数据客户端注册的交易场所（Venue）标识符。 |
| `api_key`         | `None`          | API 密钥（API Key）；省略时从 `COINBASE_INTX_API_KEY`（或测试网变体）加载。 |
| `api_secret`      | `None`          | API 密钥；省略时从 `COINBASE_INTX_API_SECRET`（或测试网变体）加载。 |
| `api_passphrase`  | `None`          | API 口令；省略时从 `COINBASE_INTX_API_PASSPHRASE` 加载。 |
| `base_url_http`   | `None`          | REST 基础 URL 覆盖。 |
| `base_url_ws`     | `None`          | WebSocket 基础 URL 覆盖。 |
| `http_timeout_secs` | `60`          | 应用于 REST 调用的默认超时时间（秒）。 |

### 执行客户端配置选项

| 选项               | 默认值           | 描述 |
|--------------------|-----------------|------|
| `venue`            | `COINBASE_INTX` | 为执行客户端注册的交易场所标识符。 |
| `api_key`          | `None`          | API 密钥；省略时从 `COINBASE_INTX_API_KEY`（或测试网变体）加载。 |
| `api_secret`       | `None`          | API 密钥；省略时从 `COINBASE_INTX_API_SECRET`（或测试网变体）加载。 |
| `api_passphrase`   | `None`          | API 口令；省略时从 `COINBASE_INTX_API_PASSPHRASE` 加载。 |
| `portfolio_id`     | `None`          | 用于交易的投资组合标识符；提交订单时必填。 |
| `base_url_http`    | `None`          | REST 基础 URL 覆盖。 |
| `base_url_ws`      | `None`          | WebSocket 基础 URL 覆盖。 |
| `http_timeout_secs`| `60`            | 应用于 REST 调用的默认超时时间（秒）。 |

配置示例：

```python
from nautilus_trader.adapters.coinbase_intx import COINBASE_INTX, CoinbaseIntxDataClientConfig, CoinbaseIntxExecClientConfig
from nautilus_trader.live.node import TradingNode

config = TradingNodeConfig(
    ...,  # 其他配置省略
    data_clients={
        COINBASE_INTX: CoinbaseIntxDataClientConfig(
            instrument_provider=InstrumentProviderConfig(load_all=True),
        ),
    },
    exec_clients={
        COINBASE_INTX: CoinbaseIntxExecClientConfig(
            instrument_provider=InstrumentProviderConfig(load_all=True),
        ),
    },
)

strat_config = TOBQuoterConfig(
    use_uuid_client_order_ids=True,  # <-- Coinbase Intx 必需
    instrument_id=instrument_id,
    external_order_claims=[instrument_id],
    ...,  # 其他配置省略
)
```

然后，创建一个 `TradingNode` 并添加客户端工厂：

```python
from nautilus_trader.adapters.coinbase_intx import COINBASE_INTX, CoinbaseIntxLiveDataClientFactory, CoinbaseIntxLiveExecClientFactory
from nautilus_trader.live.node import TradingNode

# 使用配置实例化实时交易节点
node = TradingNode(config=config)

# 向节点注册客户端工厂
node.add_data_client_factory(COINBASE_INTX, CoinbaseIntxLiveDataClientFactory)
node.add_exec_client_factory(COINBASE_INTX, CoinbaseIntxLiveExecClientFactory)

# 最后构建节点
node.build()
```

### API 凭证

通过以下方法之一向客户端提供凭证。

直接传入以下配置选项的值：

- `api_key`
- `api_secret`
- `api_passphrase`
- `portfolio_id`

或者，设置以下环境变量：

- `COINBASE_INTX_API_KEY`
- `COINBASE_INTX_API_SECRET`
- `COINBASE_INTX_API_PASSPHRASE`
- `COINBASE_INTX_PORTFOLIO_ID`

:::tip
我们建议使用环境变量来管理你的凭证。
:::

启动交易节点时，你将立即收到凭证是否有效以及是否具有交易权限的确认。

## 实现说明

- **心跳**：适配器在 WebSocket 和 FIX 连接上均维护心跳，以确保可靠的连接。
- **速率限制**：REST API 客户端配置为每秒限制 100 个请求，与 Coinbase International REST 配额一致。详情请参阅 <https://docs.cdp.coinbase.com/intx/docs/rate-limits> 获取官方指导。

:::warning
当超过每秒 100 个请求的配额时，Coinbase International 会返回 HTTP 429，并可能会在数秒内限制 API 密钥的使用，因此请将突发请求保持在文档规定的上限以下。
:::

- **优雅关闭**：适配器正确处理优雅关闭，确保在断开连接前处理完所有待处理的消息。
- **线程安全**：所有适配器组件都是线程安全的，允许从多个线程并发使用。
- **执行模型**：适配器可以为每个执行客户端配置一个 Coinbase International 投资组合。如需交易多个投资组合，可以创建多个执行客户端。
