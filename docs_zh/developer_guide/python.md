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

:::warning[弃用通知]
Cython 正在被 Rust 实现逐步替代。新代码应使用 Rust。本节仅记录遗留的 Cython 代码。
:::

对于遗留的 `.pyx` 和 `.pxd` 文件，确保所有返回 `void` 或 C 原始类型（如 `bint`、`int`、`double`）的函数和方法签名中包含 `except *` 关键字。这确保 Python 异常不会被忽略。

更多信息请参阅 [Cython 文档](https://cython.readthedocs.io/en/latest/index.html)。
