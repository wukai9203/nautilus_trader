# Betfair

Betfair 成立于 2000 年，运营着全球最大的在线博彩交易所（Betting Exchange），
总部位于伦敦，在全球多地设有分支机构。

NautilusTrader 提供了一个适配器（Adapter），用于集成（Integration） Betfair REST API 和
Exchange Streaming API。

## 安装

安装带有 Betfair 支持的 NautilusTrader：

```bash
uv pip install "nautilus_trader[betfair]"
```

从源码构建并包含 Betfair 扩展：

```bash
uv sync --all-extras
```

## 示例

您可以在[此处](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/betfair/)找到实时示例脚本。

## Betfair 文档

有关 API 详情和故障排除，请参阅官方 [Betfair Developer Documentation](https://developer.betfair.com/en/get-started/)。

## 应用密钥

Betfair 要求使用应用密钥（Application Key）来验证 API 请求。注册并为账户充值后，可通过 [API-NG Developer AppKeys Tool](https://apps.betfair.com/visualisers/api-ng-account-operations/) 获取密钥。

:::info
另请参阅 [Betfair Getting Started - Application Keys](https://betfair-developer-docs.atlassian.net/wiki/spaces/1smk3cen4v3lu3yomq5qye0ni/pages/2687105/Application+Keys) 指南。
:::

## API 凭证

通过环境变量或客户端配置提供您的 Betfair 凭证：

```bash
export BETFAIR_USERNAME=<your_username>
export BETFAIR_PASSWORD=<your_password>
export BETFAIR_APP_KEY=<your_app_key>
export BETFAIR_CERTS_DIR=<path_to_certificate_dir>
```

:::tip
建议使用环境变量来管理您的凭证。
:::

## 概述

Betfair 适配器提供三个主要组件：

- `BetfairInstrumentProvider`：加载 Betfair 市场并将其转换为 Nautilus 金融工具（Instrument）。
- `BetfairDataClient`：通过 Exchange Streaming API 流式传输实时市场数据。
- `BetfairExecutionClient`：通过 REST API 提交订单（Order）（投注）并跟踪执行状态。

## 订单功能

Betfair 作为博彩交易所运营，与传统金融交易所相比具有独特的特性：

### 订单类型

| 订单类型               | 支持 | 说明                              |
|------------------------|------|-----------------------------------|
| `MARKET`               | -    | 不适用于博彩交易所。              |
| `LIMIT`                | ✓    | 以特定赔率下单。                  |
| `STOP_MARKET`          | -    | *不支持*。                        |
| `STOP_LIMIT`           | -    | *不支持*。                        |
| `MARKET_IF_TOUCHED`    | -    | *不支持*。                        |
| `LIMIT_IF_TOUCHED`     | -    | *不支持*。                        |
| `TRAILING_STOP_MARKET` | -    | *不支持*。                        |

### 执行指令

| 指令          | 支持 | 说明                              |
|---------------|------|-----------------------------------|
| `post_only`   | -    | 不适用于博彩交易所。              |
| `reduce_only` | -    | 不适用于博彩交易所。              |

### 有效时间选项

| 有效时间      | 支持 | 说明                              |
|---------------|------|-----------------------------------|
| `GTC`         | -    | 博彩交易所使用不同的模型。        |
| `GTD`         | -    | 博彩交易所使用不同的模型。        |
| `FOK`         | -    | 博彩交易所使用不同的模型。        |
| `IOC`         | -    | 博彩交易所使用不同的模型。        |

### 高级订单功能

| 功能             | 支持 | 说明                                    |
|------------------|------|-----------------------------------------|
| 订单修改         | ✓    | 仅限于不改变风险敞口的字段。            |
| 括号/OCO 订单    | -    | *不支持*。                              |
| 冰山订单         | -    | *不支持*。                              |

### 批量操作

| 操作             | 支持 | 说明                 |
|------------------|------|----------------------|
| 批量提交         | -    | *不支持*。           |
| 批量修改         | -    | *不支持*。           |
| 批量撤销         | -    | *不支持*。           |

### 持仓管理

| 功能              | 支持 | 说明                                  |
|-------------------|------|---------------------------------------|
| 查询持仓          | -    | 博彩交易所模型不同。                  |
| 持仓模式          | -    | 不适用于博彩交易所。                  |
| 杠杆控制          | -    | 博彩交易所无杠杆。                    |
| 保证金模式        | -    | 博彩交易所无保证金。                  |

### 订单查询

| 功能               | 支持 | 说明                                  |
|--------------------|------|---------------------------------------|
| 查询活跃订单       | ✓    | 列出所有活跃投注。                    |
| 查询订单历史       | ✓    | 历史投注数据。                        |
| 订单状态更新       | ✓    | 实时投注状态变更。                    |
| 交易历史           | ✓    | 投注撮合和结算报告。                  |

### 条件订单

| 功能              | 支持 | 说明                                  |
|-------------------|------|---------------------------------------|
| 订单列表          | -    | *不支持*。                            |
| OCO 订单          | -    | *不支持*。                            |
| 括号订单          | -    | *不支持*。                            |
| 条件订单          | -    | 仅支持基本投注条件。                  |

## 配置

### 数据客户端配置选项

| 选项                      | 默认值    | 描述 |
|---------------------------|-----------|------|
| `account_currency`        | 必填      | 用于数据和价格源的 Betfair 账户货币。 |
| `username`                | `None`    | Betfair 账户用户名；省略时从环境变量获取。 |
| `password`                | `None`    | Betfair 账户密码；省略时从环境变量获取。 |
| `app_key`                 | `None`    | 用于 API 认证的 Betfair 应用密钥。 |
| `certs_dir`               | `None`    | 包含 Betfair SSL 证书的登录目录。 |
| `instrument_config`       | `None`    | 可选的 `BetfairInstrumentProviderConfig`，用于限定可用市场范围。 |
| `subscription_delay_secs` | `3`       | 发送初始市场订阅请求前的延迟（秒）。 |
| `keep_alive_secs`         | `36,000`  | Betfair 会话的保活间隔（秒）。 |
| `stream_conflate_ms`      | `None`    | 显式流合并间隔（毫秒）（`0` 表示禁用合并）。 |
| `proxy_url`               | `None`    | 可选的 HTTP 请求代理 URL。 |

### 执行客户端配置选项

| 选项                         | 默认值   | 描述 |
|------------------------------|----------|------|
| `account_currency`           | 必填     | 用于下单和余额查询的 Betfair 账户货币。 |
| `username`                   | `None`   | Betfair 账户用户名；省略时从环境变量获取。 |
| `password`                   | `None`   | Betfair 账户密码；省略时从环境变量获取。 |
| `app_key`                    | `None`   | 用于 API 认证的 Betfair 应用密钥。 |
| `certs_dir`                  | `None`   | 包含 Betfair SSL 证书的登录目录。 |
| `instrument_config`          | `None`   | 可选的 `BetfairInstrumentProviderConfig`，用于限定对账范围。 |
| `calculate_account_state`    | `True`   | 设为 `True` 时，根据事件在本地计算账户状态。 |
| `request_account_state_secs` | `300`    | 向 Betfair 轮询账户状态的间隔（秒）（`0` 表示禁用）。 |
| `reconcile_market_ids_only`  | `False`  | 设为 `True` 时，对账请求仅覆盖已配置的市场 ID。 |
| `ignore_external_orders`     | `False`  | 设为 `True` 时，忽略本地缓存中不存在的流订单。 |
| `proxy_url`                  | `None`   | 可选的 HTTP 请求代理 URL。 |

以下是一个最小示例，展示如何使用 Betfair 客户端配置实盘 `TradingNode`：

```python
from nautilus_trader.adapters.betfair import BETFAIR
from nautilus_trader.adapters.betfair import BetfairLiveDataClientFactory
from nautilus_trader.adapters.betfair import BetfairLiveExecClientFactory
from nautilus_trader.config import TradingNodeConfig
from nautilus_trader.live.node import TradingNode

# 配置 Betfair 数据和执行客户端（使用 AUD 账户货币）
config = TradingNodeConfig(
    data_clients={BETFAIR: {"account_currency": "AUD"}},
    exec_clients={BETFAIR: {"account_currency": "AUD"}},
)

# 使用 Betfair 适配器工厂构建 TradingNode
node = TradingNode(config)
node.add_data_client_factory(BETFAIR, BetfairLiveDataClientFactory)
node.add_exec_client_factory(BETFAIR, BetfairLiveExecClientFactory)
node.build()
```

:::info
如需了解更多功能或为 Betfair 适配器做出贡献，请参阅我们的
[contributing guide](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::
