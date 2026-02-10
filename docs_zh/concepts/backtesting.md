# 回测 (Backtesting)

使用 NautilusTrader 进行回测 (backtesting) 是一种系统化的模拟过程，它使用特定的系统实现来复制交易活动。该系统由多个组件构成，
包括内置引擎 (engine)、`Cache`、[消息总线](message_bus.md)、`Portfolio`、[Actor](actors.md)、[策略](strategies.md)、[执行算法](execution.md)
以及其他用户自定义模块。整个交易模拟基于由 `BacktestEngine` 处理的历史数据流。当数据流处理完毕后，引擎结束运行，
生成详细的结果和绩效指标供深入分析。

需要注意的是，NautilusTrader 提供了两个不同级别的 API 来设置和进行回测：

- **高级 API**：使用 `BacktestNode` 和配置 (configuration) 对象（内部使用 `BacktestEngine`）。
- **低级 API**：直接使用 `BacktestEngine`，需要更多"手动"设置。

## 选择 API 级别

在以下情况下考虑使用**低级** API：

- 整个数据流可以在可用的机器资源（如内存）内处理。
- 不需要以 Nautilus 专用的 Parquet 格式存储数据。
- 有特定需求或偏好保留原始格式的数据（如 CSV、二进制格式等）。
- 需要对 `BacktestEngine` 进行细粒度控制，例如在相同数据集上重新运行回测，同时替换组件（如 Actor 或策略）或调整参数配置。

在以下情况下考虑使用**高级** API：

- 数据流超出可用内存，需要分批流式加载数据。
- 希望利用 `ParquetDataCatalog` 的性能和便捷性，以 Nautilus 专用的 Parquet 格式存储数据。
- 重视通过配置对象来定义和管理多个回测运行的灵活性和功能，可以同时跨多个引擎执行。

## 低级 API

低级 API 以 `BacktestEngine` 为核心，通过 Python 脚本手动初始化和添加输入。
已实例化的 `BacktestEngine` 可以接受以下内容：

- `Data` 对象列表，会自动按 `ts_init` 排序为单调递增顺序。
- 多个交易场所 (venue)，需手动初始化。
- 多个 Actor，需手动初始化和添加。
- 多个执行算法，需手动初始化和添加。

这种方法提供了对回测过程的详细控制，允许手动配置每个组件。

### 高效加载大数据集

当处理跨多个金融工具 (instrument) 的大量数据时，数据加载方式会显著影响性能。

#### 性能考虑

默认情况下，`BacktestEngine.add_data()` 在每次调用时（当 `sort=True`，即默认值时）都会对整个数据流（已有数据 + 新添加的数据）进行排序。这意味着：

- 第一次调用 100 万根 K线 (bar)：排序 100 万根 K线。
- 第二次调用 100 万根 K线：排序 200 万根 K线。
- 第三次调用 100 万根 K线：排序 300 万根 K线。
- 以此类推...

这种对越来越大的数据集进行重复排序的方式，在加载多个金融工具的数据时可能成为瓶颈。

#### 优化策略

**策略 1：延迟到最后排序（推荐用于多个金融工具）**

```python
from nautilus_trader.backtest.engine import BacktestEngine

engine = BacktestEngine()

# 设置交易场所和金融工具
engine.add_venue(...)
engine.add_instrument(instrument1)
engine.add_instrument(instrument2)
engine.add_instrument(instrument3)

# 加载所有数据，每次调用时不排序
engine.add_data(instrument1_bars, sort=False)
engine.add_data(instrument2_bars, sort=False)
engine.add_data(instrument3_bars, sort=False)

# 最后排序一次 - 效率更高！
engine.sort_data()

# 现在运行回测
engine.add_strategy(strategy)
engine.run()
```

**策略 2：收集后一次性添加**

```python
# 先收集所有数据
all_bars = []
all_bars.extend(instrument1_bars)
all_bars.extend(instrument2_bars)
all_bars.extend(instrument3_bars)

# 一次性添加并排序
engine.add_data(all_bars, sort=True)
```

**策略 3：对超大数据集使用流式 API**

对于无法放入内存的数据集，使用流式 API：

```python
def data_generator():
    # 逐块生成数据（每块是一个 Data 对象列表）
    yield load_chunk_1()
    yield load_chunk_2()
    yield load_chunk_3()

engine.add_data_iterator(
    data_name="my_data_stream",
    generator=data_generator(),
)
```

:::note
流式 API 在回测运行过程中按需处理数据块，避免了预先将所有数据加载到内存中的需要。
:::

:::tip 性能影响
对于包含 10 个金融工具、每个有 100 万根 K线的回测：

- 每次调用都排序：约 10 次递增大小的排序（100 万、200 万、300 万、... 1000 万根 K线）。
- 最后排序一次：对 1000 万根 K线排序 1 次。

延迟排序方法对于大数据集可以**快几个数量级**。
:::

### 数据加载契约

`BacktestEngine` 强制执行重要的不变量以确保数据完整性：

**要求：**

- 所有数据必须在调用 `run()` 之前排序。
- 使用 `sort=False` 时，**必须**在运行前调用 `sort_data()`。
- 引擎会验证这一点，如果检测到未排序的数据，会抛出 `RuntimeError`。
- 多次调用 `sort_data()` 是安全的（幂等操作）。

**安全保证：**

- 数据列表在内部始终会被复制，以防止外部修改影响引擎状态。
- 将数据传递给 `add_data()` 后，可以安全地清除或修改数据列表。
- 使用 `sort=True` 添加的数据可以立即用于回测。

这种设计在保证数据完整性的同时，为大数据集提供了性能优化能力。

## 高级 API

高级 API 以 `BacktestNode` 为核心，它负责编排管理多个 `BacktestEngine` 实例，
每个实例由一个 `BacktestRunConfig` 定义。多个配置可以打包成一个列表，由节点 (node) 在一次运行中处理。

每个 `BacktestRunConfig` 对象由以下部分组成：

- `BacktestDataConfig` 对象列表。
- `BacktestVenueConfig` 对象列表。
- `ImportableActorConfig` 对象列表。
- `ImportableStrategyConfig` 对象列表。
- `ImportableExecAlgorithmConfig` 对象列表。
- 可选的 `ImportableControllerConfig` 对象。
- 可选的 `BacktestEngineConfig` 对象，如未指定则使用默认配置。

## 重复运行

进行多次回测运行时，了解组件如何重置非常重要，以避免意外行为。

### BacktestEngine.reset()

`.reset()` 方法将所有有状态字段恢复到**初始值**，但数据和金融工具会保留。

**被重置的内容：**

- 所有交易状态（订单 (order)、持仓 (position)、账户余额）。
- 策略实例会被移除（必须在下次运行前重新添加策略）。
- 引擎计数器和时间戳。

**保留的内容：**

- 通过 `.add_data()` 添加的数据（使用 `.clear_data()` 来移除）。
- 金融工具（必须与保留的数据匹配）。
- 交易场所配置。

**金融工具处理：**

对于 `BacktestEngine`，金融工具默认在重置后保留（因为数据保留了，金融工具必须与数据匹配）。
这通过 `BacktestEngineConfig` 默认设置中的 `CacheConfig.drop_instruments_on_reset=False` 来配置。

### 多次回测运行的方法

运行多次回测有两种主要方法：

#### 1. 使用 BacktestNode（推荐用于生产环境）

高级 API 专为使用不同配置进行多次回测运行而设计：

```python
from nautilus_trader.backtest.node import BacktestNode
from nautilus_trader.config import BacktestRunConfig

# 定义多个运行配置
configs = [
    BacktestRunConfig(...),  # 运行 1
    BacktestRunConfig(...),  # 运行 2
    BacktestRunConfig(...),  # 运行 3
]

# 执行所有运行
node = BacktestNode(configs=configs)
results = node.run()
```

每次运行都会获得一个全新的引擎和干净的状态 - 无需调用 reset()。

#### 2. 使用 BacktestEngine.reset()

使用低级 API 进行细粒度控制：

```python
from nautilus_trader.backtest.engine import BacktestEngine

engine = BacktestEngine()

# 一次性设置
engine.add_venue(...)
engine.add_instrument(ETHUSDT)
engine.add_data(data)

# 运行 1
engine.add_strategy(strategy1)
engine.run()

# 重置并运行 2 - 金融工具和数据保留
engine.reset()
engine.add_strategy(strategy2)
engine.run()

# 重置并运行 3
engine.reset()
engine.add_strategy(strategy3)
engine.run()
```

:::note
对于 `BacktestEngine`，金融工具和数据默认在重置后保留，使参数优化变得简单直接。
:::

:::tip 最佳实践

- **生产回测：** 使用 `BacktestNode` 配合配置对象。
- **参数优化：** 使用 `BacktestEngine.reset()` 在相同数据上运行多个策略。
- **快速实验：** 两种方法都可以 - 根据具体使用场景选择。
:::

## 数据

回测提供的数据驱动着执行流程。由于可以使用多种数据类型，
确保交易场所配置与回测数据匹配至关重要。
数据与配置之间的不匹配可能导致执行过程中的意外行为。

NautilusTrader 主要针对订单簿 (order book) 数据进行设计和优化，订单簿提供了市场中每个价格级别或订单的完整表示，
反映了交易场所的实时行为。这确保了最高级别的执行粒度和真实性。但是，如果精细的订单簿数据不可用或不必要，
平台也能按以下详细程度递减的顺序处理市场数据：

```mermaid
flowchart LR
    L3["L3 订单簿<br/>(逐笔委托)"]
    L2["L2 订单簿<br/>(逐档行情)"]
    L1["L1 报价<br/>(盘口最优)"]
    T["成交"]
    B["K线"]

    L3 --> L2 --> L1 --> T --> B

    style L3 fill:#2d5a3d,color:#fff
    style L2 fill:#3d6a4d,color:#fff
    style L1 fill:#4d7a5d,color:#fff
    style T fill:#5d8a6d,color:#fff
    style B fill:#6d9a7d,color:#fff
```

1. **订单簿数据/增量（L3 逐笔委托）**：
   - 全面的市场深度，可见所有单个订单。

2. **订单簿数据/增量（L2 逐档行情）**：
   - 所有价格级别的市场深度可见性。

3. **报价 Tick（L1 逐档行情）**：
   - 仅盘口最优价 - 最佳买价和卖价及其数量。

4. **成交 Tick**：
   - 实际执行的成交。

5. **K线**：
   - 固定时间间隔内的聚合交易活动（如 1 分钟、1 小时、1 天）。

### 选择数据：成本与精度

对于许多交易策略而言，K线数据（如 1 分钟）对于回测和策略开发可能已经足够。这一点尤其重要，
因为与 Tick 或订单簿数据相比，K线数据通常更容易获取且成本更低。

鉴于这一实际情况，Nautilus 被设计为支持基于 K线的回测，并提供高级功能来最大化模拟精度，即使使用较低粒度的数据也是如此。

:::tip
对于某些交易策略，从 K线数据开始开发以验证核心交易思路可能是可行的。
如果策略看起来有前景，但对精确的执行时机更敏感（例如，需要在 OHLC 级别之间的特定价格成交，
或使用较紧的止盈/止损水平），那么可以再投资获取更高粒度的数据以进行更准确的验证。
:::

## 交易场所

初始化回测交易场所时，必须从以下选项中指定其内部订单 `book_type` 以用于执行处理：

- `L1_MBP`：L1 逐档行情（默认）。仅维护订单簿的最优一档。
- `L2_MBP`：L2 逐档行情。维护订单簿深度，每个价格级别聚合一个订单。
- `L3_MBO`：L3 逐笔委托。维护订单簿深度，数据提供的所有单个订单都被跟踪。

:::note
数据的粒度必须与指定的订单 `book_type` 匹配。Nautilus 无法从较低级别的数据（如报价、成交或 K线）生成更高粒度的数据（L2 或 L3）。
:::

:::warning
如果将 `L2_MBP` 或 `L3_MBO` 指定为交易场所的 `book_type`，所有非订单簿数据（如报价、成交和 K线）将在执行处理中被忽略。
这可能导致订单看起来永远不会被成交。我们正在积极改进验证逻辑，以防止配置和数据不匹配。
:::

:::warning
当提供 L2 或更高级别的订单簿数据时，请确保更新 `book_type` 以反映数据的粒度。
否则将导致数据聚合：L2 数据将被压缩为每个级别一个订单，L1 数据将仅反映盘口最优价。
:::

## 执行

### 数据和消息排序

在主回测循环中，新的市场数据首先被处理用于现有订单的执行，然后才由数据引擎处理并发送给策略。

### 基于 K线的执行

K线数据为每个时间周期提供了四个关键价格的市场活动摘要（假设 K线按成交聚合）：

- **开盘价 (Open)**：开盘价（第一笔成交）
- **最高价 (High)**：最高成交价格
- **最低价 (Low)**：最低成交价格
- **收盘价 (Close)**：收盘价（最后一笔成交）

虽然这为我们提供了价格变动的概览，但与更精细的数据相比，我们丢失了一些重要信息：

- 我们不知道市场以什么顺序触及最高价和最低价。
- 我们无法确切看到时间周期内价格何时发生变化。
- 我们不知道发生的实际成交序列。

这就是为什么 Nautilus 通过一个系统来处理 K线数据，该系统尽管存在这些局限性，仍试图维持最真实且保守的市场行为。
其核心是，平台始终维护一个订单簿模拟 - 即使你提供的数据粒度较低，如报价、成交或 K线（尽管模拟只会有一个盘口最优档位）。

:::warning
当使用 K线进行执行模拟时（通过交易场所配置中的 `bar_execution=True` 默认启用），
Nautilus 严格要求每根 K线的初始化时间戳（`ts_init`）代表其**收盘时间**。
这确保了准确的时间顺序处理，防止前视偏差 (look-ahead bias)，并将市场更新（Open -> High -> Low -> Close）与 K线完成的时刻对齐。

事件时间戳（`ts_event`）可以代表 K线的开盘或收盘时间：

- 如果 `ts_event` 在**收盘**时间，处理 K线时确保 `ts_init_delta=0`（默认值）。
- 如果 `ts_event` 在**开盘**时间，将 `ts_init_delta` 设置为 K线的持续时间，以将 `ts_init` 偏移到收盘时间。

:::

#### K线时间戳约定

如果你的数据源提供的 K线以**开盘时间**为时间戳（某些数据提供商常见），你需要确保 `ts_init` 被设置为收盘时间以进行正确的执行模拟。有两种方法：

**方法 1：调整数据时间戳（推荐）**

- 使用适配器特定的配置，如 `bars_timestamp_on_close=True`（例如 Bybit 或 Databento 适配器），在数据摄入时自动处理。
- 对于自定义数据，在加载前手动将时间戳偏移 K线持续时间（例如，为 `1-MINUTE` K线加 1 分钟）。
- 这种方法最清晰，因为数据本身反映了收盘时间。

**方法 2：使用 `ts_init_delta` 参数**

- 调用 `BarDataWrangler.process()` 时，将 `ts_init_delta` 设置为 K线持续时间的纳秒值（例如，1 分钟 K线为 `60_000_000_000`）。
- 整理器计算 `ts_init = ts_event + ts_init_delta`，将执行时间偏移到收盘时间。
- 当无法或不愿修改源数据时间戳时使用此方法。

始终使用小样本验证数据的时间戳约定，以避免模拟不准确。错误的时间戳处理可能导致前视偏差和不真实的回测结果。

#### 处理 K线数据

即使你提供的是 K线数据，Nautilus 也会像真实交易场所那样为每个金融工具维护一个内部订单簿。

1. **时间处理**：
   - Nautilus 对 K线数据*的执行*有特定的时间处理方式，这对于准确的模拟至关重要。
   - 初始化时间戳（`ts_init`）用于执行时间，必须代表 K线的收盘时间。这种方法最合理，因为它代表了 K线完全形成、聚合完成的时刻。
   - 事件时间戳（`ts_event`）代表数据事件发生的时间，根据数据源可能与 `ts_init` 不同：
     - 如果 K线以**收盘**时间为时间戳（推荐默认），在 `BarDataWrangler` 中使用 `ts_init_delta=0`，使 `ts_init = ts_event`。
     - 如果 K线以**开盘**时间为时间戳，将 `ts_init_delta` 设置为 K线持续时间的纳秒值（例如，1 分钟 K线为 60_000_000_000），以将 `ts_init` 偏移到收盘时间。
   - 平台确保所有事件按 `ts_init` 的正确顺序发生，防止回测中出现任何前视偏差的可能性。

:::note K线执行的例外情况
在以下情况下，K线**不会**被处理用于执行（也不会更新订单簿）：

- **内部聚合的 K线**：具有 `AggregationSource.INTERNAL` 的 K线会被跳过，以避免处理从已处理的 Tick 数据派生的 K线。
- **非 L1 簿类型**：当交易场所的 `book_type` 配置为 `L2_MBP` 或 `L3_MBO` 时，K线数据在执行处理中被忽略，因为 K线仅来源于盘口最优价。

在这些情况下，策略仍然会收到 K线用于分析和决策，但它们不会触发订单撮合或更新模拟订单簿。
:::

2. **价格处理**：
   - 平台将每根 K线的 OHLC 价格转换为一系列市场更新。
   - 这些更新始终遵循相同的顺序：Open -> High -> Low -> Close。
   - 如果你提供多个时间框架（如 1 分钟和 5 分钟 K线），平台会使用更精细的数据以获得最高精度。

3. **执行**：
   - 当你下单时，订单会像在真实交易场所一样与模拟订单簿交互。
   - 对于市价 (MARKET) 订单，执行发生在当前模拟市场价格加上任何配置的延迟 (latency)。
   - 对于在市场中等待的限价 (LIMIT) 订单，如果 K线的任何价格达到或穿过你的限价，它们就会被执行（见下文）。
   - 撮合引擎 (matching engine) 在 OHLC 价格移动时持续处理订单，而不是等待完整的 K线。

#### OHLC 价格模拟

在回测执行期间，每根 K线被转换为四个价格点的序列：

1. 开盘价
2. 最高价 *（高/低价的顺序可配置。见下方 `bar_adaptive_high_low_ordering`。）*
3. 最低价
4. 收盘价

该 K线的交易量**平均分配**到这四个价格点（每个 25%），余数加到收盘价的成交上以保持总量。在边际情况下，
如果 K线的成交量除以 4 小于金融工具的最小 `size_increment`，我们会在每个价格点使用最小 `size_increment` 以确保
有效的市场活动（例如，CME 集团交易所为 1 张合约）。

如何排列这些价格点可以通过配置交易场所时的 `bar_adaptive_high_low_ordering` 参数来控制。

Nautilus 支持两种 K线处理模式：

1. **固定排序** (`bar_adaptive_high_low_ordering=False`，默认)
   - 以固定顺序处理每根 K线：`Open -> High -> Low -> Close`。
   - 简单且确定性的方法。

2. **自适应排序** (`bar_adaptive_high_low_ordering=True`)
   - 使用 K线结构估计可能的价格路径：
     - 如果开盘价更接近最高价：按 `Open -> High -> Low -> Close` 处理。
     - 如果开盘价更接近最低价：按 `Open -> Low -> High -> Close` 处理。
   - [研究](https://gist.github.com/stefansimik/d387e1d9ff784a8973feca0cde51e363)表明，这种方法在预测正确的高/低价顺序时准确率约为 75-85%（相比固定排序的统计约 50% 准确率）。
   - 当止盈和止损水平都出现在同一根 K线内时，这一点尤为重要 - 因为顺序决定了哪个订单先被成交。

以下是如何为交易场所配置自适应 K线排序，包括账户设置：

```python
from nautilus_trader.backtest.engine import BacktestEngine
from nautilus_trader.model.enums import OmsType, AccountType
from nautilus_trader.model import Money, Currency

# 初始化回测引擎
engine = BacktestEngine()

# 添加带有自适应K线排序的交易场所及必要的账户设置
engine.add_venue(
    venue=venue,  # 你的 Venue 标识符，例如 Venue("BINANCE")
    oms_type=OmsType.NETTING,
    account_type=AccountType.CASH,
    starting_balances=[Money(10_000, Currency.from_str("USDT"))],
    bar_adaptive_high_low_ordering=True,  # 启用高/低价K线价格的自适应排序
)
```

### 基于成交的执行

当你拥有成交 Tick 数据时，在交易场所配置中启用 `trade_execution=True` 可以基于成交活动触发订单成交。
成交 Tick 表明在成交价格处有流动性被消耗，允许等待中的限价订单进行撮合。

撮合引擎使用"瞬态覆盖"机制：在撮合过程中，它临时将最优买价（对于 BUYER 成交）或最优卖价（对于 SELLER 成交）
更新为成交价格。这允许被动方的等待订单跨越价差 (spread) 并成交。撮合后，原始订单簿状态会被恢复，确保价差不会被瞬态成交价格永久破坏。

**成交行为：**

- **SELLER 在价格 P 成交**：引擎临时将最优卖价设置为 P。在价格 P 或更高处等待的买入限价单将被成交（因为它们愿意以 P 或更高价格买入）。
- **BUYER 在价格 P 成交**：引擎临时将最优买价设置为 P。在价格 P 或更低处等待的卖出限价单将被成交（因为它们愿意以 P 或更低价格卖出）。

**成交数量上限：**

成交数量被限制以确保真实的执行模拟：

- **单订单上限**：每个订单的成交数量限制为订单剩余数量与成交 Tick 大小的最小值。例如，如果你有一个 100,000 单位的买入限价单，且一笔 200 单位的 SELLER 成交发生在你的限价处，订单将被部分成交 200 单位（而非全部 100,000）。

- **多订单上限**：当多个订单匹配同一笔成交 Tick 时，所有订单的总成交数量不会超过成交 Tick 的大小。例如，如果两个买入限价单（40 和 60 单位）在等待，且一笔 50 单位的 SELLER 成交发生，第一个订单成交 40 单位，第二个订单成交 10 单位（剩余成交量），总计 50 单位。

这种行为确保回测不会夸大执行量，使其超过历史成交数据表明的实际可用市场量。

**示例：**

```python
engine.add_venue(
    venue=venue,
    oms_type=OmsType.NETTING,
    account_type=AccountType.CASH,
    starting_balances=[Money(10_000, USDT)],
    trade_execution=True,
)
```

:::tip
将成交数据与订单簿或报价数据结合使用以获得最佳效果：订单簿/报价数据建立基准价差，
而成交 Tick 触发可能位于价差内或领先于报价更新的订单执行。
:::

### 精度要求和不变量

撮合引擎强制执行严格的精度不变量以确保整个成交管道中的数据完整性。
所有价格和数量必须匹配金融工具配置的精度（`price_precision` 和 `size_precision`）。
不匹配会立即抛出 `RuntimeError`，防止成交数量的静默损坏。

| 数据/操作 | 字段                          | 要求精度           | 验证位置          |
|----------------|--------------------------------|------------------------------|------------------------------|
| `QuoteTick`    | `bid_price`, `ask_price`       | `instrument.price_precision` | `process_quote_tick`         |
| `QuoteTick`    | `bid_size`, `ask_size`         | `instrument.size_precision`  | `process_quote_tick`         |
| `TradeTick`    | `price`                        | `instrument.price_precision` | `process_trade_tick`         |
| `TradeTick`    | `size`                         | `instrument.size_precision`  | `process_trade_tick`         |
| `Bar`          | `open`, `high`, `low`, `close` | `instrument.price_precision` | `process_bar`                |
| `Bar`          | `volume`（基础单位）          | `instrument.size_precision`  | `process_bar`                |
| `Order`        | `quantity`                     | `instrument.size_precision`  | `process_order`              |
| `Order`        | `price`                        | `instrument.price_precision` | `process_order`              |
| `Order`        | `trigger_price`                | `instrument.price_precision` | `process_order`              |
| `Order`        | `activation_price`*            | `instrument.price_precision` | `process_order`              |
| 订单更新   | `quantity`                     | `instrument.size_precision`  | `update_order`               |
| 订单更新   | `price`, `trigger_price`       | `instrument.price_precision` | `update_order`               |
| 成交           | `fill_qty`                     | `instrument.size_precision`  | `apply_fills`, `fill_order`  |
| 成交           | `fill_px`                      | `instrument.price_precision` | `apply_fills`                |

*`activation_price` 在订单提交后不可变。

:::warning
`Bar.volume` 必须以**基础货币单位**计量。某些数据提供商报告的是报价货币的成交量；
加载前需转换为基础单位（除以价格或使用提供商特定的字段）。
:::

:::tip
如果遇到精度不匹配错误，请将数据与金融工具对齐：

```python
# 将价格/数量对齐到金融工具精度
price = instrument.make_price(raw_price)
qty = instrument.make_qty(raw_qty)
```

还需验证：

1. 金融工具定义与数据源的精度匹配。
2. 数据在加载过程中未被无意四舍五入或截断。
3. 自定义数据加载器保留了原始精度元数据。

:::

### 滑点 (Slippage) 和价差处理

使用不同类型的数据进行回测时，Nautilus 对滑点和价差模拟有特定的处理方式：

对于 L2（逐档行情）或 L3（逐笔委托）数据，滑点通过以下方式进行高精度模拟：

- 根据实际订单簿级别成交订单。
- 在每个价格级别按顺序匹配可用数量。
- 维护真实的订单簿深度影响（按订单成交）。

对于 L1 数据类型（如 L1 订单簿、成交、报价、K线），滑点通过以下方式处理：

**初始成交滑点** (`prob_slippage`)：

- 由 `FillModel` 的 `prob_slippage` 参数控制。
- 决定初始成交是否会发生在距当前市场价格一个 Tick 的位置。
- 示例：当 `prob_slippage=0.5` 时，市价买单有 50% 的概率在最优卖价上方一个 Tick 成交。

:::note
当使用 K线数据进行回测时，请注意价格信息粒度的降低会影响滑点机制。
为了获得最真实的回测结果，在可用时考虑使用更高粒度的数据源，如 L2 或 L3 订单簿数据。
:::

### 成交模型 (Fill Model)

`FillModel` 在回测期间以简单的概率方式帮助模拟订单队列位置和执行。
它解决了一个根本性挑战：*即使拥有完美的历史市场数据，我们也无法完全模拟订单在实时环境中可能与其他
市场参与者的交互方式*。

`FillModel` 模拟了两个关键的交易方面，无论数据质量如何，这些都存在于真实市场中：

1. **限价单的队列位置**：
   - 当多个交易者在同一价格级别下单时，订单在队列中的位置影响其是否以及何时被成交。

2. **市场影响和竞争**：
   - 当使用市价单消耗流动性时，你与其他交易者竞争可用的流动性，这可能影响你的成交价格。

#### 配置和参数

```python
from nautilus_trader.backtest.config import BacktestVenueConfig
from nautilus_trader.backtest.config import ImportableFillModelConfig

# 为交易场所配置自定义成交模型
venue_config = BacktestVenueConfig(
    name="SIM",
    oms_type="NETTING",
    account_type="CASH",
    starting_balances=["100_000 USD"],
    fill_model=ImportableFillModelConfig(
        fill_model_path="nautilus_trader.backtest.models:FillModel",
        config_path="nautilus_trader.backtest.config:FillModelConfig",
        config={
            "prob_fill_on_limit": 0.2,    # 限价单在价格匹配时成交的概率
            "prob_fill_on_stop": 0.95,    # [已弃用] 请改用 `prob_slippage`
            "prob_slippage": 0.5,         # 1个Tick滑点的概率（仅适用于L1数据）
            "random_seed": 42,            # 可选：设置以获得可复现的结果
        },
    ),
)
```

**prob_fill_on_limit**（默认值：`1.0`）

- 用途：
  - 模拟限价单在其价格级别被市场触及时被成交的概率。
- 详情：
  - 模拟你在给定价格级别的订单队列中的位置。
  - 适用于所有数据类型（如 L1/L2/L3 订单簿、报价、成交、K线）。
  - 每次市场价格触及你的订单价格时（但没有穿过它），都会进行新的随机概率检查。
  - 概率检查通过时，成交订单的全部剩余数量。

**示例**：

- 当 `prob_fill_on_limit=0.0` 时：
  - 当最优卖价达到限价时，买入限价单永远不会成交。
  - 当最优买价达到限价时，卖出限价单永远不会成交。
  - 这模拟了处于队列最末尾且永远不会到达前面的情况。
- 当 `prob_fill_on_limit=0.5` 时：
  - 当最优卖价达到限价时，买入限价单有 50% 的成交概率。
  - 当最优买价达到限价时，卖出限价单有 50% 的成交概率。
  - 这模拟了处于队列中间的情况。
- 当 `prob_fill_on_limit=1.0` 时（默认）：
  - 当最优卖价达到限价时，买入限价单始终成交。
  - 当最优买价达到限价时，卖出限价单始终成交。
  - 这模拟了处于队列最前面、保证成交的情况。

**prob_slippage**（默认值：`0.0`）

- 用途：
  - 模拟执行市价单时经历价格滑点的概率。
- 详情：
  - 仅适用于 L1 数据类型（如报价、成交、K线）。
  - 触发时，将成交价格向不利于你的订单方向移动一个 Tick。
  - 影响所有市价类型订单（`MARKET`、`MARKET_TO_LIMIT`、`MARKET_IF_TOUCHED`、`STOP_MARKET`）。
  - 不用于 L2/L3 数据，因为订单簿深度可以确定滑点。

**示例**：

- 当 `prob_slippage=0.0` 时（默认）：
  - 不施加人工滑点，代表你始终以当前市场价格成交的理想化场景。
- 当 `prob_slippage=0.5` 时：
  - 市价买单有 50% 的概率在最优卖价上方一个 Tick 成交，50% 的概率在最优卖价成交。
  - 市价卖单有 50% 的概率在最优买价下方一个 Tick 成交，50% 的概率在最优买价成交。
- 当 `prob_slippage=1.0` 时：
  - 市价买单始终在最优卖价上方一个 Tick 成交。
  - 市价卖单始终在最优买价下方一个 Tick 成交。
  - 这模拟了持续的对你的订单不利的价格变动。

**prob_fill_on_stop**（默认值：`1.0`）

- 止损单是止损市价单的简称，当市场价格触及止损价格时，它转换为市价单。
- 止损单的成交机制遵循市价单机制，由 `prob_slippage` 参数控制。

:::warning
`prob_fill_on_stop` 参数已弃用，将在未来版本中移除（请改用 `prob_slippage`）。
:::

#### 模拟如何因数据类型而异

`FillModel` 的行为根据使用的订单簿类型而调整：

**L2/L3 订单簿数据**

拥有完整的订单簿深度时，`FillModel` 纯粹专注于通过 `prob_fill_on_limit` 模拟限价单的队列位置。
订单簿本身根据每个价格级别的可用流动性自然处理滑点。

- `prob_fill_on_limit` 激活 - 模拟队列位置。
- `prob_slippage` 不使用 - 真实订单簿深度决定价格影响。

**L1 订单簿数据**

仅有最优买/卖价可用时，`FillModel` 提供额外的模拟：

- `prob_fill_on_limit` 激活 - 模拟队列位置。
- `prob_slippage` 激活 - 由于缺乏真实深度信息，模拟基本价格影响。

**K线/报价/成交数据**

使用较低粒度数据时，与 L1 相同的行为适用：

- `prob_fill_on_limit` 激活 - 模拟队列位置。
- `prob_slippage` 激活 - 模拟基本价格影响。

#### 重要注意事项

`FillModel` 有一些需要注意的局限性：

- **部分成交支持** L2/L3 订单簿数据 - 当订单簿中不再有可用数量时，不会再生成成交，订单将保持在部分成交状态。这准确模拟了真实市场条件下在期望价格级别没有足够流动性的情况。
- 对于 L1 数据，滑点限制为固定的 1 个 Tick，此时系统以订单的全部数量成交。

:::note
随着 `FillModel` 的持续演进，未来版本可能会引入更复杂的订单执行动态模拟，包括：

- 部分成交模拟。
- 基于订单大小的可变滑点。
- 更复杂的队列位置建模。

:::

## 账户类型

将交易场所附加到引擎时 - 无论是实盘交易还是回测 - 你必须通过传递 `account_type` 参数来选择三种账户模式之一：

| 账户类型           | 典型使用场景                                         | 引擎锁定的内容                                                                                              |
| ---------------------- | -------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------|
| Cash（现金）                   | 现货交易（如 BTC/USDT、股票）                     | 每个待处理订单将要开仓的名义价值。                                                      |
| Margin（保证金）                 | 衍生品或任何允许杠杆的产品          | 每个订单的初始保证金加上持仓的维持保证金。                                          |
| Betting（博彩）                | 体育博彩、庄家业务                              | 交易场所要求的注金；无杠杆。                                                                          |

为回测交易场所添加 `CASH` 账户的示例：

```python
from nautilus_trader.adapters.binance import BINANCE_VENUE
from nautilus_trader.backtest.engine import BacktestEngine
from nautilus_trader.model.currencies import USDT
from nautilus_trader.model.enums import OmsType, AccountType
from nautilus_trader.model import Money, Currency

# 初始化回测引擎
engine = BacktestEngine()

# 为交易场所添加 CASH 账户
engine.add_venue(
    venue=BINANCE_VENUE,  # 创建或引用一个 Venue 标识符
    oms_type=OmsType.NETTING,
    account_type=AccountType.CASH,
    starting_balances=[Money(10_000, USDT)],
)
```

### 现金账户

现金账户以全额结算交易；没有杠杆，因此没有保证金的概念。

### 保证金账户

*保证金账户*用于交易需要保证金的金融工具，如期货或杠杆产品。
它跟踪账户余额，计算所需保证金，并管理杠杆以确保持仓和订单有足够的抵押品。

**关键概念**：

- **杠杆 (Leverage)**：相对于账户权益放大交易敞口。更高的杠杆增加潜在回报和风险。
- **初始保证金 (Initial Margin)**：提交订单开仓所需的抵押品。
- **维持保证金 (Maintenance Margin)**：维持持仓所需的最低抵押品。
- **锁定余额 (Locked Balance)**：作为抵押品保留的资金，不可用于新订单或取款。

:::note
仅减仓订单 (reduce-only) **不会**增加现金账户的 `balance_locked`，
也不会增加保证金账户的初始保证金 - 因为它们只能减少现有敞口。
:::

### 博彩账户

博彩账户专门用于你投注一定金额以赢取或损失固定赔付的交易场所（某些预测市场、体育博彩等）。
引擎仅锁定交易场所要求的注金；杠杆和保证金不适用。

## 保证金模型

NautilusTrader 提供灵活的保证金计算模型，以适应不同的交易场所类型和交易场景。

### 概述

不同的交易场所和经纪商有不同的保证金要求计算方法：

- **传统经纪商**（盈透证券 Interactive Brokers、TD Ameritrade）：固定保证金比例，与杠杆无关。
- **加密货币交易所**（币安 Binance 等）：杠杆可能降低保证金要求。
- **期货交易所**（CME、ICE）：每张合约固定保证金金额。

### 可用模型

#### StandardMarginModel

使用固定比例，不除以杠杆，匹配传统经纪商行为。

**公式：**

```python
# 固定比例 - 忽略杠杆
margin = notional * instrument.margin_init
```

- 初始保证金 = `名义价值 * instrument.margin_init`
- 维持保证金 = `名义价值 * instrument.margin_maint`

**使用场景：**

- 传统经纪商（盈透证券、TD Ameritrade）。
- 期货交易所（CME、ICE）。
- 固定保证金要求的外汇经纪商。

#### LeveragedMarginModel

将保证金要求除以杠杆。

**公式：**

```python
# 杠杆降低保证金要求
adjusted_notional = notional / leverage
margin = adjusted_notional * instrument.margin_init
```

- 初始保证金 = `(名义价值 / 杠杆) * instrument.margin_init`
- 维持保证金 = `(名义价值 / 杠杆) * instrument.margin_maint`

**使用场景：**

- 使用杠杆降低保证金的加密货币交易所。
- 杠杆影响保证金要求的交易场所。

### 用法

#### 编程配置

```python
from nautilus_trader.backtest.models import LeveragedMarginModel
from nautilus_trader.backtest.models import StandardMarginModel
from nautilus_trader.test_kit.stubs.execution import TestExecStubs

# 创建账户
account = TestExecStubs.margin_account()

# 为传统经纪商设置标准模型
standard_model = StandardMarginModel()
account.set_margin_model(standard_model)

# 或为加密货币交易所使用杠杆模型
leveraged_model = LeveragedMarginModel()
account.set_margin_model(leveraged_model)
```

#### 回测配置

```python
from nautilus_trader.backtest.config import BacktestVenueConfig
from nautilus_trader.backtest.config import MarginModelConfig

venue_config = BacktestVenueConfig(
    name="SIM",
    oms_type="NETTING",
    account_type="MARGIN",
    starting_balances=["1_000_000 USD"],
    margin_model=MarginModelConfig(model_type="standard"),  # 选项：'standard'、'leveraged'
)
```

#### 可用模型类型

- `"leveraged"`：保证金除以杠杆（默认）。
- `"standard"`：固定比例（传统经纪商）。
- 自定义类路径：`"my_package.my_module.MyMarginModel"`。

#### 默认行为

默认情况下，`MarginAccount` 使用 `LeveragedMarginModel`。

#### 真实世界示例

**EUR/USD 交易场景：**

- **金融工具**：EUR/USD
- **数量**：100,000 EUR
- **价格**：1.10000
- **名义价值**：$110,000
- **杠杆**：50 倍
- **金融工具初始保证金**：3%

**保证金计算：**

| 模型     | 计算公式           | 结果  | 百分比 |
|-----------|----------------------|---------|------------|
| Standard  | $110,000 x 0.03      | $3,300  | 3.00%      |
| Leveraged | ($110,000 / 50) x 0.03 | $66   | 0.06%      |

**账户余额影响：**

- **账户余额**：$10,000
- **Standard 模型**：无法交易（需要 $3,300 保证金）
- **Leveraged 模型**：可以交易（仅需 $66 保证金）

### 真实世界场景

#### 盈透证券 EUR/USD 期货

```python
# IB 要求固定保证金，与杠杆无关
account.set_margin_model(StandardMarginModel())
margin = account.calculate_margin_init(instrument, quantity, price)
# 结果：名义价值的固定百分比
```

#### 币安加密货币交易

```python
# 币安可能使用杠杆降低保证金
account.set_margin_model(LeveragedMarginModel())
margin = account.calculate_margin_init(instrument, quantity, price)
# 结果：保证金按杠杆系数降低
```

### 模型选择

#### 使用默认模型

默认的 `LeveragedMarginModel` 开箱即用：

```python
account = TestExecStubs.margin_account()
margin = account.calculate_margin_init(instrument, quantity, price)
```

#### 使用标准模型

用于传统经纪商行为：

```python
account.set_margin_model(StandardMarginModel())
margin = account.calculate_margin_init(instrument, quantity, price)
```

### 自定义模型

你可以通过继承 `MarginModel` 来创建自定义保证金模型。自定义模型通过 `MarginModelConfig` 接收配置：

```python
from nautilus_trader.backtest.models import MarginModel
from nautilus_trader.backtest.config import MarginModelConfig

class RiskAdjustedMarginModel(MarginModel):
    def __init__(self, config: MarginModelConfig):
        """使用配置参数初始化。"""
        self.risk_multiplier = Decimal(str(config.config.get("risk_multiplier", 1.0)))
        self.use_leverage = config.config.get("use_leverage", False)

    def calculate_margin_init(self, instrument, quantity, price, leverage, use_quote_for_inverse=False):
        notional = instrument.notional_value(quantity, price, use_quote_for_inverse)
        if self.use_leverage:
            adjusted_notional = notional.as_decimal() / leverage
        else:
            adjusted_notional = notional.as_decimal()
        margin = adjusted_notional * instrument.margin_init * self.risk_multiplier
        return Money(margin, instrument.quote_currency)

    def calculate_margin_maint(self, instrument, side, quantity, price, leverage, use_quote_for_inverse=False):
        return self.calculate_margin_init(instrument, quantity, price, leverage, use_quote_for_inverse)
```

#### 使用自定义模型

**编程方式：**

```python
from nautilus_trader.backtest.config import MarginModelConfig
from nautilus_trader.backtest.config import MarginModelFactory

config = MarginModelConfig(
    model_type="my_package.my_module:RiskAdjustedMarginModel",
    config={"risk_multiplier": 1.5, "use_leverage": False}
)

custom_model = MarginModelFactory.create(config)
account.set_margin_model(custom_model)
```

### 高级回测 API 配置

使用高级回测 API 时，你可以使用 `MarginModelConfig` 在交易场所配置中指定保证金模型：

```python
from nautilus_trader.backtest.config import MarginModelConfig
from nautilus_trader.backtest.config import BacktestVenueConfig
from nautilus_trader.config import BacktestRunConfig

# 使用特定保证金模型配置交易场所
venue_config = BacktestVenueConfig(
    name="SIM",
    oms_type="NETTING",
    account_type="MARGIN",
    starting_balances=["1_000_000 USD"],
    margin_model=MarginModelConfig(
        model_type="standard"  # 使用标准模型模拟传统经纪商
    ),
)

# 在回测配置中使用
config = BacktestRunConfig(
    venues=[venue_config],
    # ... 其他配置
)
```

#### 配置示例

**标准模型（传统经纪商）：**

```python
margin_model=MarginModelConfig(model_type="standard")
```

**杠杆模型（默认）：**

```python
margin_model=MarginModelConfig(model_type="leveraged")  # 默认
```

**带配置的自定义模型：**

```python
margin_model=MarginModelConfig(
    model_type="my_package.my_module:CustomMarginModel",
    config={
        "risk_multiplier": 1.5,
        "use_leverage": False,
        "volatility_threshold": 0.02,
    }
)
```

保证金模型将在回测执行期间自动应用于模拟交易所 (SimulatedExchange)。
