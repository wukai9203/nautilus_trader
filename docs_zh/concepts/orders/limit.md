# 限价单 (Limit)

`FIX OrdType <40>=2`

*限价单 (Limit)* 以某个特定价格挂到限价订单簿上，只会以该价格（或更优价格）成交。

## 适用场景

当你希望控制成交价格，并可选地提供流动性时，使用*限价单 (Limit)*：做市、在选定的价位分批建仓或减仓，或通过 `post_only` 捕获挂单方手续费档位。其优势在于成交价永远不会比你设定的价格更差；代价是没有成交保证：如果市场始终未触及或未守住你的价格，订单可能一直挂着不成交，或仅部分成交。

## 示例

在下面的示例中，我们在 Binance Futures Crypto 交易所创建一个*限价单 (Limit)*，以做市商身份，按 5000 USDT 的限价卖出 20 张 ETHUSDT-PERP 永续期货合约。

```rust tab="Rust"
use nautilus_model::{
    enums::{OrderSide, TimeInForce},
    identifiers::InstrumentId,
    types::{Price, Quantity},
};

let order = self.core.order_factory().limit(
    InstrumentId::from("ETHUSDT-PERP.BINANCE"),
    OrderSide::Sell,
    Quantity::from(20),
    Price::from("5000.00"),
    Some(TimeInForce::Gtc), // optional (default GTC)
    None,                   // expire_time
    Some(true),             // post_only (default false)
    Some(false),            // reduce_only (default false)
    None,                   // quote_quantity (default false)
    None,                   // display_qty (default full display)
    None,                   // emulation_trigger
    None,                   // trigger_instrument_id
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
from nautilus_trader.model import Price
from nautilus_trader.model import Quantity
from nautilus_trader.model.orders import LimitOrder

order: LimitOrder = self.order_factory.limit(
    instrument_id=InstrumentId.from_str("ETHUSDT-PERP.BINANCE"),
    order_side=OrderSide.SELL,
    quantity=Quantity.from_int(20),
    price=Price.from_str("5_000.00"),
    time_in_force=TimeInForce.GTC,  # <-- optional (default GTC)
    expire_time=None,  # <-- optional (default None)
    post_only=True,  # <-- optional (default False)
    reduce_only=False,  # <-- optional (default False)
    display_qty=None,  # <-- optional (default None which indicates full display)
    tags=None,  # <-- optional (default None)
)
```

更多细节请参阅 [`LimitOrder` API 参考](/docs/python-api-latest/model/orders.html#nautilus_trader.model.orders.limit.LimitOrder)。

## 相关指南

- [订单 (Orders)](index.md) - 订单概念、执行指令以及订单工厂 (order factory)。
- [模拟订单 (Emulated orders)](emulated.md) - 模拟*限价单 (Limit)*，在触发时以*市价单 (Market)* 释放。
- [执行 (Execution)](../execution.md) - 订单如何到达交易场所以及成交如何处理。
