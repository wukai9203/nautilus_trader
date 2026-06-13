# 金融工具 (Instruments)

`Instrument` 基类代表任何可交易资产/合约的核心规格定义。目前平台支持多个子类，涵盖一系列资产类别 (asset classes) 和工具类别 (instrument classes)：

- `Equity`：在现货市场交易的上市股票 (equity) 或 ETF。
- `CurrencyPair`：以 BASE/QUOTE 格式在现货市场交易的外汇或加密货币货币对 (currency pair)。
- `Commodity`：在现货市场交易的现货商品工具（如黄金或原油）。
- `IndexInstrument`：由成分股计算的现货指数；用作参考价格，不可直接交易。
- `FuturesContract`：具有明确标的、到期日和合约乘数 (multiplier) 的可交割期货合约 (futures contract)。
- `FuturesSpread`：交易所定义的多腿期货策略 (strategy)（如日历价差或跨品种价差），作为单一工具报价。
- `CryptoFuture`：有固定到期日、标的加密货币和结算 (settlement) 货币的有期限可交割加密期货合约。
- `CryptoPerpetual`：无到期日的永续期货合约（永续互换）；可以是反向或双币种结算。
- `OptionContract`：在标的上的交易所交易期权 (options)（看跌或看涨），具有行权价和到期日。
- `OptionSpread`：交易所定义的多腿期权策略（如垂直价差、日历价差、跨式），作为单一工具报价。
- `CryptoOption`：标的为加密货币的期权，以加密货币报价/结算；支持反向或双币种风格。
- `BinaryOption`：基于二元结果以 0 或 1 结算的固定赔付期权。
- `Cfd`：跟踪标的资产并以现金结算的场外差价合约。
- `BettingInstrument`：在博彩交易场所 (venue) 可交易的体育/游戏市场选项（如队伍或选手）。
- `SyntheticInstrument`：价格通过公式从成分工具衍生计算的合成工具。

## 代码标识 (Symbology)

所有金融工具都应具有唯一的 `InstrumentId`，由原生代码和交易场所 ID 组成，以句点分隔。
例如，在 Binance Futures 加密交易所上，以太坊永续期货合约的工具 ID 为 `ETHUSDT-PERP.BINANCE`。

所有原生代码在一个交易场所内*应该*是唯一的（但并非总是如此，例如 Binance 在现货和期货市场之间共享原生代码），
并且 `{symbol.venue}` 的组合在一个 Nautilus 系统中*必须*是唯一的。

:::warning
必须将正确的金融工具与市场数据集（如逐笔成交或订单簿数据）匹配，以确保逻辑上的正确运行。
工具规格定义不正确可能会导致数据截断或产生意外结果。
:::

:::tip 处理 Binance 现货/期货代码冲突

Binance 现货市场和期货市场共享部分相同的交易代码（如 `BTCUSDT`）。Nautilus 通过**适配器将其映射为不同的 InstrumentId** 来避免冲突：

| 市场 | InstrumentId 示例 | 备注 |
|------|----------------|------|
| Binance 现货 | `BTCUSDT.BINANCE` | 现货交易对 |
| Binance U 本位永续 | `BTCUSDT-PERP.BINANCE` | `-PERP` 后缀区分永续合约 |
| Binance 币本位永续 | `BTCUSD-PERP.BINANCE` | 币本位用 `USD` 而非 `USDT` |

如果你在同一系统中同时连接现货和期货市场，请确保使用不同的 `DataClient` 实例，并通过正确的 `client_id` 路由数据请求，以避免金融工具定义混淆。
:::

## 回测 (Backtesting)

可以通过 `TestInstrumentProvider` 实例化通用测试工具：

```python
from nautilus_trader.test_kit.providers import TestInstrumentProvider

audusd = TestInstrumentProvider.default_fx_ccy("AUD/USD")
```

可以使用适配器的 `InstrumentProvider` 从实时交易所数据中发现特定交易所的工具：

```python
from nautilus_trader.adapters.binance.spot.providers import BinanceSpotInstrumentProvider
from nautilus_trader.model import InstrumentId

provider = BinanceSpotInstrumentProvider(client=binance_http_client)
await provider.load_all_async()

btcusdt = InstrumentId.from_str("BTCUSDT.BINANCE")
instrument = provider.find(btcusdt)
```

或者由用户通过 `Instrument` 构造函数或其更具体的子类灵活定义：

```python
from nautilus_trader.model.instruments import Instrument

instrument = Instrument(...)  # <-- 提供所有必要参数
```

参见完整的金融工具 [API 参考文档](../api_reference/model/instruments.md)。

## 实盘交易 (Live trading)

实盘集成适配器定义了 `InstrumentProvider` 类，能够自动缓存交易所的最新工具定义。通过将匹配的 `InstrumentId` 传递给需要它的数据和执行相关方法及类来引用特定的 `Instrument` 对象。

## 查找金融工具

由于相同的 actor/策略类可同时用于回测和实盘交易，你可以通过中央缓存以完全相同的方式获取金融工具：

```python
from nautilus_trader.model import InstrumentId

instrument_id = InstrumentId.from_str("ETHUSDT-PERP.BINANCE")
instrument = self.cache.instrument(instrument_id)
```

也可以订阅特定金融工具的任何变更：

```python
self.subscribe_instrument(instrument_id)
```

或者订阅某个交易场所所有金融工具的变更：

```python
from nautilus_trader.model import Venue

binance = Venue("BINANCE")
self.subscribe_instruments(binance)
```

当 `DataEngine` 收到金融工具的更新时，对象将被传递给 actor/策略的 `on_instrument()` 方法。用户可以重写此方法以在收到工具更新时执行操作：

```python
from nautilus_trader.model.instruments import Instrument

def on_instrument(self, instrument: Instrument) -> None:
    # 在收到工具更新时执行某些操作
    pass
```

## 精度与增量 (Precisions and increments)

金融工具对象是通过*只读*属性组织工具规格的便捷方式。可以获取正确的价格精度 (price precision) 和数量精度 (size precision)，以及最小价格和数量增量、合约乘数和标准手数 (lot size)。

:::info 精度与增量属性详解
每个金融工具对象包含以下 6 个关键只读属性：

**精度 (Precision)** — 控制小数位数：

| 属性 | 类型 | 含义 | 示例 |
|------|------|------|------|
| `price_precision` | `int` | 价格的小数位数 | `2` → 价格如 1.01, 1.02 |
| `size_precision` | `int` | 数量的小数位数 | `8` → 数量如 0.00000001 |

**增量 (Increment)** — 控制最小步进值：

| 属性 | 类型 | 含义 | 示例 |
|------|------|------|------|
| `price_increment` | `Price` | 最小价格变动（即 tick size） | 外汇: `0.0001`, 股票: `0.01` |
| `size_increment` | `Quantity` | 最小数量步进 | 加密货币: `0.00001`, 股票: `1.0` |

精度与增量必须一致：`price_increment` 的精度等于 `price_precision`，`size_increment` 的精度等于 `size_precision`。若未指定 `price_increment`，系统自动计算为 `10^(-price_precision)`。

**合约乘数 (Multiplier)** — 决定名义价值和盈亏：

| 属性 | 类型 | 含义 | 示例 |
|------|------|------|------|
| `multiplier` | `Quantity` | 合约价值乘数 | 现货: `1.0`, CME ES 期货: `50` |

名义价值计算公式：`notional = quantity × multiplier × price`。例如 CME 标普 500 期货（ES），`multiplier = 50`，若价格为 5000，1 份合约的名义价值为 `1 × 50 × 5000 = 250,000 美元`。每变动 1 点（tick），盈亏 = `tick_size × multiplier = 0.25 × 50 = 12.50 美元`。

**标准手数 (Lot Size)** — 标准交易单位（可选）：

| 属性 | 类型 | 含义 | 示例 |
|------|------|------|------|
| `lot_size` | `Quantity` 或 `None` | 标准交易手数单位 | 黄金: `100`（盎司）, 期权: `100`（股） |

`lot_size` 是可选属性（可为 `None`），定义交易所的标准交易单位。与 `size_increment`（最小可下单步进）不同，`lot_size` 表示该工具约定俗成的"一手"大小。例如外汇中 1 标准手 = 100,000 单位基础货币，黄金期货 1 手 = 100 盎司。
:::

:::note
这些限制大多由 Nautilus `RiskEngine` 检查，否则无效的价格和数量值*可能*导致交易所拒绝订单 (order)。
:::

## 限制 (Limits)

某些值的限制对于金融工具是可选的，可以为 `None`，这取决于交易所，可能包括：

- `max_quantity`（单笔订单的最大数量）。
- `min_quantity`（单笔订单的最小数量）。
- `max_notional`（单笔订单的最大名义价值）。
- `min_notional`（单笔订单的最小名义价值）。
- `max_price`（最大有效报价或订单价格）。
- `min_price`（最小有效报价或订单价格）。

:::note
这些限制大多由 Nautilus `RiskEngine` 检查，否则超出发布的限制*可能*导致交易所拒绝订单。
:::

## 价格与数量

金融工具对象还提供了根据给定值创建正确价格和数量的便捷方式。

```python
instrument = self.cache.instrument(instrument_id)

price = instrument.make_price(0.90500)
quantity = instrument.make_qty(150)
```

:::tip
以上是创建有效价格和数量的推荐方法，例如将它们传递给订单工厂以创建订单时。
:::

## 保证金与费用 (Margins and fees)

保证金 (margin) 计算由 `MarginAccount` 类处理。本节介绍保证金的工作原理并引入需要了解的关键概念。

### 保证金何时适用？

每个交易所（如 CME 或 Binance）使用特定的账户类型来决定是否适用保证金计算。
在设置交易所时，你需要指定以下账户类型之一：

- `AccountType.MARGIN`：使用保证金计算的账户，具体说明见下文。
- `AccountType.CASH`：不适用保证金计算的简单账户。
- `AccountType.BETTING`：专为博彩设计的账户，同样不涉及保证金计算。

### 术语

要理解保证金交易，首先了解一些关键术语：

**名义价值 (Notional Value)**：以报价货币计算的合约总价值。它代表你持仓 (position) 的全部市场价值。例如，CME 上的欧元/美元期货（代码 6E）：

- 每份合约代表 125,000 欧元（欧元为基础货币，美元为报价货币）。
- 如果当前市场价格为 1.1000，名义价值等于 125,000 欧元 x 1.1000（欧元/美元价格）= 137,500 美元。

**杠杆 (Leverage)**（`leverage`）：决定你相对于账户存款可控制多少市场敞口的比率。例如，使用 10 倍杠杆，你可以用 1,000 美元的账户控制价值 10,000 美元的持仓。

**初始保证金 (Initial Margin)**（`margin_init`）：开仓所需的保证金费率。它代表账户中必须可用的最低资金才能开新仓。这只是预检查 -- 实际上不会锁定资金。

**维持保证金 (Maintenance Margin)**（`margin_maint`）：维持持仓所需的保证金费率。该金额在你的账户中被锁定以维持持仓。它始终低于初始保证金。你可以在策略中使用以下代码查看被锁定的总资金（所有未平仓持仓的维持保证金之和）：

```python
self.portfolio.balances_locked(venue)
```

**Maker/Taker 费用**：交易所根据你的订单与市场的交互方式收取的费用：

- Maker 费用（`maker_fee`）：当你通过下达留在订单簿上的订单来"提供"流动性时收取的费用（通常较低）。例如，低于当前价格的限价买入订单增加了流动性，成交时适用 *maker* 费用。
- Taker 费用（`taker_fee`）：当你通过下达立即执行的订单来"消耗"流动性时收取的费用（通常较高）。例如，市价买入订单或高于当前价格的限价买入订单消耗了流动性，适用 *taker* 费用。

**费率符号约定**：Nautilus 在所有适配器和回测引擎中使用一致的费率符号约定：

- **正费率** = 佣金（收取费用，减少账户余额）。
- **负费率** = 返佣（获得费用，增加账户余额）。

例如，maker 费用为 `-0.00025` 表示你因提供流动性而获得 0.025% 的返佣，而 taker 费用为 `0.00075` 表示你因消耗流动性而支付 0.075% 的佣金。

:::note
不同交易所在其 API 中使用不同的符号约定。Nautilus 适配器会将这些标准化为上述约定。如果你在回测中手动指定费率，请确保遵循此约定。
:::

:::tip
并非所有交易所或工具都实现了 maker/taker 费用。如果没有，请将 `Instrument`（如 `FuturesContract`、`Equity`、`CurrencyPair`、`Commodity`、`Cfd`、`BinaryOption`、`BettingInstrument`）的 `maker_fee` 和 `taker_fee` 都设为 0。
:::

### 保证金计算公式

`MarginAccount` 类使用以下公式计算保证金：

```python
# 初始保证金计算
margin_init = (notional_value / leverage * margin_init) + (notional_value / leverage * taker_fee)

# 维持保证金计算
margin_maint = (notional_value / leverage * margin_maint) + (notional_value / leverage * taker_fee)
```

**要点**：

- 两个公式遵循相同的结构，但使用各自的保证金费率（`margin_init` 和 `margin_maint`）。
- 每个公式由两部分组成：
  - **主保证金计算**：基于名义价值、杠杆和保证金费率。
  - **费用调整**：考虑 maker/taker 费用。

### 实现细节

如果你有兴趣探索技术实现：

- [nautilus_trader/accounting/accounts/margin.pyx](https://github.com/nautechsystems/nautilus_trader/blob/develop/nautilus_trader/accounting/accounts/margin.pyx)
- 关键方法：`calculate_margin_init(self, ...)` 和 `calculate_margin_maint(self, ...)`

## 佣金 (Commissions)

交易佣金代表交易所或经纪商为执行交易而收取的费用。
虽然 maker/taker 费用在加密货币市场很常见，但像 CME 这样的传统交易所通常
采用其他费用结构，例如按合约收取佣金。
NautilusTrader 支持多种佣金模型，以适应不同市场的各种费用结构。

### 内置费用模型

框架提供了两种内置的费用模型实现：

1. `MakerTakerFeeModel`：实现加密货币交易所常见的 maker/taker 费用结构，费用按交易价值的百分比计算。
2. `FixedFeeModel`：无论交易规模如何，按每笔交易收取固定佣金。

### 创建自定义费用模型

虽然内置费用模型涵盖了常见场景，但你可能会遇到需要特定佣金结构的情况。
NautilusTrader 灵活的架构允许你通过继承基础 `FeeModel` 类来实现自定义费用模型。

例如，如果你在按合约收取佣金的交易所（如 CME）交易期货，可以实现自定义费用模型。创建自定义费用模型时，我们从 `FeeModel` 基类继承，该基类出于性能考虑使用 Cython 实现。这种 Cython 实现反映在参数命名约定中，
其中类型信息通过下划线合并到参数名中（如 `Order_order` 或 `Quantity_fill_qty`）。

虽然这些参数名对 Python 开发者来说可能看起来不太常见，但它们是 Cython 类型系统的产物，有助于与框架核心组件保持一致。以下是创建按合约佣金模型的方法：

```python
class PerContractFeeModel(FeeModel):
    def __init__(self, commission: Money):
        super().__init__()
        self.commission = commission

    def get_commission(self, Order_order, Quantity_fill_qty, Price_fill_px, Instrument_instrument):
        total_commission = Money(self.commission * Quantity_fill_qty, self.commission.currency)
        return total_commission
```

此自定义实现通过将`固定的每合约费用`乘以`交易的合约数量`来计算总佣金。`get_commission(...)` 方法接收订单、成交数量、成交价格和金融工具的信息，允许基于这些参数进行灵活的佣金计算。

我们的新类 `PerContractFeeModel` 继承了 `FeeModel` 类（该类使用 Cython 实现），
因此请注意方法签名中的 Cython 风格参数名：

- `Order_order`：订单对象，类型前缀为 `Order_`。
- `Quantity_fill_qty`：成交数量，类型前缀为 `Quantity_`。
- `Price_fill_px`：成交价格，类型前缀为 `Price_`。
- `Instrument_instrument`：金融工具对象，类型前缀为 `Instrument_`。

这些参数名遵循 NautilusTrader 的 Cython 命名约定，其中前缀表示预期的类型。
虽然与典型的 Python 命名约定相比可能显得冗长，但它确保了与框架 Cython 代码库的类型安全性和一致性。

### 在实践中使用费用模型

要在你的交易系统中使用任何费用模型（无论是内置的还是自定义的），请在设置交易场所时指定。
以下是使用自定义按合约费用模型的示例：

```python
from nautilus_trader.model.currencies import USD
from nautilus_trader.model.objects import Money, Currency

engine.add_venue(
    venue=venue,
    oms_type=OmsType.NETTING,
    account_type=AccountType.MARGIN,
    base_currency=USD,
    fee_model=PerContractFeeModel(Money(2.50, USD)),  # 每合约 2.50 美元
    starting_balances=[Money(1_000_000, USD)],  # 起始余额 1,000,000 美元
)
```

:::tip
实现自定义费用模型时，请确保它们准确反映目标交易所的费用结构。
即使佣金计算中的微小差异也可能在回测期间显著影响策略绩效指标。
:::

### 附加信息

交易所提供的原始工具定义（通常来自 JSON 序列化数据）也作为通用 Python 字典包含在内。这是为了保留所有不一定属于统一 Nautilus API 的信息，用户可以在运行时通过调用 `.info` 属性获取。

## 合成工具 (Synthetic instruments)

平台支持创建自定义合成工具，可以生成合成行情报价和成交数据。这些工具适用于：

- 使 `Actor` 和 `Strategy` 组件能够订阅行情报价或成交数据流。
- 触发模拟订单。
- 从合成报价或成交数据构建 K 线。

合成工具不能直接交易，因为它们是仅存在于平台本地的构造。它们作为分析工具，基于其成分工具提供有用的指标。

未来我们计划支持合成工具的订单管理，使得能够基于合成工具的行为交易其成分工具。

:::info
合成工具的交易场所始终指定为 `'SYNTH'`。
:::

### 公式

合成工具由两个或多个成分工具的组合（可以包含来自多个交易场所的工具）以及一个"衍生公式"组成。
利用由 [evalexpr](https://github.com/ISibboI/evalexpr) Rust crate 驱动的动态表达式引擎，平台可以计算公式以从传入的成分工具价格计算最新的合成价格。

请参阅 `evalexpr` 文档了解可用功能、运算符和优先级的完整描述。

:::tip
在定义新的合成工具之前，请确保所有成分工具已经定义并存在于缓存中。
:::

### 订阅

以下示例演示了如何通过 actor/策略创建新的合成工具。
此合成工具将表示 Binance 上比特币和以太坊现货价格之间的简单价差。对于此示例，假设 `BTCUSDT.BINANCE` 和 `ETHUSDT.BINANCE` 的现货工具已存在于缓存中。

```python
from nautilus_trader.model.instruments import SyntheticInstrument

btcusdt_binance_id = InstrumentId.from_str("BTCUSDT.BINANCE")
ethusdt_binance_id = InstrumentId.from_str("ETHUSDT.BINANCE")

# 定义合成工具
synthetic = SyntheticInstrument(
    symbol=Symbol("BTC-ETH:BINANCE"),
    price_precision=8,
    components=[
        btcusdt_binance_id,
        ethusdt_binance_id,
    ],
    formula=f"{btcusdt_binance_id} - {ethusdt_binance_id}",
    ts_event=self.clock.timestamp_ns(),
    ts_init=self.clock.timestamp_ns(),
)

# 建议将合成工具的 ID 存储在某处
self._synthetic_id = synthetic.id

# 添加合成工具以供其他组件使用
self.add_synthetic(synthetic)

# 订阅合成工具的行情报价
self.subscribe_quote_ticks(self._synthetic_id)
```

:::note
上述示例中合成工具的 `instrument_id` 将构造为 `{symbol}.{SYNTH}`，即 `'BTC-ETH:BINANCE.SYNTH'`。
:::

### 更新公式

也可以随时更新合成工具的公式。以下示例展示了如何通过 actor/策略实现此操作。

```python
# 从缓存中恢复合成工具（假设 `synthetic_id` 已赋值）
synthetic = self.cache.synthetic(self._synthetic_id)

# 将公式更新为取平均值
new_formula = "(BTCUSDT.BINANCE + ETHUSDT.BINANCE) / 2"
synthetic.change_formula(new_formula)

# 现在更新合成工具
self.update_synthetic(synthetic)
```

### 触发工具 ID (Trigger instrument IDs)

平台允许基于合成工具价格触发模拟订单。在以下示例中，我们在前面的基础上提交一个新的模拟订单。
此订单将保留在模拟器中，直到来自合成报价的触发将其释放。
然后它将作为 MARKET 订单提交到 Binance：

```python
order = self.strategy.order_factory.limit(
    instrument_id=ETHUSDT_BINANCE.id,
    order_side=OrderSide.BUY,
    quantity=Quantity.from_str("1.5"),
    price=Price.from_str("30000.00000000"),  # <-- 合成工具价格
    emulation_trigger=TriggerType.DEFAULT,
    trigger_instrument_id=self._synthetic_id,  # <-- 合成工具标识符
)

self.strategy.submit_order(order)
```

### 错误处理

已投入大量努力来验证输入，包括合成工具的衍生公式。
尽管如此，仍需谨慎，因为无效或错误的输入可能导致未定义的行为。

:::info
参见 `SyntheticInstrument` [API 参考文档](../api_reference/model/instruments.md#class-syntheticinstrument-1) 以详细了解输入要求和可能的异常。
:::
