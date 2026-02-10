# 策略 (Strategies)

NautilusTrader 用户体验的核心在于编写和使用交易策略 (strategy)。
定义策略需要继承 `Strategy` 类，并实现策略逻辑所需的方法。

**核心能力**：

- 所有 `Actor` 的能力。
- 订单 (order) 管理。

**与 Actor 的关系**：
`Strategy` 类继承自 `Actor`，这意味着策略拥有 Actor 的全部功能，外加订单管理能力。

:::tip
我们建议在深入策略开发之前，先阅读 [Actors](actors.md) 指南。
:::

策略可以在任何[环境上下文](/concepts/architecture.md#environment-contexts)中添加到 Nautilus 系统，系统启动后会立即根据策略逻辑开始发送命令和接收事件。

利用数据 (data) 摄取、事件处理和订单管理这些基本构建模块（我们将在下文讨论），可以实现任何类型的策略，包括方向性、动量、再平衡、配对交易、做市等。

:::info
请参阅 `Strategy` [API 参考](../api_reference/trading.md)，获取所有可用方法的完整说明。
:::

Nautilus 交易策略由两个主要部分组成：

- 策略实现本身，通过继承 `Strategy` 类来定义。
- *可选的* 策略配置 (configuration)，通过继承 `StrategyConfig` 类来定义。

:::tip
一旦策略定义完成，相同的源代码既可用于回测 (backtest)，也可用于实盘交易 (live trading)。
:::

策略的主要能力包括：

- 历史数据请求。
- 实时数据订阅 (subscription)。
- 设置时间提醒或定时器。
- 缓存 (cache) 访问。
- 投资组合 (portfolio) 访问。
- 创建和管理订单与持仓 (position)。

## 策略实现

由于交易策略是继承自 `Strategy` 的类，你必须定义一个构造函数来处理初始化。至少需要初始化基类/父类：

```python
from nautilus_trader.trading.strategy import Strategy

class MyStrategy(Strategy):
    def __init__(self) -> None:
        super().__init__()  # <-- 必须调用父类来初始化策略
```

在此基础上，你可以根据需要实现处理器 (handler)，以便根据状态 (state) 转换和事件执行相应操作。

:::warning
不要在 `__init__` 构造函数中调用 `clock` 和 `logger` 等组件 (component)（此时尚未注册）。
这是因为系统时钟和日志子系统尚未初始化。
:::

### 处理器

处理器是 `Strategy` 类中的方法，可根据不同类型的事件或状态变化执行相应操作。
这些方法以 `on_*` 为前缀命名。你可以根据策略的具体目标和需求，选择实现其中任意或全部处理器方法。

为类似类型的事件提供多个处理器的目的是提供处理粒度上的灵活性。
这意味着你可以选择使用专用处理器响应特定事件，也可以使用更通用的处理器来响应一系列相关事件（使用典型的 switch 语句逻辑）。
处理器按照从最具体到最通用的顺序依次调用。

#### 状态动作

这些处理器由 `Strategy` 的生命周期 (lifecycle) 状态变化触发。建议：

- 使用 `on_start` 方法初始化策略（例如获取金融工具 (instrument)、订阅数据）。
- 使用 `on_stop` 方法执行清理任务（例如取消未完成订单、关闭已开持仓、取消数据订阅）。

```python
def on_start(self) -> None:
def on_stop(self) -> None:
def on_resume(self) -> None:
def on_reset(self) -> None:
def on_dispose(self) -> None:
def on_degrade(self) -> None:
def on_fault(self) -> None:
def on_save(self) -> dict[str, bytes]:  # 返回用户自定义的状态字典用于保存
def on_load(self, state: dict[str, bytes]) -> None:
```

#### 数据处理

这些处理器接收数据更新，包括内置的行情数据 (market data) 和用户自定义数据。
你可以使用这些处理器定义收到数据对象实例时的操作。

```python
from nautilus_trader.core import Data
from nautilus_trader.model import OrderBook
from nautilus_trader.model import Bar
from nautilus_trader.model import QuoteTick
from nautilus_trader.model import TradeTick
from nautilus_trader.model import OrderBookDeltas
from nautilus_trader.model import InstrumentClose
from nautilus_trader.model import InstrumentStatus
from nautilus_trader.model.instruments import Instrument

def on_order_book_deltas(self, deltas: OrderBookDeltas) -> None:
def on_order_book(self, order_book: OrderBook) -> None:
def on_quote_tick(self, tick: QuoteTick) -> None:
def on_trade_tick(self, tick: TradeTick) -> None:
def on_bar(self, bar: Bar) -> None:
def on_instrument(self, instrument: Instrument) -> None:
def on_instrument_status(self, data: InstrumentStatus) -> None:
def on_instrument_close(self, data: InstrumentClose) -> None:
def on_historical_data(self, data: Data) -> None:
def on_data(self, data: Data) -> None:  # 自定义数据传递给此处理器
def on_signal(self, signal: Data) -> None:  # 自定义信号传递给此处理器
```

#### 订单管理

这些处理器接收与订单相关的事件。
`OrderEvent` 类型消息按以下顺序传递给处理器：

1. 特定处理器（例如 `on_order_accepted`、`on_order_rejected` 等）
2. `on_order_event(...)`
3. `on_event(...)`

```python
from nautilus_trader.model.events import OrderAccepted
from nautilus_trader.model.events import OrderCanceled
from nautilus_trader.model.events import OrderCancelRejected
from nautilus_trader.model.events import OrderDenied
from nautilus_trader.model.events import OrderEmulated
from nautilus_trader.model.events import OrderEvent
from nautilus_trader.model.events import OrderExpired
from nautilus_trader.model.events import OrderFilled
from nautilus_trader.model.events import OrderInitialized
from nautilus_trader.model.events import OrderModifyRejected
from nautilus_trader.model.events import OrderPendingCancel
from nautilus_trader.model.events import OrderPendingUpdate
from nautilus_trader.model.events import OrderRejected
from nautilus_trader.model.events import OrderReleased
from nautilus_trader.model.events import OrderSubmitted
from nautilus_trader.model.events import OrderTriggered
from nautilus_trader.model.events import OrderUpdated

def on_order_initialized(self, event: OrderInitialized) -> None:
def on_order_denied(self, event: OrderDenied) -> None:
def on_order_emulated(self, event: OrderEmulated) -> None:
def on_order_released(self, event: OrderReleased) -> None:
def on_order_submitted(self, event: OrderSubmitted) -> None:
def on_order_rejected(self, event: OrderRejected) -> None:
def on_order_accepted(self, event: OrderAccepted) -> None:
def on_order_canceled(self, event: OrderCanceled) -> None:
def on_order_expired(self, event: OrderExpired) -> None:
def on_order_triggered(self, event: OrderTriggered) -> None:
def on_order_pending_update(self, event: OrderPendingUpdate) -> None:
def on_order_pending_cancel(self, event: OrderPendingCancel) -> None:
def on_order_modify_rejected(self, event: OrderModifyRejected) -> None:
def on_order_cancel_rejected(self, event: OrderCancelRejected) -> None:
def on_order_updated(self, event: OrderUpdated) -> None:
def on_order_filled(self, event: OrderFilled) -> None:
def on_order_event(self, event: OrderEvent) -> None:  # 所有订单事件消息最终都会传递给此处理器
```

#### 持仓管理

这些处理器接收与持仓相关的事件。
`PositionEvent` 类型消息按以下顺序传递给处理器：

1. 特定处理器（例如 `on_position_opened`、`on_position_changed` 等）
2. `on_position_event(...)`
3. `on_event(...)`

```python
from nautilus_trader.model.events import PositionChanged
from nautilus_trader.model.events import PositionClosed
from nautilus_trader.model.events import PositionEvent
from nautilus_trader.model.events import PositionOpened

def on_position_opened(self, event: PositionOpened) -> None:
def on_position_changed(self, event: PositionChanged) -> None:
def on_position_closed(self, event: PositionClosed) -> None:
def on_position_event(self, event: PositionEvent) -> None:  # 所有持仓事件消息最终都会传递给此处理器
```

#### 通用事件处理

此处理器最终会接收到达策略的所有事件消息，包括没有其他特定处理器的事件。

```python
from nautilus_trader.core.message import Event

def on_event(self, event: Event) -> None:
```

#### 处理器示例

以下示例展示了一个典型的 `on_start` 处理器方法实现（取自 EMA 交叉策略示例）。
我们可以看到以下内容：

- 注册指标 (indicator) 以接收 K线 (Bar) 更新。
- 请求历史数据（用于填充指标）。
- 订阅实时数据。

```python
def on_start(self) -> None:
    """
    策略启动时执行的操作。
    """
    self.instrument = self.cache.instrument(self.instrument_id)
    if self.instrument is None:
        self.log.error(f"Could not find instrument for {self.instrument_id}")
        self.stop()  # 将策略转换为 STOPPED 状态
        return

    # 注册指标以接收更新
    self.register_indicator_for_bars(self.bar_type, self.fast_ema)
    self.register_indicator_for_bars(self.bar_type, self.slow_ema)

    # 获取历史数据
    self.request_bars(self.bar_type)

    # 订阅实时数据
    self.subscribe_bars(self.bar_type)
    self.subscribe_quote_ticks(self.instrument_id)
```

### 时钟与定时器

策略可以访问一个 `Clock`，它提供了多种方法来创建不同的时间戳，以及设置时间提醒或定时器来触发 `TimeEvent`。

:::info
请参阅 `Clock` [API 参考](../api_reference/common.md)，获取所有可用方法的完整列表。
:::

#### 当前时间戳

虽然有多种方式获取当前时间戳，以下是两个常用方法示例：

获取当前 UTC 时间戳（带时区的 `pd.Timestamp`）：

```python
import pandas as pd


now: pd.Timestamp = self.clock.utc_now()
```

获取当前 UTC 时间戳（自 UNIX 纪元以来的纳秒数）：

```python
unix_nanos: int = self.clock.timestamp_ns()
```

#### 时间提醒

可以设置时间提醒，在指定的提醒时间向 `on_event` 处理器分发一个 `TimeEvent`。在实盘环境中，可能会有几微秒的轻微延迟。

以下示例设置了一个从当前时间起一分钟后触发的时间提醒：

```python
import pandas as pd

# 从现在起一分钟后触发一个 TimeEvent
self.clock.set_time_alert(
    name="MyTimeAlert1",
    alert_time=self.clock.utc_now() + pd.Timedelta(minutes=1),
)
```

#### 定时器

可以设置连续定时器，按固定间隔生成 `TimeEvent`，直到定时器到期或被取消。

以下示例设置了一个每分钟触发一次、立即开始的定时器：

```python
import pandas as pd

# 每分钟触发一个 TimeEvent
self.clock.set_timer(
    name="MyTimer1",
    interval=pd.Timedelta(minutes=1),
)
```

### 缓存访问

可以访问交易实例的中央 `Cache` 来获取数据和执行对象（订单、持仓等）。
有许多可用方法，通常带有过滤功能，这里我们介绍一些基本用例。

#### 获取数据

以下示例展示了如何从缓存中获取数据（假设已分配了某个金融工具 ID 属性）。
如果请求的数据不可用，这些方法返回 `None`。

```python
last_quote = self.cache.quote_tick(self.instrument_id)
last_trade = self.cache.trade_tick(self.instrument_id)
last_bar = self.cache.bar(bar_type)
```

#### 获取执行对象

以下示例展示了如何从缓存中获取单个订单和持仓对象：

```python
order = self.cache.order(client_order_id)
position = self.cache.position(position_id)

```

:::info
请参阅 `Cache` [API 参考](../api_reference/cache.md)，获取所有可用方法的完整说明。
:::

### 投资组合访问

可以访问交易的中央 `Portfolio` 来获取账户和持仓信息。
以下展示了可用方法的概览。

#### 账户和持仓信息

```python
import decimal

from nautilus_trader.accounting.accounts.base import Account
from nautilus_trader.model import Venue
from nautilus_trader.model import Currency
from nautilus_trader.model import Money
from nautilus_trader.model import InstrumentId

def account(self, venue: Venue) -> Account

def balances_locked(self, venue: Venue) -> dict[Currency, Money]
def margins_init(self, venue: Venue) -> dict[Currency, Money]
def margins_maint(self, venue: Venue) -> dict[Currency, Money]
def unrealized_pnls(self, venue: Venue) -> dict[Currency, Money]
def realized_pnls(self, venue: Venue) -> dict[Currency, Money]
def net_exposures(self, venue: Venue) -> dict[Currency, Money]

def unrealized_pnl(self, instrument_id: InstrumentId) -> Money
def realized_pnl(self, instrument_id: InstrumentId) -> Money
def net_exposure(self, instrument_id: InstrumentId) -> Money
def net_position(self, instrument_id: InstrumentId) -> decimal.Decimal

def is_net_long(self, instrument_id: InstrumentId) -> bool
def is_net_short(self, instrument_id: InstrumentId) -> bool
def is_flat(self, instrument_id: InstrumentId) -> bool
def is_completely_flat(self) -> bool
```

:::info
请参阅 `Portfolio` [API 参考](../api_reference/portfolio.md)，获取所有可用方法的完整说明。
:::

#### 报告与分析

`Portfolio` 还提供了一个 `PortfolioAnalyzer`，可以输入灵活数量的数据（以适应不同的回溯窗口）。分析器可以提供绩效指标和统计数据的跟踪和生成。

:::info
请参阅 `PortfolioAnalyzer` [API 参考](../api_reference/analysis.md)，获取所有可用方法的完整说明。
:::

:::info
请参阅[投资组合统计](portfolio.md#portfolio-statistics)指南。
:::

### 交易命令

NautilusTrader 提供了一套全面的交易命令，支持为算法交易量身定制的精细订单管理。这些命令对于执行策略、管理风险 (risk) 以及确保与各交易场所 (venue) 的无缝交互至关重要。在以下章节中，我们将深入探讨每个命令及其用例。

:::info
[执行](../concepts/execution.md) (Execution) 指南解释了系统中的流程，结合以下内容阅读会很有帮助。
:::

#### 提交订单

每个 `Strategy` 的基类上都提供了一个 `OrderFactory` 作为便利工具，减少了创建不同 `Order` 对象所需的样板代码（不过如果交易者愿意，也可以直接使用 `Order.__init__(...)` 构造函数初始化这些对象）。

`SubmitOrder` 或 `SubmitOrderList` 命令将流向哪个组件执行，取决于以下条件：

- 如果指定了 `emulation_trigger`，命令将*首先*发送到 `OrderEmulator`。
- 如果指定了 `exec_algorithm_id`（且没有 `emulation_trigger`），命令将*首先*发送到相应的 `ExecAlgorithm`。
- 否则，命令将*首先*发送到 `RiskEngine`。

以下示例提交一个用于模拟的 `LIMIT` 买入订单（参见[模拟订单](orders.md#emulated-orders)）：

```python
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.enums import TriggerType
from nautilus_trader.model.orders import LimitOrder


def buy(self) -> None:
    """
    用户的简单买入方法（示例）。
    """
    order: LimitOrder = self.order_factory.limit(
        instrument_id=self.instrument_id,
        order_side=OrderSide.BUY,
        quantity=self.instrument.make_qty(self.trade_size),
        price=self.instrument.make_price(5000.00),
        emulation_trigger=TriggerType.LAST_PRICE,
    )

    self.submit_order(order)
```

:::info
你可以同时指定订单模拟和执行算法。在这种情况下，订单首先发送到 `OrderEmulator`，释放后再路由到 `ExecAlgorithm`。
:::

以下示例向 TWAP 执行算法提交一个 `MARKET` 买入订单：

```python
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.enums import TimeInForce
from nautilus_trader.model import ExecAlgorithmId


def buy(self) -> None:
    """
    用户的简单买入方法（示例）。
    """
    order: MarketOrder = self.order_factory.market(
        instrument_id=self.instrument_id,
        order_side=OrderSide.BUY,
        quantity=self.instrument.make_qty(self.trade_size),
        time_in_force=TimeInForce.FOK,
        exec_algorithm_id=ExecAlgorithmId("TWAP"),
        exec_algorithm_params={"horizon_secs": 20, "interval_secs": 2.5},
    )

    self.submit_order(order)
```

#### 取消订单

订单可以单独取消、批量取消，或取消某个金融工具的所有订单（可选方向过滤）。

如果订单已经*关闭*或已在等待取消中，则会记录一条警告。

如果订单当前处于*开放*状态，其状态将变为 `PENDING_CANCEL`。

`CancelOrder`、`CancelAllOrders` 或 `BatchCancelOrders` 命令将流向哪个组件执行，取决于以下条件：

- 如果订单当前处于模拟状态，命令将*首先*发送到 `OrderEmulator`。
- 如果指定了 `exec_algorithm_id`（且没有 `emulation_trigger`），并且订单仍在本地系统中活跃，命令将*首先*发送到相应的 `ExecAlgorithm`。
- 否则，订单将*首先*发送到 `ExecutionEngine`。

:::info
命令离开策略后，任何托管 (managed) 的 GTD 定时器也会被取消。
:::

以下展示如何取消单个订单：

```python

self.cancel_order(order)

```

以下展示如何批量取消订单：

```python
from nautilus_trader.model.orders import Order


my_order_list: list[Order] = [order1, order2, order3]
self.cancel_orders(my_order_list)

```

以下展示如何取消所有订单：

```python

self.cancel_all_orders()

```

#### 修改订单

当订单处于模拟状态，或在交易场所上*开放*时（如果支持），可以单独修改订单。

如果订单已经*关闭*或已在等待取消中，则会记录一条警告。
如果订单当前处于*开放*状态，其状态将变为 `PENDING_UPDATE`。

:::warning
至少需要有一个值与原始订单不同，命令才有效。
:::

`ModifyOrder` 命令将流向哪个组件执行，取决于以下条件：

- 如果订单当前处于模拟状态，命令将*首先*发送到 `OrderEmulator`。
- 否则，订单将*首先*发送到 `RiskEngine`。

:::info
一旦订单处于执行算法的控制之下，策略就无法直接修改它（只能取消）。
:::

以下展示如何修改当前在交易场所上*开放*的 `LIMIT` 买入订单的数量：

```python
from nautilus_trader.model import Quantity


new_quantity: Quantity = Quantity.from_int(5)
self.modify_order(order, new_quantity)

```

:::info
价格和触发价格也可以修改（当处于模拟状态或交易场所支持时）。
:::

## 策略配置

独立配置类的主要目的是为交易策略的实例化提供完全的灵活性，包括在何处以及如何实例化。这包括能够通过网络序列化策略及其配置，使分布式回测和远程启动实盘交易成为可能。

这种配置灵活性实际上是可选的，你可以选择不使用任何策略配置，只使用你传入策略构造函数的参数。如果你想运行分布式回测或远程启动实盘交易服务器，那么你需要定义一个配置。

以下是一个配置示例：

```python
from decimal import Decimal
from nautilus_trader.config import StrategyConfig
from nautilus_trader.model import Bar, BarType
from nautilus_trader.model import InstrumentId
from nautilus_trader.trading.strategy import Strategy


# 配置定义
class MyStrategyConfig(StrategyConfig):
    instrument_id: InstrumentId   # 示例值: "ETHUSDT-PERP.BINANCE"
    bar_type: BarType             # 示例值: "ETHUSDT-PERP.BINANCE-15-MINUTE[LAST]-EXTERNAL"
    fast_ema_period: int = 10
    slow_ema_period: int = 20
    trade_size: Decimal
    order_id_tag: str


# 策略定义
class MyStrategy(Strategy):
    def __init__(self, config: MyStrategyConfig) -> None:
        # 始终初始化父类 Strategy
        # 此后，配置已存储并可通过 `self.config` 访问
        super().__init__(config)

        # 自定义状态变量
        self.time_started = None
        self.count_of_processed_bars: int = 0

    def on_start(self) -> None:
        self.time_started = self.clock.utc_now()    # 记录策略启动时间
        self.subscribe_bars(self.config.bar_type)   # 通过 `self.config` 访问配置数据

    def on_bar(self, bar: Bar):
        self.count_of_processed_bars += 1           # 更新已处理的 K线 计数


# 使用具体值实例化配置。通过设置：
#   - InstrumentId - 参数化策略将交易的金融工具。
#   - BarType - 参数化策略将使用的 K线 数据。
config = MyStrategyConfig(
    instrument_id=InstrumentId.from_str("ETHUSDT-PERP.BINANCE"),
    bar_type=BarType.from_str("ETHUSDT-PERP.BINANCE-15-MINUTE[LAST]-EXTERNAL"),
    trade_size=Decimal(1),
    order_id_tag="001",
)

# 将配置传递给交易策略。
strategy = MyStrategy(config=config)
```

在实现策略时，建议直接通过 `self.config` 访问配置值。
这提供了清晰的分离：

- 配置数据（通过 `self.config` 访问）：
  - 包含定义策略工作方式的初始设置。
  - 例如：`self.config.trade_size`、`self.config.instrument_id`

- 策略状态变量（作为直接属性）：
  - 跟踪策略的任何自定义状态。
  - 例如：`self.time_started`、`self.count_of_processed_bars`

这种分离使代码更易于理解和维护。

:::note
尽管定义交易单一金融工具的策略通常是合理的，但单个策略可以处理的金融工具数量仅受机器资源限制。
:::

### 托管 GTD 过期

策略可以为具有 GTD（*Good 'till Date*，到期前有效）有效期类型的订单管理过期。如果交易所/经纪商不支持此有效期选项，或者你出于任何原因希望由策略来管理，这会很有用。

要使用此选项，请在 `StrategyConfig` 中传入 `manage_gtd_expiry=True`。当提交具有 GTD 有效期的订单时，策略将自动启动一个内部时间提醒。
一旦内部 GTD 时间提醒到达，订单将被取消（如果尚未*关闭*）。

某些交易场所（如 Binance Futures）支持 GTD 有效期，因此在使用 `managed_gtd_expiry` 时，你应该在执行客户端配置中设置 `use_gtd=False` 以避免冲突。

### 多策略

如果你打算运行同一策略的多个实例，使用不同的配置（例如交易不同的金融工具），那么你需要为每个策略定义一个唯一的 `order_id_tag`（如上所示）。

:::note
平台有内置的安全措施：如果两个策略共享重复的策略 ID，在注册时会抛出 `RuntimeError`，提示该策略 ID 已被注册。
:::

原因是系统必须能够识别各种命令和事件属于哪个策略。策略 ID 由策略类名和策略的 `order_id_tag` 以连字符分隔组成。例如上述配置将生成策略 ID `MyStrategy-001`。

:::note
请参阅 `StrategyId` [API 参考](../api_reference/model/identifiers.md)，获取更多详情。
:::
