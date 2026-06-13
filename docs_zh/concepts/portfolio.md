# 投资组合 (Portfolio)

`Portfolio` 是管理和跟踪交易节点或回测 (backtest) 中所有活跃策略 (strategy) 持仓 (position) 的核心枢纽。
它整合来自多个金融工具 (instrument) 的持仓数据，提供持仓、风险敞口和整体表现的统一视图。
通过本节了解 NautilusTrader 如何聚合和更新投资组合状态，以支持有效的交易和风险管理。

## 货币转换 (Currency Conversion)

`Portfolio` 支持对盈亏和敞口计算进行自动货币转换，
允许你以首选货币查看结果。当跨多个结算货币不同的金融工具交易，
或管理基础货币不同的多个账户时，此功能尤为实用。

### 支持的转换

以下投资组合查询方法支持货币转换：

- `realized_pnl()` / `realized_pnls()` - 将已实现盈亏转换为目标货币。
- `unrealized_pnl()` / `unrealized_pnls()` - 将未实现盈亏转换为目标货币。
- `total_pnl()` / `total_pnls()` - 将总盈亏转换为目标货币。
- `net_exposure()` / `net_exposures()` - 将净风险敞口转换为目标货币。

所有方法均接受可选的 `target_currency` 参数以指定所需的输出货币。

### 单账户行为

查询单个账户且未指定 `target_currency` 时，`Portfolio`
会自动将值转换为该账户的基础货币：

```python
# 以账户基础货币（如 USD）返回敞口
exposure = portfolio.net_exposures(venue=BINANCE, account_id=account_id)
```

### 多账户行为

同时查询多个账户时，行为取决于查询所有金融工具（`net_exposures()`）
还是单个金融工具（`net_exposure()`）：

**`net_exposures()`（所有金融工具）：**

- **相同基础货币**：自动转换为公共基础货币。
- **不同基础货币**：返回包含多种货币的字典，每种货币已转换为对应账户的基础货币。
  提供 `target_currency` 可获得单货币结果。

**`net_exposure()`（跨账户的单个金融工具）：**

- **不同基础货币**：除非提供 `target_currency`，否则返回 `None`。

```python
# 场景 1：多账户，均以 USD 为基础货币
exposures = portfolio.net_exposures(venue=BINANCE)
# 返回 {USD: Money(...)}

# 场景 2：多账户，基础货币不同（USD 和 EUR）
exposures = portfolio.net_exposures(venue=BINANCE)
# 返回 {USD: Money(...), EUR: Money(...)}

# 强制跨账户使用单一货币
exposures = portfolio.net_exposures(venue=BINANCE, target_currency=USD)
# 返回 {USD: Money(...)}
```

### 转换失败处理

当提供 `target_currency` 但货币转换失败时，行为取决于方法类型：

- **单值方法**（`realized_pnl`、`unrealized_pnl`、`total_pnl`、`net_exposure`）：
  返回 `None` 并记录错误，以防止返回不正确的值。
- **返回字典的方法**（`realized_pnls`、`unrealized_pnls`、`total_pnls`、`net_exposures`）：
  跳过转换失败的金融工具，但返回转换成功的结果。

:::warning
使用 `target_currency` 进行跨币种聚合时，请确保汇率数据可用。
:::

### 转换价格类型

将敞口转换为目标货币时，`Portfolio` 根据持仓构成使用不同的价格类型：

- **全多头持仓**：使用 `BID` 价格（对多头敞口保守估计）。
- **全空头持仓**：使用 `ASK` 价格（对空头敞口保守估计）。
- **多空混合持仓**：使用 `MID` 价格（多空并存时取中间价）。

这确保转换反映了真实市场条件：多头持仓以买价平仓，空头持仓以卖价回补。
对于混合持仓，中间价提供中性估值。

如果在投资组合配置中启用了 `use_mark_xrates`，则对于混合持仓和一般转换，
`MARK` 价格将替代 `MID` 价格。

## 投资组合统计

NautilusTrader 提供了多种[内置投资组合统计指标](https://github.com/nautechsystems/nautilus_trader/tree/develop/crates/analysis/src/statistics)，
用于分析回测和实盘交易中投资组合的表现。

统计指标大致分为以下几类：

- 基于盈亏 (PnL) 的统计（按币种）
- 基于收益率 (Returns) 的统计
- 基于持仓的统计
- 基于订单 (order) 的统计

你也可以随时调用交易者的 `PortfolioAnalyzer` 来计算统计指标，
包括在回测*进行中*或实盘交易期间。

## 自定义统计

可以通过继承 `PortfolioStatistic` 基类并实现任意 `calculate_` 方法来定义自定义投资组合统计指标。

例如，以下是内置 `WinRate`（胜率）统计的实现：

```python
import pandas as pd
from typing import Any
from nautilus_trader.analysis.statistic import PortfolioStatistic


class WinRate(PortfolioStatistic):
    """
    根据已实现盈亏 (realized PnL) 序列计算胜率。
    """

    def calculate_from_realized_pnls(self, realized_pnls: pd.Series) -> Any | None:
        # 前置条件检查
        if realized_pnls is None or realized_pnls.empty:
            return 0.0

        # 计算统计指标
        winners = [x for x in realized_pnls if x > 0.0]
        losers = [x for x in realized_pnls if x <= 0.0]

        return len(winners) / float(max(1, (len(winners) + len(losers))))
```

:::tip 统计方法选择指南

`PortfolioStatistic` 基类提供四种数据源方法，对应不同的分析需求：

| 方法 | 数据来源 | 适用统计类型 |
|------|---------|------------|
| `calculate_from_realized_pnls(pnls: pd.Series)` | 已实现盈亏序列（浮点数，按币种聚合） | 胜率、盈亏比、平均盈利/亏损、期望值 |
| `calculate_from_returns(returns: pd.Series)` | 持仓收益率时间序列 | 夏普比率、最大回撤、年化收益率、波动率 |
| `calculate_from_positions(positions: list)` | 原始持仓对象列表 | 持仓持续时间统计、持仓频率、多空比 |
| `calculate_from_orders(orders: list)` | 原始订单对象列表 | 成交率、撤单率、订单执行质量 |

实现自定义统计时，只需重写用到的方法即可（可同时重写多个）。系统在回测结束后会调用所有已注册统计的全部 `calculate_from_*` 方法，并将结果显示在绩效报告中。
:::

然后可以将这些统计指标注册到交易者的 `PortfolioAnalyzer` 中。

```python
stat = WinRate()

# 注册到投资组合分析器
engine.portfolio.analyzer.register_statistic(stat)
```

:::info
有关可用方法的详细信息，请参阅 `PortfolioAnalyzer` [API 参考](../api_reference/analysis.md#class-portfolioanalyzer)。
:::

:::tip
请确保你的统计指标能够稳健地处理退化输入，例如 ``None``、空序列或数据不足的情况。
对于未知/无法计算的值返回 ``None``，或在语义上合理时返回合理的默认值（如 ``0.0``，例如没有交易时的胜率）。
:::

## 回测分析

回测运行结束后，系统会依次将已实现盈亏、收益率、持仓和订单数据传递给每个已注册的统计指标来执行绩效分析。所有输出结果将显示在绩效报告的 `Portfolio Performance` 标题下，分组如下：

- 已实现盈亏统计（按币种）
- 收益率统计（针对整个投资组合）
- 基于持仓和订单数据的综合统计（针对整个投资组合）
