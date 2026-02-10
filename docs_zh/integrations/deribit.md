# Deribit

Deribit 成立于 2016 年，是一家专注于比特币和以太坊期权（Options）及期货（Futures）的加密货币衍生品交易所。
按交易量计算，它是最大的加密期权交易所之一。
此集成（Integration）支持 Deribit 的实时市场数据接入和订单执行（Execution）。

:::warning
此集成目前正在开发中，尚未准备好投入使用。
:::

## 概述

此适配器（Adapter）使用 Rust 实现，并提供可选的 Python 绑定以用于基于 Python 的工作流。
Deribit 使用 JSON-RPC 2.0 协议，同时支持 HTTP 和 WebSocket 传输（而非 REST）。
WebSocket 是订阅和实时数据的首选方式。

官方 Deribit API 参考文档可在 <https://docs.deribit.com/v2/> 查阅。
