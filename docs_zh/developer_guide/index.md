# 开发者指南

欢迎阅读 NautilusTrader 开发者指南（Developer Guide）！

在这里，你将了解如何开发和扩展 NautilusTrader 以满足你的交易需求，或为项目贡献改进。

:::info
本指南的结构设计使自动化工具和人类读者都能方便地使用。
:::

我们坚信选择合适的工具来完成任务。整体设计哲学是充分利用 Python 的高层能力及其丰富的框架和库生态系统，同时利用 Rust 处理性能关键（performance-critical）组件并提供全面的类型安全。

NautilusTrader 采用 **Rust 核心 + Python 绑定** 的架构：

- **Rust** 处理网络通信、数据解析、订单撮合及其他性能关键操作。
- **Python** 提供面向用户的 API，用于策略开发、配置和系统集成。
- **PyO3** 作为桥梁，以极低的开销将 Rust 功能暴露给 Python。

这种方式结合了 Python 的简洁性和生态优势与 Rust 的性能和内存安全。

## 目录

- [环境搭建](environment_setup.md)
- [设计原则](design_principles.md)
- [编码规范](coding_standards.md)
- [Rust](rust.md)
- [Python](python.md)
- [测试](testing.md)
- [测试数据集](test_datasets.md)
- [文档风格](docs.md)
- [发布说明](releases.md)
- [发布安全架构](release_security.md)
- [适配器](adapters.md)
- [数据测试规范](spec_data_testing.md)
- [执行测试规范](spec_exec_testing.md)
- [基准测试](benchmarking.md)
- [FFI 内存契约](ffi.md)
