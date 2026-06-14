# 市价单 (Market)

`FIX OrdType <40>=1`

*Market*（市价）单是交易者发出的指令，要求以当前可获得的最优价格立即成交指定的数量。你还可以指定多种 time in force 选项，并标明该订单是否仅用于减仓。

## 使用场景

当成交本身比精确价格更重要时，应使用 *Market* 单：例如紧急的风险削减、进入快速波动的流动性市场，或者跨越一个很窄的价差——此时等待的代价高于价差本身。其优势在于几乎可以确定地立即成交。代价则是缺乏价格保护：你需要付出价差，并且在稀薄或快速变动的市场中承担滑点风险，因此它远比适用于流动性差的标的更适合流动性好的标的。

## 示例

在下面的示例中，我们在 Interactive Brokers 的 [IdealPro](https://ibkr.info/node/1708) 外汇 ECN 上创建一个 *Market* 单，用 USD 买入（BUY）100,000 AUD：

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce},
    identifiers::InstrumentId,
    types::Quantity,
};
use ustr::Ustr;

let order = self.core.order_factory().market(
    InstrumentId::from("AUD/USD.IDEALPRO"),
    OrderSide::Buy,
    Quantity::from(100_000),
    Some(TimeInForce::Ioc),          // optional (default GTC)
    Some(false),                     // reduce_only (default false)
    None,                            // quote_quantity (default false)
    None,                            // exec_algorithm_id
    None,                            // exec_algorithm_params
    Some(vec![Ustr::from("ENTRY")]), // tags
    None,                            // client_order_id (auto-generated if None)
);
```

```python tab="Python"
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.enums import TimeInForce
from nautilus_trader.model import InstrumentId
from nautilus_trader.model import Quantity
from nautilus_trader.model.orders import MarketOrder

order: MarketOrder = self.order_factory.market(
    instrument_id=InstrumentId.from_str("AUD/USD.IDEALPRO"),
    order_side=OrderSide.BUY,
    quantity=Quantity.from_int(100_000),
    time_in_force=TimeInForce.IOC,  # <-- optional (default GTC)
    reduce_only=False,  # <-- optional (default False)
    tags=["ENTRY"],  # <-- optional (default None)
)
```

更多细节参见 [`MarketOrder` API 参考](/docs/python-api-latest/model/orders.html#nautilus_trader.model.orders.market.MarketOrder)。

## 相关指南

- [订单](index.md) - 订单概念、执行指令以及 order factory。
- [执行](../execution.md) - 订单如何到达交易场所以及成交如何被处理。
