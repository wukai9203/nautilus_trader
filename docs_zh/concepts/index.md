# 概念

概念 (Concepts) 指南介绍并解释了 NautilusTrader 平台的基础理念、组件和最佳实践。
这些指南旨在提供概念性和实践性的见解，帮助你理解系统的架构 (architecture)、策略 (strategy)、数据管理、执行 (execution) 流程等。
浏览以下指南，加深对 NautilusTrader 的理解并充分发挥其能力。

## [概述](overview.md)

**概述 (Overview)** 指南涵盖了平台的主要功能和预期使用场景。

## [架构](architecture.md)

**架构** 指南深入探讨了平台的基础原则、结构和设计。
它适合开发者、系统架构师，或任何对 NautilusTrader 内部运作机制感兴趣的人。

## [Actor](actors.md)

`Actor` 是与交易系统交互的基础组件。
**Actor** 指南涵盖了其功能和实现细节。

## [策略](strategies.md)

`Strategy` 是 NautilusTrader 用户体验的核心，用于编写和使用交易策略。
**策略** 指南介绍了如何为平台实现策略。

## [金融工具](instruments.md)

**金融工具 (Instruments)** 指南涵盖了可交易资产和合约的不同工具定义规范。

## [数据](data.md)

NautilusTrader 平台定义了一系列专为交易领域设计的内置数据 (data) 类型。
**数据** 指南介绍了如何使用内置数据和自定义数据。

## [执行](execution.md)

NautilusTrader 可以同时处理多个策略和交易场所 (venue) 的交易执行和订单 (order) 管理（每个实例）。
**执行** 指南涵盖了参与执行的组件，以及执行消息（命令和事件）的流转方式。

## [订单](orders.md)

**订单** 指南提供了平台可用订单类型的更多细节，以及每种类型支持的执行指令。
高级订单类型和模拟订单也在其中进行了介绍。

## [持仓](positions.md)

**持仓 (Positions)** 指南解释了 NautilusTrader 中持仓的运作方式，包括其生命周期、
从订单成交 (trade) 中的聚合、盈亏计算，以及净额持仓管理系统 (netting OMS) 配置中
持仓快照的重要概念。

## [缓存](cache.md)

`Cache` 是用于管理所有交易相关数据的核心内存数据存储。
**缓存 (Cache)** 指南涵盖了缓存的功能和最佳实践。

## [消息总线](message_bus.md)

`MessageBus` 是实现组件间解耦消息传递模式的核心通信系统，
包括点对点、发布/订阅和请求/响应模式。
**消息总线 (Message Bus)** 指南涵盖了 `MessageBus` 的功能和最佳实践。

## [值类型](value_types.md)

`Price`、`Quantity` 和 `Money` 是 NautilusTrader 用于表示核心交易概念的专用值类型，
基于定点算术实现高性能且确定性的计算。
**值类型 (Value Types)** 指南涵盖不可变性、算术运算、精度处理和类型约束。

## [投资组合](portfolio.md)

`Portfolio` 作为中央枢纽，负责管理和跟踪交易节点或回测 (backtest) 中所有活跃策略的持仓。
它整合了来自多个金融工具的持仓数据，提供持仓、风险 (risk) 敞口和整体绩效的统一视图。
浏览本节以了解 NautilusTrader 如何聚合和更新投资组合 (Portfolio) 状态，以支持有效的交易和风险管理。

## [报告](reports.md)

**报告 (Reports)** 指南涵盖了 NautilusTrader 的报告功能，包括执行报告、
投资组合分析报告、盈亏核算注意事项，以及报告在回测后分析中的使用方式。

## [日志](logging.md)

平台为回测和实盘交易 (live trading) 提供日志 (logging) 功能，使用 Rust 实现的高性能日志记录器。

## [回测](backtesting.md)

使用 NautilusTrader 进行回测是一个系统化的模拟过程，它使用特定的系统实现来复现交易活动。

## [可视化](visualization.md)

**可视化 (Visualization)** 指南介绍了用于分析回测结果的交互式分析面板系统，
包括可用图表、主题、自定义选项，以及如何使用可扩展的图表注册机制创建自定义可视化。

## [实盘交易](live.md)

NautilusTrader 中的实盘交易使交易者能够将经过回测验证的策略实时部署，
无需修改任何代码。这种无缝过渡确保了一致性和可靠性，但回测和实盘交易之间存在一些关键差异。

## [适配器](adapters.md)

NautilusTrader 的设计允许通过适配器 (adapter) 实现来集成数据提供商和/或交易场所。
**适配器** 指南涵盖了为平台开发新集成适配器的要求和最佳实践。

:::note
[API 参考](../api_reference/index.md) 文档应被视为平台的权威信息来源。
如果此处描述的概念与 API 参考之间存在任何差异，
则应以 API 参考为准。我们正在努力确保概念文档与 API 参考保持同步，
并将在不久的将来引入文档测试来帮助实现这一目标。
:::
