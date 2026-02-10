# Polymarket

Polymarket 成立于 2020 年，是全球最大的去中心化预测市场（Prediction Market）平台，
允许交易者通过使用加密货币买卖二元期权（Binary Option）合约来对世界事件的结果进行投机。

NautilusTrader 通过 Polymarket 的中央限价订单簿（CLOB）API 提供数据和执行的交易场所集成（Integration）。
该集成利用[官方 Python CLOB 客户端库](https://github.com/Polymarket/py-clob-client)来实现与 Polymarket 平台的交互。

NautilusTrader 支持多种 Polymarket 签名类型用于订单签名，为不同的钱包配置提供灵活性。
此集成确保交易者可以在各种钱包类型中安全高效地执行订单，
同时 NautilusTrader 抽象了签名和准备订单的复杂性，实现无缝执行。

## 安装

安装带有 Polymarket 支持的 NautilusTrader：

```bash
uv pip install "nautilus_trader[polymarket]"
```

从源码构建并包含所有扩展（包括 Polymarket）：

```bash
uv sync --all-extras
```

## 示例

你可以在[这里](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/polymarket/)找到实时示例脚本。

## 二元期权

[二元期权](https://en.wikipedia.org/wiki/Binary_option)是一种金融奇异期权合约，交易者对一个"是或否"命题的结果进行押注。
如果预测正确，交易者获得固定收益；否则不获得任何收益。

在 Polymarket 上交易的所有资产均以 **USDC.e (PoS)** 计价和结算，详情请参阅[下文](#usdce-pos)。

## Polymarket 文档

Polymarket 为不同受众提供了全面的资源：

- [Polymarket Learn](https://learn.polymarket.com/)：教育内容和指南，帮助用户了解平台及其使用方式。
- [Polymarket CLOB API](https://docs.polymarket.com/#introduction)：面向开发者的技术文档，用于与 Polymarket CLOB API 交互。

## 概述

本指南假设交易者正在同时设置实时市场数据源和交易执行。
Polymarket 集成适配器包含多个组件，可以根据使用场景一起使用或单独使用。

- `PolymarketWebSocketClient`：底层 WebSocket API 连接（基于 Rust 编写的 Nautilus `WebSocketClient` 构建）。
- `PolymarketInstrumentProvider`：`BinaryOption` 金融工具的解析和加载功能。
- `PolymarketDataClient`：市场数据源管理器。
- `PolymarketExecutionClient`：交易执行网关。
- `PolymarketLiveDataClientFactory`：Polymarket 数据客户端工厂（由交易节点构建器使用）。
- `PolymarketLiveExecClientFactory`：Polymarket 执行客户端工厂（由交易节点构建器使用）。

:::note
大多数用户将为实时交易节点定义配置（如下所述），不一定需要直接使用这些底层组件。
:::

## USDC.e (PoS)

**USDC.e** 是从以太坊桥接到 Polygon 网络的 USDC 版本，运行在 Polygon 的**权益证明（Proof of Stake, PoS）**链上。
这使得在 Polygon 上进行更快、更具成本效益的交易成为可能，同时保持以太坊上 USDC 的支持。

合约地址为 Polygon 区块链上的 [0x2791bca1f2de4661ed88a30c99a7a9449aa84174](https://polygonscan.com/address/0x2791bca1f2de4661ed88a30c99a7a9449aa84174)。
更多信息请参阅此[博客](https://polygon.technology/blog/phase-one-of-native-usdc-migration-on-polygon-pos-is-underway)。

## 钱包和账户

要通过 NautilusTrader 与 Polymarket 交互，你需要一个兼容 **Polygon** 的钱包（如 MetaMask）。

### 签名类型

Polymarket 支持多种签名类型用于订单签名和验证：

| 签名类型 | 钱包类型                         | 描述 | 使用场景 |
|---------|----------------------------------|------|---------|
| `0`     | EOA（外部拥有账户）               | 来自具有直接私钥控制的钱包的标准 EIP712 签名。 | **默认。**直接钱包连接（MetaMask、硬件钱包等）。 |
| `1`     | 邮件/Magic 钱包代理               | 用于基于邮件账户（Magic Link）的智能合约钱包。仅邮件关联地址可以执行功能。 | 与邮件/Magic 账户关联的 Polymarket 代理。需要 `funder` 地址。 |
| `2`     | 浏览器钱包代理                    | 用于浏览器钱包的修改版 Gnosis Safe（1-of-1 多签）。 | 与浏览器钱包关联的 Polymarket 代理。支持 UI 验证。需要 `funder` 地址。 |

:::note
另请参阅 Polymarket 文档中的 [Proxy wallet](https://docs.polymarket.com/developers/proxy-wallet)，了解更多关于签名类型和代理钱包基础设施的详情。
:::

NautilusTrader 默认使用签名类型 0（EOA），但可以通过 `signature_type` 配置参数配置为使用任何支持的签名类型。

使用环境变量时，每个交易者实例支持单个钱包地址，
或者可以通过多个 `PolymarketExecutionClient` 实例配置多个钱包。

:::note
确保你的钱包中有足够的 **USDC.e** 资金，否则在提交订单时会遇到"余额/授权额度不足"的 API 错误。
:::

### 设置 Polymarket 合约的授权额度

在开始交易之前，你需要确保你的钱包已为 Polymarket 的智能合约设置了授权额度。
你可以通过运行位于 `/adapters/polymarket/scripts/set_allowances.py` 的脚本来完成此操作。

此脚本改编自 @poly-rodr 创建的一个 [gist](https://gist.github.com/poly-rodr/44313920481de58d5a3f6d1f8226bd5e)。

:::note
对于每个你打算在 Polymarket 上用于交易的 EOA 钱包，你只需运行此脚本**一次**。
:::

此脚本自动化了批准 Polymarket 合约所需授权额度的过程。
它为 USDC 代币和条件代币框架（CTF）合约设置批准，以允许 Polymarket CLOB 交易所与你的资金交互。

运行脚本前，请确保满足以下先决条件：

- 安装 web3 Python 包：`uv pip install "web3==7.12.1"`。
- 拥有一个充有一定 MATIC（用于 Gas 费用）的兼容 **Polygon** 的钱包。
- 在 shell 中设置以下环境变量：
  - `POLYGON_PRIVATE_KEY`：你的兼容 **Polygon** 钱包的私钥。
  - `POLYGON_PUBLIC_KEY`：你的兼容 **Polygon** 钱包的公钥。

完成上述准备后，脚本将：

- 批准 Polymarket USDC 代币合约的最大 USDC 额度（使用 `MAX_INT` 值）。
- 设置 CTF 合约的批准，允许其与你的账户交互以进行交易。

:::note
你也可以在脚本中调整批准额度而不使用 `MAX_INT`，
额度以 **USDC.e** 的*小数单位*指定，但这尚未经过测试。
:::

确保在运行脚本之前将私钥和公钥正确存储在环境变量中。
以下是在终端会话中设置变量的示例：

```bash
export POLYGON_PRIVATE_KEY="YOUR_PRIVATE_KEY"
export POLYGON_PUBLIC_KEY="YOUR_PUBLIC_KEY"
```

使用以下命令运行脚本：

```bash
python nautilus_trader/adapters/polymarket/scripts/set_allowances.py
```

### 脚本详解

脚本执行以下操作：

- 通过 RPC URL (<https://polygon-rpc.com/>) 连接到 Polygon 网络。
- 签名并发送交易以批准 Polymarket 合约的最大 USDC 授权额度。
- 设置 CTF 合约的批准，以代你管理条件代币。
- 对 Polymarket CLOB Exchange 和 Neg Risk Adapter 等特定地址重复批准过程。

这使得 Polymarket 在执行交易时能够与你的资金交互，并确保与 CLOB 交易所的顺畅集成。

## API 密钥

要在 Polymarket 上交易，你需要生成 API 凭证。请按以下步骤操作：

1. 确保设置了以下环境变量：
   - `POLYMARKET_PK`：用于签署交易的私钥。
   - `POLYMARKET_FUNDER`：**Polygon** 网络上用于在 Polymarket 上资助交易的钱包地址（公钥）。

2. 使用以下命令运行脚本：

   ```bash
   python nautilus_trader/adapters/polymarket/scripts/create_api_key.py
   ```

脚本将生成并打印 API 凭证，你应将其保存到以下环境变量中：

- `POLYMARKET_API_KEY`
- `POLYMARKET_API_SECRET`
- `POLYMARKET_PASSPHRASE`

这些凭证可用于 Polymarket 客户端配置：

- `PolymarketDataClientConfig`
- `PolymarketExecClientConfig`

## 配置

在设置 NautilusTrader 与 Polymarket 配合使用时，正确配置必要参数（尤其是私钥）至关重要。

**关键参数**：

- `private_key`：用于签署订单的钱包私钥。其解释取决于你的 `signature_type` 配置。如果未在配置中显式提供，将自动读取 `POLYMARKET_PK` 环境变量。
- `funder`：用于资助交易的 **USDC.e** 钱包地址。如果未提供，将读取 `POLYMARKET_FUNDER` 环境变量。
- API 凭证：你需要提供以下 API 凭证以与 Polymarket CLOB 交互：
  - `api_key`：如果未提供，将读取 `POLYMARKET_API_KEY` 环境变量。
  - `api_secret`：如果未提供，将读取 `POLYMARKET_API_SECRET` 环境变量。
  - `passphrase`：如果未提供，将读取 `POLYMARKET_PASSPHRASE` 环境变量。

:::tip
我们建议使用环境变量来管理你的凭证。
:::

## 订单功能

Polymarket 作为预测市场运营，与传统交易所相比，其订单类型和指令集更为有限。

### 订单类型

| 订单类型                 | 二元期权 | 备注                                   |
|------------------------|----------|----------------------------------------|
| `MARKET`               | ✓        | **买单需要报价数量**，卖单需要基础数量。  |
| `LIMIT`                | ✓        |                                        |
| `STOP_MARKET`          | -        | *Polymarket 不支持*。                   |
| `STOP_LIMIT`           | -        | *Polymarket 不支持*。                   |
| `MARKET_IF_TOUCHED`    | -        | *Polymarket 不支持*。                   |
| `LIMIT_IF_TOUCHED`     | -        | *Polymarket 不支持*。                   |
| `TRAILING_STOP_MARKET` | -        | *Polymarket 不支持*。                   |

### 数量语义

Polymarket 根据订单类型*和*方向对订单数量的解释不同：

- **限价**订单将 `quantity` 解释为条件代币数量（基础单位）。
- **市价卖**单也使用基础单位数量。
- **市价买**单将 `quantity` 解释为 **USDC.e** 的报价名义金额。

因此，使用基础计价数量提交的市价买单将执行远超预期的规模。

:::warning
提交市价买单时，请设置 `quote_quantity=True`（或预先计算报价计价金额），
并将执行引擎配置为 `convert_quote_qty_to_base=False`，以便报价金额不变地传递给适配器。
Polymarket 执行客户端会拒绝基础计价的市价买单，以防止意外成交。

**NautilusTrader 现在将市价单转发到 Polymarket 的原生市价单端点，因此
你为买单指定的报价金额将直接执行（不再使用合成最高价格限制）。**
:::

```python
from nautilus_trader.execution.config import ExecEngineConfig
from nautilus_trader.execution.engine import ExecutionEngine

# 临时方案：禁用自动转换，直到在未来版本中完全移除该行为
config = ExecEngineConfig(convert_quote_qty_to_base=False)
engine = ExecutionEngine(msgbus=msgbus, cache=cache, clock=clock, config=config)

# 正确示例：使用报价数量的市价买单（花费 $10 USDC）
order = strategy.order_factory.market(
    instrument_id=instrument_id,
    order_side=OrderSide.BUY,
    quantity=instrument.make_qty(10.0),
    quote_quantity=True,  # 解释为 USDC.e 名义金额
)
strategy.submit_order(order)
```

### 执行指令

| 指令          | 二元期权 | 备注                                     |
|---------------|----------|------------------------------------------|
| `post_only`   | -        | *Polymarket 不支持*。                     |
| `reduce_only` | -        | *Polymarket 不支持*。                     |

### 有效期选项

| 有效期   | 二元期权 | 备注                                     |
|----------|----------|------------------------------------------|
| `GTC`    | ✓        | 撤销前有效（Good Till Canceled）。         |
| `GTD`    | ✓        | 指定日期前有效（Good Till Date）。         |
| `FOK`    | ✓        | 全部成交或取消（Fill or Kill）。           |
| `IOC`    | ✓        | 立即成交或取消（映射为 FAK）。            |

:::note
FAK（Fill and Kill）是 Polymarket 对立即成交或取消（IOC）语义的术语。
:::

### 高级订单功能

| 功能             | 二元期权 | 备注                                   |
|------------------|----------|----------------------------------------|
| 订单修改         | -        | 仅有取消功能。                          |
| 括号/OCO 订单    | -        | *Polymarket 不支持*。                   |
| 冰山订单         | -        | *Polymarket 不支持*。                   |

### 批量操作

| 操作             | 二元期权 | 备注                                   |
|------------------|----------|----------------------------------------|
| 批量提交         | -        | *Polymarket 不支持*。                   |
| 批量修改         | -        | *Polymarket 不支持*。                   |
| 批量取消         | -        | *Polymarket 不支持*。                   |

### 持仓管理

| 功能             | 二元期权 | 备注                                   |
|------------------|----------|----------------------------------------|
| 查询持仓         | ✓        | 基于合约余额的持仓。                    |
| 持仓模式         | -        | 仅二元结果持仓。                        |
| 杠杆控制         | -        | 无杠杆可用。                            |
| 保证金模式       | -        | 无保证金交易。                          |

### 订单查询

| 功能             | 二元期权 | 备注                                   |
|------------------|----------|----------------------------------------|
| 查询未成交订单   | ✓        | 仅活跃订单。                            |
| 查询订单历史     | ✓        | 有限的历史数据。                        |
| 订单状态更新     | ✓        | 实时订单状态变更。                       |
| 交易历史         | ✓        | 执行和成交报告。                        |

### 条件订单

| 功能             | 二元期权 | 备注                                   |
|------------------|----------|----------------------------------------|
| 订单列表         | -        | *Polymarket 不支持*。                   |
| OCO 订单         | -        | *Polymarket 不支持*。                   |
| 括号订单         | -        | *Polymarket 不支持*。                   |
| 条件订单         | -        | *Polymarket 不支持*。                   |

### 精度限制

Polymarket 根据最小变动价位和订单类型强制执行不同的精度约束。

**二元期权金融工具**通常支持最多 6 位小数的金额（最小变动价位为 0.0001），但**市价单有更严格的精度要求**：

- **FOK（全部成交或取消）市价单：**
  - 卖单：maker 金额限制为 **2 位小数**。
  - taker 金额：限制为 **4 位小数**。
  - 乘积 `数量 × 价格` 不得超过 **2 位小数**。

- **普通 GTC 订单：** 基于市场最小变动价位的更灵活精度。

### 最小变动价位精度层级

| 最小变动价位 | 价格小数位 | 数量小数位 | 金额小数位 |
|-------------|-----------|-----------|-----------|
| 0.1         | 1         | 2         | 3         |
| 0.01        | 2         | 2         | 4         |
| 0.001       | 3         | 2         | 5         |
| 0.0001      | 4         | 2         | 6         |

:::note

- 最小变动价位精度层级在 [`py-clob-client` `ROUNDING_CONFIG`](https://github.com/Polymarket/py-clob-client/blob/main/py_clob_client/order_builder/builder.py) 中定义。
- FOK 市价单精度限制（maker 金额 2 位小数）基于 [issue #121](https://github.com/Polymarket/py-clob-client/issues/121) 中记录的 API 错误响应。
- 最小变动价位可能在市场条件下动态变化，尤其是当市场变得单边时。

:::

## 交易

Polymarket 上的交易可能具有以下状态：

- `MATCHED`：交易已被匹配，由运营方发送到执行器服务。执行器服务将交易作为交易事务提交到交易所合约。
- `MINED`：交易已被观察到挖入链中，尚未建立最终确定性阈值。
- `CONFIRMED`：交易已达到强概率最终确定性且成功。
- `RETRYING`：交易事务失败（回滚或重组），运营方正在重试/重新提交。
- `FAILED`：交易失败且不会重试。

一旦交易初始匹配，后续交易状态更新将通过 WebSocket 接收。
NautilusTrader 在 `OrderFilled` 事件的 `info` 字段中记录初始交易详情，
额外的交易事件以 JSON 格式存储在缓存中的自定义键下，以保留此信息。

## 对账

Polymarket API 返回所有**活跃**（未成交）订单，或在按 Polymarket 订单 ID（`venue_order_id`）查询时返回特定订单。Polymarket 的执行对账流程如下：

- 为 Polymarket 报告的所有具有活跃（未成交）订单的金融工具生成订单报告。
- 从 Polymarket 报告的合约余额生成持仓报告，*仅针对缓存中可用的金融工具*。
- 将这些报告与 Nautilus 执行状态进行比较。
- 生成缺失的订单，使 Nautilus 执行状态与 Polymarket 报告的持仓保持一致。

**注意**：Polymarket 不直接提供已不再活跃的订单数据。

:::warning
一个可选的执行客户端配置 `generate_order_history_from_trades` 目前正在开发中。
目前不建议在生产环境中使用。
:::

## WebSocket

`PolymarketWebSocketClient` 基于高性能的 Nautilus `WebSocketClient` 基类构建，该基类由 Rust 编写。

### 数据

主数据 WebSocket 处理初始连接序列期间接收的所有 `market` 频道订阅，直到 `ws_connection_delay_secs`。对于任何额外的订阅，将为每个新金融工具（资产）创建一个新的 `PolymarketWebSocketClient`。

### 执行

主执行 WebSocket 基于初始连接序列期间缓存中可用的 Polymarket 金融工具管理所有 `user` 频道订阅。当为额外金融工具发出交易命令时，将为每个新金融工具（资产）创建一个单独的 `PolymarketWebSocketClient`。

:::note
Polymarket 不支持在订阅后取消频道流的订阅。
:::

### 订阅限制

Polymarket 强制执行**每个 WebSocket 连接最多 500 个金融工具**的限制（未记录的限制）。

当你尝试在单个 WebSocket 连接上订阅 501 个或更多金融工具时：

- 你**不会**收到每个金融工具的初始订单簿快照。
- 你只会收到后续的订单簿更新。

为处理此限制，NautilusTrader 自动管理 WebSocket 连接：

- 当订阅数量超过 500 个金融工具时，适配器会**自动创建额外的 WebSocket 连接**。
- 每个连接最多维护 500 个金融工具订阅。
- 此保护确保你收到所有已订阅金融工具的完整订单簿数据（包括初始快照）。

:::tip
如果你需要订阅大量金融工具（例如 5000+），适配器将自动将这些订阅分布在多个 WebSocket 连接上，每个连接最多处理 500 个金融工具。
:::

## 限制和注意事项

目前已知以下限制和注意事项：

- 通过 Polymarket Python 客户端的订单签名速度较慢，大约需要一秒。
- 不支持 post-only 订单。
- 不支持 reduce-only 订单。

## 配置

### 数据客户端配置选项

| 选项                              | 默认值            | 描述 |
|-----------------------------------|-------------------|------|
| `venue`                           | `POLYMARKET`      | 为数据客户端注册的交易场所标识符。 |
| `private_key`                     | `None`            | 钱包私钥；省略时从 `POLYMARKET_PK` 读取。 |
| `signature_type`                  | `0`               | 签名方案（0 = EOA，1 = 邮件代理，2 = 浏览器钱包代理）。 |
| `funder`                          | `None`            | USDC.e 资金钱包；省略时从 `POLYMARKET_FUNDER` 读取。 |
| `api_key`                         | `None`            | API 密钥；省略时从 `POLYMARKET_API_KEY` 读取。 |
| `api_secret`                      | `None`            | API 密钥；省略时从 `POLYMARKET_API_SECRET` 读取。 |
| `passphrase`                      | `None`            | API 口令；省略时从 `POLYMARKET_PASSPHRASE` 读取。 |
| `base_url_http`                   | `None`            | REST 基础 URL 覆盖。 |
| `base_url_ws`                     | `None`            | WebSocket 基础 URL 覆盖。 |
| `ws_connection_initial_delay_secs`| `5`               | 第一次 WebSocket 连接前的延迟（秒），用于缓冲订阅。 |
| `ws_connection_delay_secs`        | `0.1`             | 后续 WebSocket 连接尝试之间的延迟（秒）。 |
| `update_instruments_interval_mins`| `60`              | 金融工具目录刷新间隔（分钟）。 |
| `compute_effective_deltas`        | `False`           | 计算有效订单簿增量以节省带宽。 |
| `drop_quotes_missing_side`        | `True`            | 丢弃缺少买/卖价的报价，而不是替换为边界值。 |

### 执行客户端配置选项

| 选项                                | 默认值         | 描述 |
|-------------------------------------|---------------|------|
| `venue`                             | `POLYMARKET`  | 为执行客户端注册的交易场所标识符。 |
| `private_key`                       | `None`        | 钱包私钥；省略时从 `POLYMARKET_PK` 读取。 |
| `signature_type`                    | `0`           | 签名方案（0 = EOA，1 = 邮件代理，2 = 浏览器钱包代理）。 |
| `funder`                            | `None`        | USDC.e 资金钱包；省略时从 `POLYMARKET_FUNDER` 读取。 |
| `api_key`                           | `None`        | API 密钥；省略时从 `POLYMARKET_API_KEY` 读取。 |
| `api_secret`                        | `None`        | API 密钥；省略时从 `POLYMARKET_API_SECRET` 读取。 |
| `passphrase`                        | `None`        | API 口令；省略时从 `POLYMARKET_PASSPHRASE` 读取。 |
| `base_url_http`                     | `None`        | REST 基础 URL 覆盖。 |
| `base_url_ws`                       | `None`        | WebSocket 基础 URL 覆盖。 |
| `max_retries`                       | `None`        | 提交/取消请求的最大重试次数。 |
| `retry_delay_initial_ms`            | `None`        | 重试之间的初始延迟（毫秒）。 |
| `retry_delay_max_ms`                | `None`        | 重试之间的最大延迟（毫秒）。 |
| `ack_timeout_secs`                  | `5.0`         | 等待来自缓存的订单/交易确认的超时时间（秒）。 |
| `generate_order_history_from_trades`| `False`       | 为 `True` 时从交易报告生成合成订单历史（实验性功能）。 |
| `log_raw_ws_messages`               | `False`       | 为 `True` 时以 INFO 级别记录原始 WebSocket 消息。 |

:::info
如需了解更多功能或为 Polymarket 适配器做出贡献，请参阅我们的[贡献指南](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::

## 历史数据加载

`PolymarketDataLoader` 提供了获取和解析历史市场数据的方法，
用于研究和回测（Backtest）目的。该加载器与多个 Polymarket API 集成以提供所需数据。

:::note
所有数据获取方法都是**异步**的，必须使用 `await` 调用。加载器可以选择性地接受 `http_client` 参数用于依赖注入（对测试有用）。
:::

### 数据来源

加载器从两个主要来源获取数据：

1. **Polymarket Gamma API** - 市场元数据、金融工具详情和活跃市场列表。
2. **Polymarket CLOB API** - 价格/交易历史时间序列和订单簿历史快照。

### 查找市场

使用提供的实用脚本来发现活跃市场：

```bash
# 列出所有活跃市场
python nautilus_trader/adapters/polymarket/scripts/active_markets.py

# 专门列出 BTC 和 ETH UpDown 市场
python nautilus_trader/adapters/polymarket/scripts/list_updown_markets.py
```

### 基本用法

:::note
所有数据加载器方法都是**异步**的，必须使用 `await` 调用。
:::

```python
import asyncio
from datetime import UTC, datetime, timedelta

from nautilus_trader.adapters.polymarket import PolymarketDataLoader
from nautilus_trader.adapters.polymarket import parse_polymarket_instrument
from nautilus_trader.core.datetime import millis_to_nanos

async def load_market_data():
    # 发现方法是静态的——不需要实例
    market = await PolymarketDataLoader.find_market_by_slug("fed-rate-hike-in-2025")
    condition_id = market["conditionId"]

    market_details = await PolymarketDataLoader.fetch_market_details(condition_id)
    token = market_details["tokens"][0]
    token_id = token["token_id"]

    instrument = parse_polymarket_instrument(
        market_info=market_details,
        token_id=token_id,
        outcome=token["outcome"],
    )

    return instrument, token_id

# 运行异步函数并创建绑定到金融工具的加载器
instrument, token_id = asyncio.run(load_market_data())
loader = PolymarketDataLoader(instrument=instrument, token_id=token_id)
```

:::note
你也可以跳过手动连接，直接调用 `await PolymarketDataLoader.from_market_slug(...)`，它会获取元数据并返回一个已设置好 `instrument` 和 `token_id` 的加载器。
:::

### 获取订单簿历史

`load_orderbook_snapshots()` 便捷方法一步完成获取和解析订单簿数据：

```python
import pandas as pd

# 定义时间范围
end = pd.Timestamp.now(tz="UTC")
start = end - pd.Timedelta(hours=24)

# 获取并解析订单簿快照（自动处理分页）
deltas = await loader.load_orderbook_snapshots(
    start=start,
    end=end,
)
```

或者，你可以使用底层方法分别获取和解析：

```python
# 将时间戳转换为毫秒供 API 使用
start_time_ms = int(start.timestamp() * 1000)
end_time_ms = int(end.timestamp() * 1000)
token_id = loader.token_id

orderbook_snapshots = await loader.fetch_orderbook_history(
    token_id=token_id,
    start_time_ms=start_time_ms,
    end_time_ms=end_time_ms,
)

# 解析为 NautilusTrader OrderBookDeltas
deltas = loader.parse_orderbook_snapshots(orderbook_snapshots)
```

### 获取价格历史

`load_trades()` 便捷方法一步完成获取和解析交易数据：

```python
import pandas as pd

# 定义时间范围
end = pd.Timestamp.now(tz="UTC")
start = end - pd.Timedelta(hours=24)

# 获取并解析交易 tick（1 分钟精度）
trades = await loader.load_trades(
    start=start,
    end=end,
    fidelity=1,  # 1 = 1 分钟分辨率
)
```

或者，你可以使用底层方法分别获取和解析：

```python
# 将时间戳转换为毫秒供 API 使用
start_time_ms = int(start.timestamp() * 1000)
end_time_ms = int(end.timestamp() * 1000)
token_id = loader.token_id

# 获取原始价格历史
price_history = await loader.fetch_price_history(
    token_id=token_id,
    start_time_ms=start_time_ms,
    end_time_ms=end_time_ms,
    fidelity=1,  # 1 = 1 分钟分辨率
)

# 解析为 NautilusTrader TradeTick
trades = loader.parse_price_history(price_history)
```

:::warning
`parse_price_history()` 方法从价格点创建合成的 `TradeTick` 对象，
因为价格历史端点不包含实际交易规模。交易规模设置为 `1.0`，
主动方从价格变动推断。
:::

### 完整回测示例

完整的工作示例位于 `examples/backtest/polymarket_simple_quoter.py`：

```python
import asyncio
from decimal import Decimal

import pandas as pd

from nautilus_trader.adapters.polymarket import POLYMARKET_VENUE
from nautilus_trader.adapters.polymarket import PolymarketDataLoader
from nautilus_trader.backtest.config import BacktestEngineConfig
from nautilus_trader.backtest.engine import BacktestEngine
from nautilus_trader.examples.strategies.orderbook_imbalance import OrderBookImbalance
from nautilus_trader.examples.strategies.orderbook_imbalance import OrderBookImbalanceConfig
from nautilus_trader.model.currencies import USDC_POS
from nautilus_trader.model.enums import AccountType
from nautilus_trader.model.enums import BookType
from nautilus_trader.model.enums import OmsType
from nautilus_trader.model.identifiers import TraderId
from nautilus_trader.model.objects import Money

async def run_backtest():
    # 初始化加载器并获取市场数据
    loader = await PolymarketDataLoader.from_market_slug("fed-rate-hike-in-2025")
    instrument = loader.instrument

    # 获取历史数据
    start = pd.Timestamp("2025-10-30", tz="UTC")
    end = pd.Timestamp("2025-10-31", tz="UTC")

    deltas = await loader.load_orderbook_snapshots(
        start=start,
        end=end,
    )

    trades = await loader.load_trades(
        start=start,
        end=end,
    )

    # 配置并运行回测
    config = BacktestEngineConfig(trader_id=TraderId("BACKTESTER-001"))
    engine = BacktestEngine(config=config)

    engine.add_venue(
        venue=POLYMARKET_VENUE,
        oms_type=OmsType.NETTING,
        account_type=AccountType.CASH,
        base_currency=USDC_POS,
        starting_balances=[Money(10_000, USDC_POS)],
        book_type=BookType.L2_MBP,
    )

    engine.add_instrument(instrument)
    engine.add_data(deltas)
    engine.add_data(trades)

    strategy_config = OrderBookImbalanceConfig(
        instrument_id=instrument.id,
        max_trade_size=Decimal("20"),
    )

    strategy = OrderBookImbalance(config=strategy_config)
    engine.add_strategy(strategy=strategy)
    engine.run()

    # 显示结果
    print(engine.trader.generate_account_report(POLYMARKET_VENUE))

# 运行回测
asyncio.run(run_backtest())
```

**运行完整示例**：

```bash
python examples/backtest/polymarket_simple_quoter.py
```

### 辅助函数

适配器提供了用于处理 Polymarket 标识符的实用函数：

```python
from nautilus_trader.adapters.polymarket import get_polymarket_instrument_id

# 从 Polymarket 标识符创建 NautilusTrader InstrumentId
instrument_id = get_polymarket_instrument_id(
    condition_id="0x4319532e181605cb15b1bd677759a3bc7f7394b2fdf145195b700eeaedfd5221",
    token_id="60487116984468020978247225474488676749601001829886755968952521846780452448915"
)
```
