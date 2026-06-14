# Python

[Python](https://www.python.org/) 编程语言用于 NautilusTrader 中大部分面向用户的代码。
Python 拥有丰富的库和框架生态系统，非常适合策略开发、数据分析和系统集成。

## 代码风格

### PEP-8

代码库总体遵循 PEP-8 风格指南。
一个显著的不同是，对于集合以外的类型，并不总是利用 Python 的真值性（truthiness）来检查参数是否为 `None`。

按照 [Google Python Style Guide](https://google.github.io/styleguide/pyguide.html) 的建议，不推荐使用真值性来检查参数是否为 `None`，因为可能有意外的对象传入函数或方法，导致意外的真值判断（进而可能产生逻辑错误类型的 bug）。

*"Always use if foo is None: (or is not None) to check for a None value. E.g., when testing whether a variable or argument that defaults to None was set to some other value. The other value might be a value that's false in a boolean context!"*

:::note
使用真值性来检查空集合（如 `if not my_list:`），而非显式与 `None` 或空值比较。
:::

我们欢迎对代码库中无明显理由偏离 PEP-8 之处的反馈。

### 类型提示（Type hints）

所有函数和方法签名*必须*包含完整的类型注解：

```python
def __init__(self, config: EMACrossConfig) -> None:
def on_bar(self, bar: Bar) -> None:
def on_save(self) -> dict[str, bytes]:
def on_load(self, state: dict[str, bytes]) -> None:
```

**Union 语法**：使用 PEP 604 的 union 语法表示可选类型：

```python
# 推荐
def get_instrument(self, id: InstrumentId) -> Instrument | None:

# 避免
def get_instrument(self, id: InstrumentId) -> Optional[Instrument]:
```

**泛型类型**：可复用组件使用 `TypeVar`：

```python
T = TypeVar("T")
class ThrottledEnqueuer(Generic[T]):
```

### 文档字符串（Docstrings）

整个代码库使用 [NumPy 文档字符串规范](https://numpydoc.readthedocs.io/en/latest/format.html)。
需要一致地遵守此规范以确保文档正确构建。

**Python** 文档字符串应使用**祈使语气（imperative mood）** -- 例如 *"Return a cached client."*

此约定与 Python 生态的主流风格一致，使生成的文档对最终用户更加自然。

#### 私有方法

不要为私有方法（以 `_` 为前缀的方法）添加文档字符串：

- 文档字符串会生成面向公众的 API 文档。
- 私有方法上的文档字符串会错误地暗示它们是公共 API 的一部分。
- 私有方法是实现细节，不面向最终用户。

可以添加文档字符串的例外情况：

- 非常复杂的方法，包含不简单的逻辑、多个步骤或重要的边界情况。
- 由于复杂性需要详细参数或返回值文档的方法。

当私有方法需要上下文说明（如棘手的前置条件或副作用）时，优先在相关逻辑附近使用简短的内联注释（`#`）而非文档字符串。

### 属性 vs 方法（PyO3 绑定）

通过 PyO3 将 Rust 类型暴露给 Python 时，应基于调用点要传达的语义来选择使用 `#[getter]`（属性）还是普通方法，而不是基于该值是否会变化：

- **属性（`#[getter]`）：** 廉价、无副作用、类似属性访问的当前状态视图。标量字段、谓词以及轻量级的派生值都属于此类，即使它们在对象生命周期内会发生变化也是如此。
  示例：`status`、`side`、`quantity`、`price`、`is_open`、`has_inputs`、`realized_pnl`、`venue_order_id`。
- **方法（无 `#[getter]`）：** 动作、变更、非平凡的工作、分配/拷贝、I/O，或任何带参数的操作。
  示例：`apply(fill)`、`unrealized_pnl(price)`、`calculate_pnl(...)`。
- **灰色地带（优先用方法）：** 每次调用都会克隆或分配一个集合的 getter。使用方法可以向调用方传达其开销。
  示例：`events()`、`adjustments()`、`client_order_ids()`、`trade_ids()`。

## Python v2 实盘回调路由

Python v2 实盘节点保持一条运行时不变量：在实盘交易期间，Tokio 工作线程不运行 Python 代码。

`LiveNode::py_run` 在 Rust 异步运行时运行期间会释放 GIL。工作线程侧需要触发 Python 的工作不会在工作线程上调用 `Python::attach`，而是使用现有的实盘 runner 事件通道。定时器回调使用 time-event 通道。runner 会在启动缓冲阶段和主 select 循环中排空该通道，然后在实盘事件循环线程上执行回调。

这条路径是用于处理不可避免的用户 Python 回调工作的边界。它不是把 adapter、provider、data 或 execution 逻辑搬进 Python 的地方。Python v2 的 adapter 模块负责配置 Rust adapter 并注册 factory；adapter 的操作由 Rust 拥有。如果工作线程侧的 Rust 工作需要一个 Python 回调，应通过一个属于实盘 runner 的特定事件类型来路由。

添加可感知 Python 的实盘代码时：

- 优先使用现有的 runner 事件通道。
- 保持回调体简短，因为它们会在实盘事件循环上同步运行。
- 不要在 Python v2 实盘交易中从 Tokio 工作任务里调用 `Python::attach`。
- 不要为了适配回调路由而在 Python 中添加 adapter 业务逻辑。

遗留的 Cython `LiveClock` 回调是一条独立的 FFI 路径。它们使用 capsule 风格的回调参数以兼容 v1，并且可以在没有实盘 runner sender 的情况下创建。在 time event 派发能够跨 v1 与 v2 统一之前，应保持该 ABI 相互独立。

### 测试命名

使用描述性名称说明场景：

```python
def test_currency_with_negative_precision_raises_overflow_error(self):
def test_sma_with_no_inputs_returns_zero_count(self):
def test_sma_with_single_input_returns_expected_value(self):
```

### Ruff

[ruff](https://astral.sh/ruff) 用于对代码库进行代码检查。Ruff 规则可以在顶层 `pyproject.toml` 中找到，忽略规则通常有注释说明。

## Cython（遗留）

:::note
本节介绍适用于 `.pyx` 和 `.pxd` 文件的 Cython 约定。
:::

:::warning[弃用通知]
Cython 正在被 Rust 实现逐步替代。新代码应使用 Rust。本节仅记录遗留的 Cython 代码。
:::

对于 `.pyx` 和 `.pxd` 文件，确保所有返回 `void` 或 C 原始类型（如 `bint`、`int`、`double`）的函数和方法签名中包含 `except *` 关键字。否则，Python 异常会被静默忽略。

更多信息请参阅 [Cython 文档](https://cython.readthedocs.io/en/latest/index.html)。
