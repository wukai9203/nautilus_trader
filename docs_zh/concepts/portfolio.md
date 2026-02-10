# 投资组合 (Portfolio)

`Portfolio` 是管理和跟踪交易节点或回测 (backtest) 中所有活跃策略 (strategy) 持仓 (position) 的核心枢纽。
它整合来自多个金融工具 (instrument) 的持仓数据，提供持仓、风险敞口和整体表现的统一视图。
通过本节了解 NautilusTrader 如何聚合和更新投资组合状态，以支持有效的交易和风险管理。

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
