# 市价转限价 (Market-To-Limit)

`FIX OrdType <40>=K`（Market With Left Over as Limit，剩余转限价的市价单）

*市价转限价 (Market-To-Limit)* 订单以当前最优价格作为市价单提交。
如果订单部分成交，系统会取消剩余部分，并以已成交价格将其重新提交为一张*限价 (Limit)* 单。

## 适用场景

使用*市价转限价 (Market-To-Limit)* 订单可以立即吃掉最优价格上的可用流动性，而不会以更差的价格扫向更深的盘口层级：这在盘口较薄时很有帮助，或者适用于较大的订单——你希望成交在触及价（touch price），但不想承受逐层扫单带来的市场冲击。它的优势是在最优价格上立即成交，任何剩余部分则作为*限价 (Limit)* 单挂在该价位，而不是追价。它的代价在于，如果市场行情走开，未成交的剩余部分可能一直挂着无法成交。

## 示例

在下面的示例中，我们在 Interactive Brokers 的 [IdealPro](https://ibkr.info/node/1708) 外汇 ECN 上创建一张*市价转限价 (Market-To-Limit)* 订单，用 JPY 买入 200,000 USD：

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce},
    identifiers::InstrumentId,
    types::Quantity,
};

let order = self.core.order_factory().market_to_limit(
    InstrumentId::from("USD/JPY.IDEALPRO"),
    OrderSide::Buy,
    Quantity::from(200_000),
    Some(TimeInForce::Gtc), // optional (default GTC)
    None,                   // expire_time
    Some(false),            // reduce_only (default false)
    None,                   // quote_quantity (default false)
    None,                   // display_qty (default full display)
    None,                   // exec_algorithm_id
    None,                   // exec_algorithm_params
    None,                   // tags
    None,                   // client_order_id
);
```

```python tab="Python"
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.enums import TimeInForce
from nautilus_trader.model import InstrumentId
from nautilus_trader.model import Quantity
from nautilus_trader.model.orders import MarketToLimitOrder

order: MarketToLimitOrder = self.order_factory.market_to_limit(
    instrument_id=InstrumentId.from_str("USD/JPY.IDEALPRO"),
    order_side=OrderSide.BUY,
    quantity=Quantity.from_int(200_000),
    time_in_force=TimeInForce.GTC,  # <-- optional (default GTC)
    reduce_only=False,  # <-- optional (default False)
    display_qty=None,  # <-- optional (default None which indicates full display)
    tags=None,  # <-- optional (default None)
)
```

更多细节请参阅 [`MarketToLimitOrder` API 参考文档](/docs/python-api-latest/model/orders.html#nautilus_trader.model.orders.market_to_limit.MarketToLimitOrder)。

:::tip 限价如何确定？
`market_to_limit()` 工厂方法**没有 `price` 参数**——创建时 `price = None`。

限价由撮合引擎在**首次成交时自动确定**：引擎将成交价（fill price）设为该订单的限价，并通过 `OrderUpdated` 事件通知策略。此后，剩余未成交数量将作为限价单处理，只会以该价格或更优价格成交。

**示例：** 提交买入 100 手 MTL 订单 → 以 150.25 成交 60 手 → 引擎自动将 `price` 设为 150.25 → 剩余 40 手变为限价 150.25 的买入限价单（只会以 ≤ 150.25 成交）。
:::

## 相关指南

- [订单 (Orders)](index.md) - 订单概念、执行指令以及订单工厂。
- [执行 (Execution)](../execution.md) - 订单如何到达交易场所以及成交如何处理。
