# OKX

OKX 成立于 2017 年，是一家领先的加密货币交易所，提供现货（Spot）、永续合约（Perpetual Swap）、期货（Futures）和期权（Options）交易。本集成（Integration）支持在 OKX 上进行实时市场数据接入和订单执行。

## 概述

该适配器（Adapter）使用 Rust 实现，提供可选的 Python 绑定以便在基于 Python 的工作流中使用。它不需要外部 OKX 客户端库——核心组件编译为静态库并在构建过程中自动链接。

## 示例

您可以在[此处](https://github.com/nautechsystems/nautilus_trader/tree/develop/examples/live/okx/)找到实时示例脚本。

### 产品支持

| 产品类型      | 数据推送 | 交易 | 备注                                            |
|--------------|---------|------|------------------------------------------------|
| 现货          | ✓       | ✓   | 用于获取指数价格。                                |
| 永续合约      | ✓       | ✓   | 线性和反向合约。                                  |
| 期货          | ✓       | ✓   | 特定到期日。                                     |
| 保证金        | ✓       | ✓   | 带保证金/杠杆的现货交易（现货保证金）。               |
| 期权          | ✓       | -   | *数据推送已支持，交易功能即将推出*。                  |

:::note
**期权支持**：虽然您可以订阅期权市场数据并接收价格更新，但期权的订单执行尚未实现。您可以使用上面显示的符号格式来订阅期权数据推送。
:::

:::info
**金融工具乘数**：对于衍生品（SWAP、FUTURES、OPTIONS），金融工具（Instrument）乘数计算为 OKX 的 `ctMult`（合约乘数）和 `ctVal`（合约面值）字段的乘积。这确保持仓规模准确反映合约大小和价值。
:::

OKX 适配器包含多个组件，可以根据您的使用场景单独或组合使用。

- `OKXHttpClient`：底层 HTTP API 连接。
- `OKXWebSocketClient`：底层 WebSocket API 连接。
- `OKXInstrumentProvider`：金融工具解析和加载功能。
- `OKXDataClient`：市场数据推送管理器。
- `OKXExecutionClient`：账户管理和交易执行网关。
- `OKXLiveDataClientFactory`：OKX 数据客户端工厂（由交易节点构建器使用）。
- `OKXLiveExecClientFactory`：OKX 执行客户端工厂（由交易节点构建器使用）。

:::note
大多数用户只需为实时交易节点定义配置（如下所示），无需直接使用这些底层组件。
:::

## 符号体系

OKX 对不同金融工具类型使用特定的符号约定。引用所有金融工具 ID 时应包含 `.OKX` 后缀（例如，现货比特币为 `BTC-USDT.OKX`）。

### 按金融工具类型的符号格式

#### 现货（SPOT）

格式：`{基础货币}-{报价货币}`

示例：

- `BTC-USDT` - 比特币兑 USDT (Tether)
- `BTC-USDC` - 比特币兑 USDC
- `ETH-USDT` - 以太坊兑 USDT
- `SOL-USDT` - Solana 兑 USDT

在策略中订阅现货比特币 USD：

```python
InstrumentId.from_str("BTC-USDT.OKX")  # USDT 计价的现货
InstrumentId.from_str("BTC-USDC.OKX")  # USDC 计价的现货
```

#### SWAP（永续期货）

格式：`{基础货币}-{报价货币}-SWAP`

示例：

- `BTC-USDT-SWAP` - 比特币永续合约（线性，USDT 保证金）
- `BTC-USD-SWAP` - 比特币永续合约（反向，币本位保证金）
- `ETH-USDT-SWAP` - 以太坊永续合约（线性）
- `ETH-USD-SWAP` - 以太坊永续合约（反向）

线性 vs 反向合约：

- **线性**（USDT 保证金）：使用 USDT 等稳定币作为保证金。
- **反向**（币本位保证金）：使用基础加密货币作为保证金。

#### 期货（FUTURES，交割期货）

格式：`{基础货币}-{报价货币}-{YYMMDD}`

示例：

- `BTC-USD-251226` - 2025 年 12 月 26 日到期的比特币期货
- `ETH-USD-251226` - 2025 年 12 月 26 日到期的以太坊期货
- `BTC-USD-250328` - 2025 年 3 月 28 日到期的比特币期货

注意：期货通常为反向合约（币本位保证金）。

#### 期权（OPTIONS）

格式：`{基础货币}-{报价货币}-{YYMMDD}-{行权价}-{类型}`

示例：

- `BTC-USD-251226-100000-C` - 比特币看涨期权，行权价 $100,000，2025 年 12 月 26 日到期
- `BTC-USD-251226-100000-P` - 比特币看跌期权，行权价 $100,000，2025 年 12 月 26 日到期
- `ETH-USD-251226-4000-C` - 以太坊看涨期权，行权价 $4,000，2025 年 12 月 26 日到期

其中：

- `C` = 看涨期权（Call）
- `P` = 看跌期权（Put）

### 常见问题

**Q：如何订阅现货比特币 USD？**
A：使用 `BTC-USDT.OKX` 订阅 USDT 保证金现货，或使用 `BTC-USDC.OKX` 订阅 USDC 保证金现货。

**Q：BTC-USDT-SWAP 和 BTC-USD-SWAP 有什么区别？**
A：`BTC-USDT-SWAP` 是线性永续合约（USDT 保证金），而 `BTC-USD-SWAP` 是反向永续合约（BTC 保证金）。

**Q：如何知道应该使用哪种合约类型？**
A：查看配置中的 `contract_types` 参数：

- 线性合约：`OKXContractType.LINEAR`。
- 反向合约：`OKXContractType.INVERSE`。

## 订单功能

以下是 OKX 上线性永续合约产品支持的订单类型、执行指令和有效时间选项。

### 客户端订单 ID 要求

:::note
OKX 对客户端订单 ID 有特定要求：

- **不允许使用连字符**：OKX 不接受客户端订单 ID 中的连字符（`-`）。
- 最大长度：32 个字符。
- 允许的字符：仅限字母数字字符和下划线。

配置策略时，请确保设置：

```python
use_hyphens_in_client_order_ids=False
```

:::

### 订单类型

| 订单类型            | 线性永续合约 | 备注                                                         |
|--------------------|------------|-------------------------------------------------------------|
| `MARKET`           | ✓          | 以市场价格立即执行。支持报价数量。                               |
| `LIMIT`            | ✓          | 以指定价格或更优价格执行。                                      |
| `STOP_MARKET`      | ✓          | 条件市价单（OKX 算法订单）。                                    |
| `STOP_LIMIT`       | ✓          | 条件限价单（OKX 算法订单）。                                    |
| `MARKET_IF_TOUCHED`| ✓          | 条件市价单（OKX 算法订单）。                                    |
| `LIMIT_IF_TOUCHED` | ✓          | 条件限价单（OKX 算法订单）。                                    |
| `TRAILING_STOP`    | -          | *尚未支持*。                                                  |

:::info
**条件订单**：`STOP_MARKET`、`STOP_LIMIT`、`MARKET_IF_TOUCHED` 和 `LIMIT_IF_TOUCHED` 作为 OKX 算法订单实现，提供支持多种价格源的高级触发功能。
:::

### 现货保证金交易的数量语义

在使用现货保证金交易（`use_spot_margin=True`）时，OKX 根据订单方向对订单数量有不同的解释：

- **限价**订单将 `quantity` 解释为基础货币单位数量。
- **市价卖出**订单也使用基础货币单位数量。
- **市价买入**订单将 `quantity` 解释为报价名义金额（例如 USDT）。

:::warning
**提交现货保证金市价买入订单时，您必须**：

1. 在订单上设置 `quote_quantity=True`（或预先计算以报价货币计价的金额）。
2. 配置执行引擎 `convert_quote_qty_to_base=False`，以便报价金额不被转换地传递到适配器。

OKX 执行客户端将拒绝以基础货币计价的现货保证金市价买入订单，以防止意外成交。

**在首次成交时**，订单数量将从报价数量自动更新为实际收到的基础货币数量，以反映已执行的交易。
:::

```python
from nautilus_trader.execution.config import ExecEngineConfig
from nautilus_trader.execution.engine import ExecutionEngine

# 禁用报价数量的自动转换
config = ExecEngineConfig(convert_quote_qty_to_base=False)
engine = ExecutionEngine(msgbus=msgbus, cache=cache, clock=clock, config=config)

# 正确：使用报价数量的现货保证金市价买入（花费 100 USDT）
order = strategy.order_factory.market(
    instrument_id=instrument_id,
    order_side=OrderSide.BUY,
    quantity=instrument.make_qty(100.0),
    quote_quantity=True,  # 解释为 USDT 名义金额
)
strategy.submit_order(order)
```

### 执行指令

| 指令           | 线性永续合约 | 备注                  |
|---------------|------------|----------------------|
| `post_only`   | ✓          | 仅限限价单。           |
| `reduce_only` | ✓          | 仅限衍生品。           |

### 有效时间

| 有效时间 | 线性永续合约 | 备注                                             |
|---------|------------|--------------------------------------------------|
| `GTC`   | ✓          | 撤销前有效（Good Till Canceled）。                  |
| `FOK`   | ✓          | 全部成交或撤销（Fill or Kill）。                     |
| `IOC`   | ✓          | 立即成交或撤销（Immediate or Cancel）。              |
| `GTD`   | ✗          | *OKX API 不支持。*                                 |

:::note
**GTD（到期前有效）**：OKX 不通过其 API 支持原生 GTD 功能。

如果您需要 GTD 功能，必须使用 Nautilus 的策略管理 GTD 特性，该特性将在指定的到期时间取消订单。
:::

### 批量操作

| 操作          | 线性永续合约 | 备注                                     |
|--------------|------------|------------------------------------------|
| 批量提交      | ✓          | 在单个请求中提交多个订单。                   |
| 批量修改      | ✓          | 在单个请求中修改多个订单。                   |
| 批量取消      | ✓          | 在单个请求中取消多个订单。                   |

### 持仓管理

| 功能          | 线性永续合约 | 备注                                                |
|--------------|------------|-----------------------------------------------------|
| 查询持仓      | ✓          | 实时持仓更新。                                        |
| 持仓模式      | ✓          | 净持仓 vs 多/空模式（见下文）。                         |
| 杠杆控制      | ✓          | 按金融工具动态调整杠杆。                                |
| 保证金模式    | ✓          | 支持现金、逐仓、全仓、现货逐仓模式。                     |

#### 持仓模式

OKX 为衍生品交易支持两种持仓模式：

- **净持仓模式**（Netting）：每个金融工具一个持仓，可以为正（多头）或负（空头）。买入和卖出订单相互抵消。这是默认模式，推荐大多数交易者使用。
- **多/空模式**（Hedging）：同一金融工具的多头和空头持仓分开。允许同时持有多头和空头持仓，适用于对冲策略。

:::note
持仓模式必须通过 OKX 网页/App 界面配置，适用于全账户。适配器会自动检测当前持仓模式并相应处理持仓报告。
:::

### 交易模式和保证金配置

OKX 的统一账户系统支持现货和衍生品交易的不同交易模式。适配器根据您的配置和金融工具类型自动确定正确的交易模式。

:::note
**重要**：账户模式必须首先通过 OKX 网页/App 界面进行配置。API 无法首次设置账户模式。
:::

有关 OKX 账户模式和保证金系统的更多详细信息，请参阅 [OKX 账户模式文档](https://www.okx.com/docs-v5/en/#overview-account-mode)。

#### 交易模式概述

OKX 支持四种交易模式，适配器根据您的配置自动选择：

| 模式                | 用途                                       | 杠杆 | 借贷 | 配置 |
|--------------------|--------------------------------------------|------|------|------|
| **`cash`**         | 简单现货交易                                 | -    | -    | `use_spot_margin=False`（现货默认值） |
| **`spot_isolated`**| 带保证金/杠杆的现货交易                        | ✓    | ✓    | `use_spot_margin=True` |
| **`isolated`**     | 衍生品交易（SWAP/FUTURES/OPTIONS）            | ✓    | ✓    | `margin_mode=ISOLATED` 或未设置（衍生品默认值） |
| **`cross`**        | 共享保证金池的衍生品交易                        | ✓    | ✓    | `margin_mode=CROSS` |

#### 基于配置的交易模式选择

**适配器根据以下条件自动选择正确的交易模式**：

1. **金融工具类型**（现货 vs 衍生品）
2. **配置设置**（现货使用 `use_spot_margin`，衍生品使用 `margin_mode`）

##### 现货交易

```python
# 不带杠杆的简单现货交易（使用 'cash' 模式）
exec_clients={
    OKX: OKXExecClientConfig(
        instrument_types=(OKXInstrumentType.SPOT,),
        use_spot_margin=False,  # 默认 - 简单现货
        # ... 其他配置
    ),
}

# 带保证金/杠杆的现货交易（使用 'spot_isolated' 模式）
exec_clients={
    OKX: OKXExecClientConfig(
        instrument_types=(OKXInstrumentType.SPOT,),
        use_spot_margin=True,  # 启用现货保证金交易
        # ... 其他配置
    ),
}
```

##### 衍生品交易（SWAP/FUTURES/OPTIONS）

```python
# 逐仓保证金的衍生品交易（默认 - 使用 'isolated' 模式）
exec_clients={
    OKX: OKXExecClientConfig(
        instrument_types=(OKXInstrumentType.SWAP,),
        margin_mode=OKXMarginMode.ISOLATED,  # 或省略 - ISOLATED 为默认值
        # ... 其他配置
    ),
}

# 全仓保证金的衍生品交易（使用 'cross' 模式）
exec_clients={
    OKX: OKXExecClientConfig(
        instrument_types=(OKXInstrumentType.SWAP,),
        margin_mode=OKXMarginMode.CROSS,  # 在所有持仓间共享保证金
        # ... 其他配置
    ),
}
```

##### 混合现货和衍生品交易

当同时交易现货和衍生品金融工具时，适配器根据交易的金融工具类型**按订单**自动确定正确的交易模式：

```python
# 混合现货 + SWAP 配置
exec_clients={
    OKX: OKXExecClientConfig(
        instrument_types=(OKXInstrumentType.SPOT, OKXInstrumentType.SWAP),
        use_spot_margin=True,           # 仅适用于现货订单
        margin_mode=OKXMarginMode.CROSS,  # 仅适用于 SWAP 订单
        # ... 其他配置
    ),
}
```

**工作原理：**

- **现货订单** -> 使用 `spot_isolated` 模式（因为 `use_spot_margin=True`）
- **SWAP 订单** -> 使用 `cross` 模式（因为 `margin_mode=CROSS`）
- 每个订单根据其金融工具类型自动获得正确的 `tdMode`
- 无需手动干预

这使得策略可以跨多种金融工具类型使用不同的保证金配置进行交易，例如：

- 现货-期货套利策略
- 结合现货和永续合约的 Delta 中性策略
- 跨现货和衍生品市场的做市策略

:::warning
**手动交易模式覆盖**：虽然您仍然可以使用 `params={"td_mode": "..."}` 手动覆盖每个订单的交易模式，但**不推荐**这样做，因为它绕过了自动模式选择，如果为金融工具类型指定了错误的模式（例如，对现货金融工具使用 `isolated`），可能导致订单被拒绝。

仅在有无法通过配置满足的特殊需求时才使用手动覆盖。
:::

#### 基于配置方式的优势

- **类型安全**：配置在启动时验证，在下单之前。
- **自动化**：系统根据金融工具类型和意图选择正确的模式。
- **清晰明了**：字段名称说明用途（`use_spot_margin` vs 晦涩的 `td_mode` 参数）。
- **安全可靠**：不可能使用不兼容的组合（例如，对现货使用 `isolated` 模式）。
- **向后兼容**：默认值保持现有行为。

### 订单查询

| 功能             | 线性永续合约 | 备注                                     |
|-----------------|------------|------------------------------------------|
| 查询未结订单     | ✓          | 列出所有活动订单。                          |
| 查询历史订单     | ✓          | 历史订单数据。                              |
| 订单状态更新     | ✓          | 实时订单状态变更。                           |
| 交易历史         | ✓          | 成交和填充报告。                             |

### 条件订单

| 功能            | 线性永续合约 | 备注                                      |
|----------------|------------|-------------------------------------------|
| 订单列表        | -          | *不支持*。                                  |
| OCO 订单       | ✓          | 二选一订单（One-Cancels-Other）。             |
| 组合订单        | ✓          | 止损 + 止盈组合。                            |
| 条件订单        | ✓          | 止损和触价订单。                              |

#### 条件订单架构

条件订单（OKX 算法订单）使用混合架构以获得最佳性能和可靠性：

- **提交**：通过 HTTP REST API（`/api/v5/trade/order-algo`）
- **状态更新**：通过 WebSocket 业务端点（`/ws/v5/business`）的 `orders-algo` 频道
- **取消**：通过 HTTP REST API 使用算法订单 ID 跟踪

此设计确保：

- 通过 HTTP 即时获得提交确认。
- 通过 WebSocket 获得实时状态更新。
- 通过算法订单 ID 映射进行正确的订单生命周期管理。

#### 支持的条件订单类型

| 订单类型            | 触发类型              | 备注                                     |
|--------------------|-----------------------|------------------------------------------|
| `STOP_MARKET`      | Last, Mark, Index     | 触发时以市价执行。                          |
| `STOP_LIMIT`       | Last, Mark, Index     | 触发时下限价单。                            |
| `MARKET_IF_TOUCHED`| Last, Mark, Index     | 价格触及时以市价执行。                       |
| `LIMIT_IF_TOUCHED` | Last, Mark, Index     | 价格触及时下限价单。                         |

#### 触发价格类型

条件订单支持不同的触发价格来源：

- **最新价**（`TriggerType.LAST_PRICE`）：使用最新成交价（默认）。
- **标记价**（`TriggerType.MARK_PRICE`）：使用标记价格（推荐用于衍生品）。
- **指数价**（`TriggerType.INDEX_PRICE`）：使用标的指数价格。

```python
# 示例：使用标记价触发的止损
stop_order = order_factory.stop_market(
    instrument_id=instrument_id,
    order_side=OrderSide.SELL,
    quantity=Quantity.from_str("0.1"),
    trigger_price=Price.from_str("45000.0"),
    trigger_type=TriggerType.MARK_PRICE,  # 使用标记价作为触发
)
strategy.submit_order(stop_order)
```

## 风险管理

### 强平和 ADL 事件处理

OKX 适配器自动检测和处理交易所发起的风险管理事件：

- **强平订单**：当持仓被交易所强制平仓（全部或部分）时，适配器检测强平类别并记录带有订单详情的警告。这些订单通过正常的订单和成交管道处理。
- **自动减仓（ADL）**：当您的持仓被交易所关闭以抵消对手方的强平时，适配器检测并记录带有持仓详情的 ADL 事件。

:::info
**强平和 ADL 事件以 WARNING 级别记录**，包含订单 ID、金融工具和状态等详情。请监控日志中的这些事件，作为风险管理流程的一部分。

适配器无缝处理这些交易所生成的订单，生成适当的 `OrderFilled` 事件并相应更新持仓。您的策略代码无需特殊处理。
:::

## 身份验证

要使用 OKX 适配器，您需要在 OKX 账户中创建 API 密钥（API Key）凭证：

1. 登录您的 OKX 账户并导航到 API 管理页面。
2. 创建一个新的 API 密钥，具有交易和数据访问所需的权限。
3. 记下您的 API 密钥、密钥（Secret Key）和密码短语（Passphrase）。

您可以通过环境变量提供这些凭证：

```bash
export OKX_API_KEY="your_api_key"
export OKX_API_SECRET="your_api_secret"
export OKX_API_PASSPHRASE="your_passphrase"
```

或者直接在配置中传递（不推荐在生产环境中使用）。

## 模拟交易

OKX 提供模拟交易环境，用于在不使用真实资金的情况下测试策略。

### 设置模拟账户

1. 在 [okx.com](https://www.okx.com) 登录您的 OKX 账户。
2. 导航到 **交易** -> **模拟交易**。
3. 进入模拟交易中的 **个人中心**。
4. 选择 **模拟交易 API** 并创建新的 API 密钥。
5. 记下您的模拟 API 密钥、密钥和密码短语。

您可以通过环境变量提供模拟凭证：

```bash
export OKX_API_KEY="your_demo_api_key"
export OKX_API_SECRET="your_demo_api_secret"
export OKX_API_PASSPHRASE="your_demo_passphrase"
```

### 配置

在客户端配置中设置 `is_demo=True`：

```python
config = TradingNodeConfig(
    data_clients={
        OKX: OKXDataClientConfig(
            is_demo=True,  # 启用模拟模式
            # ... 其他配置
        ),
    },
    exec_clients={
        OKX: OKXExecClientConfig(
            is_demo=True,  # 启用模拟模式
            # ... 其他配置
        ),
    },
)
```

启用模拟模式后：

- REST API 请求包含 `x-simulated-trading: 1` 请求头。
- WebSocket 连接使用模拟端点（`wspap.okx.com`）。

:::note
模拟 API 密钥与生产密钥是分开的。您必须通过模拟交易界面专门为模拟交易创建 API 密钥——生产 API 密钥在模拟模式下不起作用。
:::

## 速率限制

适配器在保持合理默认值的同时，对 REST 和 WebSocket 调用执行 OKX 的逐端点配额限制。

### REST 限制

- 全局上限：每秒 250 个请求（匹配每 2 秒 500 个请求的 IP 限额）。
- 端点特定配额见下表，反映 OKX 已发布的限制。

### WebSocket 限制

- 连接建立：每秒 3 个请求（每个 IP）。
- 订阅操作（订阅/取消订阅/登录）：每小时每连接 480 个请求。
- 订单操作（下单/取消/修改）：每秒 250 个请求。

:::warning
OKX 执行逐端点和逐账户的配额限制；超出限制将导致 HTTP 429 响应和对该密钥的临时限流。
:::

| 密钥/端点                          | 限制（请求/秒） | 备注                                                   |
|-----------------------------------|----------------|--------------------------------------------------------|
| `okx:global`                      | 250            | 匹配每 2 秒 500 个请求的 IP 限额。                       |
| `/api/v5/public/instruments`      | 10             | 匹配 OKX 文档中每 2 秒 20 个请求。                       |
| `/api/v5/market/candles`          | 50             | K 线数据流的较高限额。                                    |
| `/api/v5/market/history-candles`  | 20             | 大量历史数据拉取的保守配额。                               |
| `/api/v5/market/history-trades`   | 30             | 成交历史拉取。                                           |
| `/api/v5/account/balance`         | 5              | OKX 建议：每 2 秒 10 个请求。                             |
| `/api/v5/trade/order`             | 30             | 每金融工具每 2 秒 60 个请求的限制。                        |
| `/api/v5/trade/orders-pending`    | 20             | 获取未结订单。                                           |
| `/api/v5/trade/orders-history`    | 20             | 历史订单。                                               |
| `/api/v5/trade/fills`             | 30             | 成交报告。                                               |
| `/api/v5/trade/order-algo`        | 10             | 算法下单（条件订单）。                                     |
| `/api/v5/trade/cancel-algos`      | 10             | 算法订单取消。                                            |

所有密钥自动包含 `okx:global` 桶。URL 在速率限制前会被标准化（移除查询字符串），因此不同过滤条件的请求共享同一配额。

:::info
有关速率限制的更多详细信息，请参阅官方文档：<https://www.okx.com/docs-v5/en/#rest-api-rate-limit>。
:::

## 配置

### 配置选项

OKX 数据客户端提供以下配置选项：

#### 数据客户端

| 选项                                 | 默认值                          | 描述 |
|--------------------------------------|---------------------------------|------|
| `instrument_types`                   | `(OKXInstrumentType.SPOT,)`     | 控制加载哪些 OKX 金融工具系列（现货、合约、期货、期权）。 |
| `contract_types`                     | `None`                          | 与 `instrument_types` 结合时，限制加载特定的合约类型。 |
| `instrument_families`                | `None`                          | 要加载的金融工具系列（例如 "BTC-USD"、"ETH-USD"）。OPTIONS 必填。FUTURES/SWAP 可选。不适用于 SPOT/MARGIN。 |
| `base_url_http`                      | `None`                          | OKX REST 端点覆盖；默认使用运行时解析的生产 URL。 |
| `base_url_ws`                        | `None`                          | 市场数据 WebSocket 端点覆盖。 |
| `api_key`                            | `None`      | 未设置时回退到 `OKX_API_KEY` 环境变量。 |
| `api_secret`                         | `None`      | 未设置时回退到 `OKX_API_SECRET` 环境变量。 |
| `api_passphrase`                     | `None`      | 未设置时回退到 `OKX_PASSPHRASE` 环境变量。 |
| `is_demo`                            | `False`                         | 为 `True` 时连接到 OKX 模拟环境。 |
| `http_timeout_secs`                  | `60`                            | REST 市场数据调用的请求超时时间（秒）。 |
| `max_retries`                        | `3`                             | 可恢复 REST 错误的最大重试次数。 |
| `retry_delay_initial_ms`             | `1,000`                         | 重试失败请求前的初始延迟（毫秒）。 |
| `retry_delay_max_ms`                 | `10,000`                        | 重试间指数退避延迟的上限。 |
| `update_instruments_interval_mins`   | `60`                            | 后台金融工具刷新的间隔（分钟）。 |
| `vip_level`                          | `None`                          | 设置为匹配的 OKX VIP 等级时启用更深的订单簿频道。 |
| `http_proxy_url`                     | `None`                          | 可选的 HTTP 代理 URL。 |
| `ws_proxy_url`                       | `None`                          | 可选的 WebSocket 代理 URL。 |

OKX 执行客户端提供以下配置选项：

#### 执行客户端

| 选项                       | 默认值     | 描述 |
|---------------------------|-----------|------|
| `instrument_types`        | `(OKXInstrumentType.SPOT,)` | 该客户端可交易的金融工具系列。 |
| `contract_types`          | `None`    | 与 `instrument_types` 配合时限制可交易合约类型（线性、反向、期权）。 |
| `instrument_families`     | `None`    | 要加载的金融工具系列（例如 "BTC-USD"、"ETH-USD"）。OPTIONS 必填。FUTURES/SWAP 可选。不适用于 SPOT/MARGIN。 |
| `base_url_http`           | `None`    | OKX 交易 REST 端点覆盖。 |
| `base_url_ws`             | `None`    | 私有 WebSocket 端点覆盖。 |
| `api_key`                 | `None`    | 未设置时回退到 `OKX_API_KEY` 环境变量。 |
| `api_secret`              | `None`    | 未设置时回退到 `OKX_API_SECRET` 环境变量。 |
| `api_passphrase`          | `None`    | 未设置时回退到 `OKX_PASSPHRASE` 环境变量。 |
| `margin_mode`             | `None`    | 衍生品交易的保证金模式（`ISOLATED` 或 `CROSS`）。仅适用于 SWAP/FUTURES/OPTIONS。未指定时默认为 `ISOLATED`。 |
| `use_spot_margin`         | `False`   | 为现货交易启用保证金/杠杆。为 `True` 时使用 `spot_isolated` 交易模式。为 `False` 时使用 `cash` 交易模式（无杠杆）。仅适用于现货金融工具。 |
| `is_demo`                 | `False`   | 连接到 OKX 模拟交易环境。 |
| `http_timeout_secs`       | `60`      | REST 交易调用的请求超时时间（秒）。 |
| `use_fills_channel`       | `False`   | 订阅专用成交频道（需要 VIP5+）以获得更低延迟的成交报告。 |
| `use_mm_mass_cancel`      | `False`   | 可用时使用做市商批量取消端点；否则回退到逐单取消。 |
| `max_retries`             | `3`       | 可恢复 REST 错误的最大重试次数。 |
| `retry_delay_initial_ms`  | `1,000`   | 重试失败请求前应用的初始延迟（毫秒）。 |
| `retry_delay_max_ms`      | `10,000`  | 重试间指数退避延迟的上限。 |
| `http_proxy_url`          | `None`    | 可选的 HTTP 代理 URL。 |
| `ws_proxy_url`            | `None`    | 可选的 WebSocket 代理 URL。 |

以下是使用 OKX 数据和执行客户端的实时交易节点配置示例：

```python
from nautilus_trader.adapters.okx import OKX
from nautilus_trader.adapters.okx import OKXDataClientConfig, OKXExecClientConfig
from nautilus_trader.adapters.okx.factories import OKXLiveDataClientFactory, OKXLiveExecClientFactory
from nautilus_trader.config import InstrumentProviderConfig, LiveExecEngineConfig, LoggingConfig, TradingNodeConfig
from nautilus_trader.core.nautilus_pyo3 import OKXContractType
from nautilus_trader.core.nautilus_pyo3 import OKXInstrumentType
from nautilus_trader.core.nautilus_pyo3 import OKXMarginMode
from nautilus_trader.live.node import TradingNode

config = TradingNodeConfig(
    ...,
    data_clients={
        OKX: OKXDataClientConfig(
            api_key=None,           # 将使用 OKX_API_KEY 环境变量
            api_secret=None,        # 将使用 OKX_API_SECRET 环境变量
            api_passphrase=None,    # 将使用 OKX_API_PASSPHRASE 环境变量
            base_url_http=None,
            instrument_provider=InstrumentProviderConfig(load_all=True),
            instrument_types=(OKXInstrumentType.SWAP,),
            contract_types=(OKXContractType.LINEAR,),
            is_demo=False,
        ),
    },
    exec_clients={
        OKX: OKXExecClientConfig(
            api_key=None,
            api_secret=None,
            api_passphrase=None,
            base_url_http=None,
            base_url_ws=None,
            instrument_provider=InstrumentProviderConfig(load_all=True),
            instrument_types=(OKXInstrumentType.SWAP,),
            contract_types=(OKXContractType.LINEAR,),
            is_demo=False,
        ),
    },
)
node = TradingNode(config=config)
node.add_data_client_factory(OKX, OKXLiveDataClientFactory)
node.add_exec_client_factory(OKX, OKXLiveExecClientFactory)
node.build()
```

:::info
如需额外功能或为 OKX 适配器做贡献，请参阅我们的
[贡献指南](https://github.com/nautechsystems/nautilus_trader/blob/develop/CONTRIBUTING.md)。
:::
