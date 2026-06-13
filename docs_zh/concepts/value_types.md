# 值类型 (Value Types)

NautilusTrader 提供了用于表示核心交易概念的专用值类型：
`Price`（价格）、`Quantity`（数量）和 `Money`（货币金额）。这些类型在内部使用定点算术 (fixed-point arithmetic)
以确保在不同平台和环境中实现高性能且确定性的计算。

## 概览

| 类型       | 用途                                   | 有符号 | 货币 |
|------------|----------------------------------------|--------|------|
| `Quantity` | 交易数量、订单金额、持仓规模。           | 否     | -    |
| `Price`    | 市场价格、报价、价格水平。               | 是     | -    |
| `Money`    | 货币金额、盈亏、账户余额。               | 是     | 是   |

## 不可变性

所有值类型均为**不可变 (immutable)**。一旦构造完成，值便无法更改。
算术运算始终返回新实例，而非修改现有实例。

```python
from nautilus_trader.model.objects import Quantity

qty1 = Quantity(100, precision=0)
qty2 = Quantity(50, precision=0)

# 这将创建一个新的 Quantity；qty1 和 qty2 保持不变
result = qty1 + qty2

print(qty1)    # 100
print(qty2)    # 50
print(result)  # 150
```

这种设计带来以下优势：

- **线程安全**：不可变值无需同步即可在线程间安全共享。
- **可预测性**：值不会意外改变，使调试更为简单。
- **可哈希性**：不可变类型可用作字典键和集合元素。

## 算术运算

值类型支持标准算术运算符（`+`、`-`、`*`、`/`、`%`、`//`）。
返回类型取决于操作数类型。

### 同类型运算

当两个操作数为相同值类型时，结果也为该类型：

| 运算                  | 结果       |
|-----------------------|------------|
| `Quantity + Quantity` | `Quantity` |
| `Quantity - Quantity` | `Quantity` |
| `Price + Price`       | `Price`    |
| `Price - Price`       | `Price`    |
| `Money + Money`       | `Money`    |
| `Money - Money`       | `Money`    |

```python
from nautilus_trader.model.objects import Price

price1 = Price(100.50, precision=2)
price2 = Price(0.25, precision=2)

result = price1 + price2  # 返回 Price(100.75, precision=2)
print(type(result))       # <class 'Price'>
```

### 混合类型运算

与其他数值类型运算时，返回类型遵循 Python 的
[数值塔 (numeric tower)](https://docs.python.org/3/library/numbers.html) 约定。基本
原则是运算结果向更通用的类型扩展：`float` 运算返回 `float`，
而 `int` 和 `Decimal` 运算返回 `Decimal` 以保持精度。

| 左操作数    | 右操作数      | 结果类型    |
|-------------|---------------|-------------|
| 值类型      | `int`         | `Decimal`   |
| 值类型      | `float`       | `float`     |
| 值类型      | `Decimal`     | `Decimal`   |
| `int`       | 值类型        | `Decimal`   |
| `float`     | 值类型        | `float`     |
| `Decimal`   | 值类型        | `Decimal`   |

```python
from decimal import Decimal
from nautilus_trader.model.objects import Quantity

qty = Quantity(100, precision=0)

# Quantity + int → Decimal
result1 = qty + 50
print(type(result1))  # <class 'decimal.Decimal'>

# Quantity + float → float
result2 = qty + 50.5
print(type(result2))  # <class 'float'>

# Quantity + Decimal → Decimal
result3 = qty + Decimal("50")
print(type(result3))  # <class 'decimal.Decimal'>
```

## 精度处理

每种值类型都存储一个精度字段，表示小数位数。
对不同精度的值进行算术运算时，结果采用操作数中的最大精度。

```python
from nautilus_trader.model.objects import Price

price1 = Price(100.5, precision=1)    # 1 位小数
price2 = Price(0.125, precision=3)    # 3 位小数

result = price1 + price2
print(result)            # 100.625
print(result.precision)  # 3（1 和 3 中的最大值）
```

## 类型特定约束

### Quantity

`Quantity` 表示非负数量。尝试创建负数量或从较小数量中减去较大数量将引发错误：

```python
from nautilus_trader.model.objects import Quantity

# 这将引发 ValueError: Quantity cannot be negative
qty = Quantity(-100, precision=0)

# 这也会引发 ValueError
qty1 = Quantity(50, precision=0)
qty2 = Quantity(100, precision=0)
result = qty1 - qty2  # 结果为 -50，这是无效的
```

### Money

`Money` 值包含货币信息。`Money` 值之间的算术运算要求货币一致：

```python
from nautilus_trader.model.objects import Money
from nautilus_trader.model.currencies import USD, EUR

usd_amount = Money(100.00, USD)
eur_amount = Money(50.00, EUR)

# 这可以正常工作——货币相同
result = usd_amount + Money(25.00, USD)

# 这将引发 ValueError——货币不匹配
result = usd_amount + eur_amount
```

## 常用模式

### 累积值

由于值类型是不可变的，通过重新赋值来累积：

```python
from nautilus_trader.model.objects import Money
from nautilus_trader.model.currencies import USD

total = Money(0.00, USD)
amounts = [Money(100.00, USD), Money(50.00, USD), Money(25.00, USD)]

for amount in amounts:
    total = total + amount  # 重新赋值为新的 Money 实例

print(total)  # 175.00 USD
```

### 转换为其他类型

值类型提供转换方法：

```python
from nautilus_trader.model.objects import Price

price = Price(123.456, precision=3)

# 转换为 Decimal（保留精度）
decimal_value = price.as_decimal()

# 转换为 float
float_value = price.as_double()

# 转换为字符串
string_value = str(price)  # "123.456"
```

### 从字符串创建

从字符串表示解析值类型：

```python
from nautilus_trader.model.objects import Quantity, Price, Money

qty = Quantity.from_str("100.5")
price = Price.from_str("99.95")
money = Money.from_str("1000.00 USD")
```
