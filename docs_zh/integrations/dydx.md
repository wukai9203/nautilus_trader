# dYdX

dYdX 是按日交易量计算最大的去中心化加密货币衍生品交易所之一。dYdX 运行在以太坊区块链上的智能合约上，允许用户在没有中介的情况下进行交易。本集成（Integration）支持与 dYdX v4 的实时市场数据接入和订单执行（Execution），这是该协议的第一个完全去中心化、没有中心化组件的版本。

## 安装

安装带有 dYdX 支持的 NautilusTrader：

```bash
uv pip install "nautilus_trader[dydx]"
```

从源码构建并包含所有扩展（包括 dYdX）：

```bash
uv sync --all-extras
```

## 示例

您可以在[此处](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/dydx/)找到实时示例脚本。

## 概述

本指南假设交易者正在设置实时市场数据推送和交易执行。dYdX 适配器（Adapter）包含多个组件，可以根据使用场景组合或单独使用。

- `DYDXHttpClient`：底层 HTTP API 连接。
- `DYDXWebSocketClient`：底层 WebSocket API 连接。
- `DYDXAccountGRPCAPI`：用于账户更新的底层 gRPC API 连接。
- `DYDXInstrumentProvider`：金融工具（Instrument）解析和加载功能。
- `DYDXDataClient`：市场数据推送管理器。
- `DYDXExecutionClient`：账户管理和交易执行网关。
- `DYDXLiveDataClientFactory`：dYdX 数据客户端工厂（由交易节点构建器使用）。
- `DYDXLiveExecClientFactory`：dYdX 执行客户端工厂（由交易节点构建器使用）。

:::note
大多数用户只需为实时交易节点定义配置（如下所示），无需直接使用这些底层组件。
:::

:::warning 首次账户激活
dYdX v4 交易账户（子账户 0）**仅在**钱包首次入金或交易后才会创建。在此之前，每个 gRPC/Indexer 查询都会返回 `NOT_FOUND`，导致 `DYDXExecutionClient.connect()` 失败。

**操作 ->** 在启动实时 `TradingNode` 之前，从同一钱包**在同一网络**（主网/测试网）上发送任意正数金额的 USDC（>= 1 wei）或其他支持的抵押品。
交易确认后（几个区块后）重启节点；客户端将正常连接。
:::

## 故障排除

### `StatusCode.NOT_FOUND` — 未找到账户 ... /0

**原因** *钱包/子账户从未充值，因此在链上尚不存在。*

**解决方法**

1. 在正确的网络上向子账户 0 存入任意正数金额的 USDC。
2. 等待最终确认（主网约 30 秒，测试网更长）。
3. 重启 `TradingNode`；连接现在应该成功。

:::tip
在无人值守的部署中，将 `connect()` 调用包裹在指数退避循环中，以便客户端在入金到账前持续重试。
:::

## 符号体系

dYdX 上仅提供永续合约。为了与其他适配器保持一致，并为 dYdX 未来可能提供其他产品做好准备，NautilusTrader 为所有可用的永续符号添加 `-PERP` 后缀。例如，比特币/USD-C 永续期货合约标识为 `BTC-USD-PERP`。所有市场的报价货币为 USD-C。因此，dYdX 将其缩写为 USD。

## 短期订单和长期订单

dYdX 区分短期订单和长期订单（即有状态订单）。短期订单旨在立即下单，属于接收订单的同一区块。这些订单在内存中保留最多 20 个区块，只有成交数量和到期区块高度被提交到状态。短期订单主要面向高吞吐量的做市商或市价单使用。

默认情况下，所有订单作为短期订单发送。要构建长期订单，您可以给订单附加标签，如下所示：

```python
from nautilus_trader.adapters.dydx import DYDXOrderTags

order: LimitOrder = self.order_factory.limit(
    instrument_id=self.instrument_id,
    order_side=OrderSide.BUY,
    quantity=self.instrument.make_qty(self.trade_size),
    price=self.instrument.make_price(price),
    time_in_force=TimeInForce.GTD,
    expire_time=self.clock.utc_now() + pd.Timedelta(minutes=10),
    post_only=True,
    emulation_trigger=self.emulation_trigger,
    tags=[DYDXOrderTags(is_short_term_order=False).value],
)
```

要指定订单活跃的区块数量：

```python
from nautilus_trader.adapters.dydx import DYDXOrderTags

order: LimitOrder = self.order_factory.limit(
    instrument_id=self.instrument_id,
    order_side=OrderSide.BUY,
    quantity=self.instrument.make_qty(self.trade_size),
    price=self.instrument.make_price(price),
    time_in_force=TimeInForce.GTD,
    expire_time=self.clock.utc_now() + pd.Timedelta(seconds=5),
    post_only=True,
    emulation_trigger=self.emulation_trigger,
    tags=[DYDXOrderTags(is_short_term_order=True, num_blocks_open=5).value],
)
```

## 市价单

市价单需要指定价格以提供价格滑点保护，并使用隐藏订单。通过为市价单设置价格，您可以限制潜在的价格滑点。例如，如果您为市价买入订单设置了 $100 的价格，订单将仅在市场价格等于或低于 $100 时执行。如果市场价格高于 $100，订单将不会执行。

某些交易所（包括 dYdX）支持隐藏订单。隐藏订单是对其他市场参与者不可见但仍可执行的订单。通过为市价单设置价格，您可以创建一个仅在市场价格达到指定价格时才会执行的隐藏订单。

如果未指定市场价格，则使用默认值 0。

创建市价单时指定价格：

```python
order = self.order_factory.market(
    instrument_id=self.instrument_id,
    order_side=OrderSide.BUY,
    quantity=self.instrument.make_qty(self.trade_size),
    time_in_force=TimeInForce.IOC,
    tags=[DYDXOrderTags(is_short_term_order=True, market_order_price=Price.from_str("10_000")).value],
)
```

## 止损限价单和止损市价单

可以提交止损限价和止损市价条件订单。dYdX 对条件订单仅支持长期订单。

## 订单功能

dYdX 支持永续期货交易，提供全面的订单类型和执行功能。

### 订单类型

| 订单类型               | 永续合约 | 备注                                   |
|------------------------|---------|----------------------------------------|
| `MARKET`               | ✓       | 需要价格以提供滑点保护。不支持报价数量。    |
| `LIMIT`                | ✓       |                                        |
| `STOP_MARKET`          | ✓       | 仅限长期订单。                           |
| `STOP_LIMIT`           | ✓       | 仅限长期订单。                           |
| `MARKET_IF_TOUCHED`    | -       | *不支持*。                               |
| `LIMIT_IF_TOUCHED`     | -       | *不支持*。                               |
| `TRAILING_STOP_MARKET` | -       | *不支持*。                               |

### 执行指令

| 指令           | 永续合约 | 备注                          |
|---------------|---------|-------------------------------|
| `post_only`   | ✓       | 所有订单类型均支持。             |
| `reduce_only` | ✓       | 所有订单类型均支持。             |

### 有效时间选项

| 有效时间 | 永续合约 | 备注                                |
|---------|---------|-------------------------------------|
| `GTC`   | ✓       | 撤销前有效（Good Till Canceled）。     |
| `GTD`   | ✓       | 到期前有效（Good Till Date）。         |
| `FOK`   | ✓       | 全部成交或撤销（Fill or Kill）。        |
| `IOC`   | ✓       | 立即成交或撤销（Immediate or Cancel）。 |

### 高级订单功能

| 功能            | 永续合约 | 备注                                          |
|----------------|---------|-----------------------------------------------|
| 订单修改        | ✓       | 仅限短期订单；使用取消-替换方式。                  |
| 组合/OCO 订单   | -       | *不支持*。                                     |
| 冰山订单        | -       | *不支持*。                                     |

### 批量操作

| 操作          | 永续合约 | 备注                                          |
|--------------|---------|-----------------------------------------------|
| 批量提交      | -       | *不支持*。                                     |
| 批量修改      | -       | *不支持*。                                     |
| 批量取消      | -       | *不支持*。                                     |

### 持仓管理

| 功能          | 永续合约 | 备注                                          |
|--------------|---------|-----------------------------------------------|
| 查询持仓      | ✓       | 实时持仓更新。                                  |
| 持仓模式      | -       | 仅支持净持仓模式。                               |
| 杠杆控制      | ✓       | 按市场的杠杆设置。                               |
| 保证金模式    | -       | 仅支持全仓保证金。                               |

### 订单查询

| 功能             | 永续合约 | 备注                                          |
|-----------------|---------|-----------------------------------------------|
| 查询未结订单     | ✓       | 列出所有活动订单。                               |
| 查询历史订单     | ✓       | 历史订单数据。                                   |
| 订单状态更新     | ✓       | 实时订单状态变更。                               |
| 交易历史         | ✓       | 成交和填充报告。                                 |

### 条件订单

| 功能            | 永续合约 | 备注                                          |
|----------------|---------|-----------------------------------------------|
| 订单列表        | -       | *不支持*。                                     |
| OCO 订单       | -       | *不支持*。                                     |
| 组合订单        | -       | *不支持*。                                     |
| 条件订单        | ✓       | 止损市价单和止损限价单。                          |

### 订单分类

dYdX 将订单分为**短期**和**长期**订单：

- **短期订单**：所有订单的默认类型；面向高频交易和市价单。
- **长期订单**：条件订单必须使用；通过 `DYDXOrderTags` 指定。

## 配置

每个客户端的产品类型必须在配置中指定。

### 数据客户端配置选项

| 选项                               | 默认值   | 描述 |
|------------------------------------|---------|------|
| `wallet_address`                   | `None`  | 钱包地址；省略时从 `DYDX_WALLET_ADDRESS`/`DYDX_TESTNET_WALLET_ADDRESS` 加载。 |
| `is_testnet`                       | `False` | 为 `True` 时连接到 dYdX 测试网。 |
| `update_instruments_interval_mins` | `60`    | 金融工具目录刷新的间隔（分钟）。 |
| `max_retries`                      | `None`  | REST/WebSocket 恢复的最大重试次数。 |
| `retry_delay_initial_ms`           | `None`  | 重试间的初始延迟（毫秒）。 |
| `retry_delay_max_ms`               | `None`  | 重试间的最大延迟（毫秒）。 |
| `proxy_url`                        | `None`  | 可选的 HTTP 请求代理 URL。 |

### 执行客户端配置选项

| 选项                     | 默认值   | 描述 |
|--------------------------|---------|------|
| `wallet_address`         | `None`  | 钱包地址；省略时从 `DYDX_WALLET_ADDRESS`/`DYDX_TESTNET_WALLET_ADDRESS` 加载。 |
| `subaccount`             | `0`     | 子账户编号（dYdX 默认配置子账户 `0`）。 |
| `mnemonic`               | `None`  | 用于派生签名密钥的助记词；省略时从环境变量加载。 |
| `base_url_http`          | `None`  | REST 基础 URL 覆盖。 |
| `base_url_ws`            | `None`  | WebSocket 基础 URL 覆盖。 |
| `is_testnet`             | `False` | 为 `True` 时连接到 dYdX 测试网。 |
| `max_retries`            | `None`  | 订单提交/取消/修改调用的最大重试次数。 |
| `retry_delay_initial_ms` | `None`  | 重试间的初始延迟（毫秒）。 |
| `retry_delay_max_ms`     | `None`  | 重试间的最大延迟（毫秒）。 |
| `proxy_url`              | `None`  | 可选的 HTTP 请求代理 URL。 |

### 执行客户端

账户类型必须为保证金账户才能交易永续期货合约。

最常见的使用场景是配置实时 `TradingNode` 以包含 dYdX 数据和执行客户端。为此，将 `DYDX` 部分添加到您的客户端配置中：

```python
from nautilus_trader.live.node import TradingNode

config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        "DYDX": {
            "wallet_address": "YOUR_DYDX_WALLET_ADDRESS",
            "is_testnet": False,
        },
    },
    exec_clients={
        "DYDX": {
            "wallet_address": "YOUR_DYDX_WALLET_ADDRESS",
            "subaccount": "YOUR_DYDX_SUBACCOUNT_NUMBER"
            "mnemonic": "YOUR_MNEMONIC",
            "is_testnet": False,
        },
    },
)
```

然后，创建 `TradingNode` 并添加客户端工厂：

```python
from nautilus_trader.adapters.dydx import DYDXLiveDataClientFactory
from nautilus_trader.adapters.dydx import DYDXLiveExecClientFactory
from nautilus_trader.live.node import TradingNode

# 使用配置实例化实时交易节点
node = TradingNode(config=config)

# 向节点注册客户端工厂
node.add_data_client_factory("DYDX", DYDXLiveDataClientFactory)
node.add_exec_client_factory("DYDX", DYDXLiveExecClientFactory)

# 最后构建节点
node.build()
```

### API 凭证

有两种方式向 dYdX 客户端提供凭证。将对应的 `wallet_address` 和 `mnemonic` 值传递给配置对象，或设置以下环境变量：

对于 dYdX 正式环境客户端，您可以设置：

- `DYDX_WALLET_ADDRESS`
- `DYDX_MNEMONIC`

对于 dYdX 测试网客户端，您可以设置：

- `DYDX_TESTNET_WALLET_ADDRESS`
- `DYDX_TESTNET_MNEMONIC`

:::tip
我们建议使用环境变量来管理您的凭证。
:::

数据客户端使用钱包地址来确定交易手续费。交易手续费仅在回测中使用。

### 测试网

也可以配置一个或两个客户端连接到 dYdX 测试网。将 `is_testnet` 选项设置为 `True`（默认为 `False`）：

```python
config = TradingNodeConfig(
    ...,  # 省略
    data_clients={
        "DYDX": {
            "wallet_address": "YOUR_DYDX_WALLET_ADDRESS",
            "is_testnet": True,
        },
    },
    exec_clients={
        "DYDX": {
            "wallet_address": "YOUR_DYDX_WALLET_ADDRESS",
            "subaccount": "YOUR_DYDX_SUBACCOUNT_NUMBER"
            "mnemonic": "YOUR_MNEMONIC",
            "is_testnet": True,
        },
    },
)
```

### 解析器警告

某些 dYdX 金融工具如果包含超出平台处理能力的巨大字段值，则无法解析为 Nautilus 对象。在这种情况下，采用*警告并继续*的方式（该金融工具将不可用）。

## 订单簿

订单簿可以根据订阅维护全深度或最优价格（Top-of-Book）报价。交易场所（Venue）不提供报价，但适配器订阅订单簿增量更新，并在最优价格或数量发生变化时向 `DataEngine` 发送新的报价。

:::info
如需额外功能或为 dYdX 适配器做贡献，请参阅我们的
[贡献指南](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::
