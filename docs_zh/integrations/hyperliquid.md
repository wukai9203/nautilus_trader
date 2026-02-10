# Hyperliquid

Hyperliquid 是一个构建在 Arbitrum 区块链上的去中心化永续期货（Perpetual Futures）交易所，
提供完全链上的订单簿（Order Book）和撮合引擎。此集成（Integration）支持 Hyperliquid 的实时市场
数据（Data）源和订单执行（Execution）。

:::warning
Hyperliquid 集成正在积极开发中。部分功能可能尚未完成。
:::

## 概述

此适配器（Adapter）使用 Rust 实现，并提供 Python 绑定。它直接集成
Hyperliquid 的 REST 和 WebSocket API，无需依赖外部客户端库。

Hyperliquid 适配器包含多个组件：

- `HyperliquidHttpClient`：底层 HTTP API 连接。
- `HyperliquidWebSocketClient`：底层 WebSocket API 连接。
- `HyperliquidInstrumentProvider`：金融工具（Instrument）解析和加载功能。
- `HyperliquidDataClient`：市场数据源管理器。
- `HyperliquidExecutionClient`：账户管理和交易执行网关。
- `HyperliquidLiveDataClientFactory`：Hyperliquid 数据客户端工厂（供交易节点构建器使用）。
- `HyperliquidLiveExecClientFactory`：Hyperliquid 执行客户端工厂（供交易节点构建器使用）。

:::note
大多数用户只需为实盘交易节点定义配置（如下所示），
无需直接使用这些底层组件。
:::

## 示例

您可以在[此处](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/hyperliquid/)找到实时示例脚本。

## 测试网设置

Hyperliquid 提供测试网（Testnet）环境，用于在不冒真实资金风险的情况下测试策略。

### 获取测试网凭证

1. 访问 [Hyperliquid 测试网门户](https://app.hyperliquid-testnet.xyz/)
2. 连接您的钱包（MetaMask 或 WalletConnect）
3. 测试网将自动为您的钱包地址创建账户
4. 从水龙头（Faucet）领取测试网资金（如可用）

### 导出私钥

要在 NautilusTrader 中使用您的测试网账户，需要导出钱包的私钥：

**MetaMask：**

1. 点击账户旁边的三点菜单
2. 选择"Account details"
3. 点击"Show private key"
4. 输入密码并复制私钥

:::warning
**切勿分享您的私钥**
请使用环境变量安全存储私钥，切勿将其提交到版本控制系统。
:::

### 设置环境变量

将测试网凭证设置为环境变量：

```bash
export HYPERLIQUID_TESTNET_PK="your_private_key_here"
# 可选：用于金库交易
export HYPERLIQUID_TESTNET_VAULT="vault_address_here"
```

当配置中设置 `testnet=True` 时，适配器将自动加载这些变量。

## 产品支持

Hyperliquid 目前支持永续期货合约。

| 产品类型            | 数据源 | 交易 | 说明                              |
|---------------------|--------|------|-----------------------------------|
| 永续期货            | ✓      | ✓    | 同时支持 PERP 和 SPOT 工具。      |
| 现货（Spot）        | ✓      | ✓    | 原生现货市场。                    |

:::note
Hyperliquid 上所有金融工具均以 USDC 结算。
:::

## 交易代码

Hyperliquid 对金融工具使用特定的代码格式：

### 永续期货

格式：`{Base}-USD-PERP`

示例：

- `BTC-USD-PERP` - 比特币永续期货
- `ETH-USD-PERP` - 以太坊永续期货
- `SOL-USD-PERP` - Solana 永续期货

在策略中订阅：

```python
InstrumentId.from_str("BTC-USD-PERP.HYPERLIQUID")
InstrumentId.from_str("ETH-USD-PERP.HYPERLIQUID")
```

### 现货市场

格式：`{Base}-{Quote}-SPOT`

示例：

- `PURR-USDC-SPOT` - PURR/USDC 现货交易对
- `HYPE-USDC-SPOT` - HYPE/USDC 现货交易对

在策略中订阅：

```python
InstrumentId.from_str("PURR-USDC-SPOT.HYPERLIQUID")
```

:::note
现货工具可能包含金库代币（以 `vntls:` 为前缀）。这些由金融工具提供者自动处理。
:::

## 订单功能

Hyperliquid 支持一套全面的订单类型和执行选项。

### 订单类型

| 订单类型               | 永续   | 现货 | 说明                                   |
|------------------------|--------|------|----------------------------------------|
| `MARKET`               | ✓      | ✓    | 以 IOC 限价单方式执行。                |
| `LIMIT`                | ✓      | ✓    |                                        |
| `STOP_MARKET`          | ✓      | ✓    | 止损订单。                             |
| `STOP_LIMIT`           | ✓      | ✓    | 带限价执行的止损订单。                 |
| `MARKET_IF_TOUCHED`    | ✓      | ✓    | 市价止盈。                             |
| `LIMIT_IF_TOUCHED`     | ✓      | ✓    | 带限价执行的止盈。                     |

:::info
条件订单（止损和触价订单）使用 Hyperliquid 原生的触发订单功能实现，
并自动检测 TP/SL 模式。
:::

### 有效时间

| 有效时间      | 永续   | 现货 | 说明                         |
|---------------|--------|------|------------------------------|
| `GTC`         | ✓      | ✓    | Good Till Canceled（撤销前有效）。 |
| `IOC`         | ✓      | ✓    | Immediate or Cancel（立即成交或撤销）。 |
| `FOK`         | -      | -    | *不支持*。                   |
| `GTD`         | -      | -    | *不支持*。                   |

### 执行指令

| 指令          | 永续   | 现货 | 说明                               |
|---------------|--------|------|------------------------------------|
| `post_only`   | ✓      | ✓    | 等同于 ALO 有效时间。              |
| `reduce_only` | ✓      | ✓    | 仅平仓订单。                      |

### 订单操作

| 操作             | 永续   | 现货 | 说明                                   |
|------------------|--------|------|----------------------------------------|
| 提交订单         | ✓      | ✓    | 单笔订单提交。                         |
| 提交订单列表     | ✓      | ✓    | 批量订单提交。                         |
| 修改订单         | ✓      | ✓    | 修改价格和数量。                       |
| 撤销订单         | ✓      | ✓    | 通过客户端订单 ID 撤销。              |
| 撤销全部订单     | ✓      | ✓    | 撤销指定工具/方向的全部订单。          |
| 批量撤销         | ✓      | ✓    | 在一个请求中撤销多个订单。            |

## 配置

### 数据客户端配置选项

| 选项                     | 默认值  | 描述                                             |
|--------------------------|---------|--------------------------------------------------|
| `base_url_http`          | `None`  | REST 基础 URL 的覆盖值。                         |
| `base_url_ws`            | `None`  | WebSocket 基础 URL 的覆盖值。                    |
| `testnet`                | `False` | 设为 `True` 时连接到 Hyperliquid 测试网。        |
| `http_timeout_secs`      | `10`    | REST 调用的超时时间（秒）。                      |
| `http_proxy_url`         | `None`  | 可选的 HTTP 代理 URL。                           |
| `ws_proxy_url`           | `None`  | 可选的 WebSocket 代理 URL。                      |

### 执行客户端配置选项

| 选项                     | 默认值  | 描述                                                       |
|--------------------------|---------|-------------------------------------------------------------|
| `private_key`            | `None`  | EVM 私钥；省略时从 `HYPERLIQUID_PK` 或 `HYPERLIQUID_TESTNET_PK` 加载。 |
| `vault_address`          | `None`  | 用于委托交易的金库地址；省略时从 `HYPERLIQUID_VAULT` 或 `HYPERLIQUID_TESTNET_VAULT` 加载。 |
| `base_url_http`          | `None`  | REST 基础 URL 的覆盖值。                                    |
| `base_url_ws`            | `None`  | WebSocket 基础 URL 的覆盖值。                               |
| `testnet`                | `False` | 设为 `True` 时连接到 Hyperliquid 测试网。                   |
| `max_retries`            | `None`  | 订单提交/撤销/修改的最大重试次数。                          |
| `retry_delay_initial_ms` | `None`  | 重试之间的初始延迟（毫秒）。                                |
| `retry_delay_max_ms`     | `None`  | 重试之间的最大延迟（毫秒）。                                |
| `http_timeout_secs`      | `10`    | REST 调用的超时时间（秒）。                                 |
| `http_proxy_url`         | `None`  | 可选的 HTTP 代理 URL。                                      |
| `ws_proxy_url`           | `None`  | 可选的 WebSocket 代理 URL。                                 |

### 配置示例

```python
from nautilus_trader.adapters.hyperliquid import HYPERLIQUID
from nautilus_trader.adapters.hyperliquid import HyperliquidDataClientConfig
from nautilus_trader.adapters.hyperliquid import HyperliquidExecClientConfig
from nautilus_trader.config import InstrumentProviderConfig
from nautilus_trader.config import TradingNodeConfig

config = TradingNodeConfig(
    data_clients={
        HYPERLIQUID: HyperliquidDataClientConfig(
            instrument_provider=InstrumentProviderConfig(load_all=True),
            testnet=True,  # 使用测试网
        ),
    },
    exec_clients={
        HYPERLIQUID: HyperliquidExecClientConfig(
            private_key=None,  # 从 HYPERLIQUID_TESTNET_PK 环境变量加载
            vault_address=None,  # 可选：从 HYPERLIQUID_TESTNET_VAULT 加载
            instrument_provider=InstrumentProviderConfig(load_all=True),
            testnet=True,  # 使用测试网
        ),
    },
)
```

:::note
当 `testnet=True` 时，适配器自动使用测试网环境变量
（`HYPERLIQUID_TESTNET_PK` 和 `HYPERLIQUID_TESTNET_VAULT`）而非主网变量。
:::
