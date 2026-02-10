# Blockchain

## 核心原语

Nautilus Trader 的区块链集成（Integration）建立在 DeFi 领域模型（`nautilus_model::defi`）中定义的基础原语之上。这些构建块为基于 EVM 的区块链提供了类型安全的抽象。

### Chain

`Chain` 结构体表示一个区块链网络及其连接端点和元数据（Metadata）。每个 Chain 实例包含：

**字段：**

- `name` (`Blockchain`)：区块链网络类型（包含 80 多条支持链的枚举）
- `chain_id` (`u32`)：唯一的 EVM 链标识符（例如，1 表示 Ethereum，42161 表示 Arbitrum）
- `hypersync_url` (`String`)：用于高性能 Hypersync 数据流的端点
- `rpc_url` (`Option<String>`)：可选的 HTTP/WSS RPC 端点，用于与节点直接通信
- `native_currency_decimals` (`u8`)：链原生 Gas 代币的小数精度（通常为 18）

**链检索：**

可以通过数字 ID 或字符串名称（不区分大小写）检索链：

- **按链 ID：** 使用 EVM 链标识符通过 `from_chain_id` 查找
- **按名称：** 使用区块链名称（不区分大小写："ethereum"、"Ethereum"、"ETHEREUM" 均可）通过 `from_chain_name` 查找
- **静态实例：** 以常量形式提供的预配置链

每条链都有一种用于支付 Gas 费用的原生货币。`native_currency()` 方法返回一个正确配置的 Currency 实例：

| 链族                                             | 代码 | 名称         | 小数位 |
|-------------------------------------------------|------|--------------|--------|
| Ethereum 及 L2（Arbitrum、Base、Optimism 等）     | ETH  | Ethereum     | 18     |
| Polygon                                         | POL  | Polygon      | 18     |
| Avalanche                                       | AVAX | Avalanche    | 18     |
| BSC                                             | BNB  | Binance Coin | 18     |

## 合约

用于查询 EVM 智能合约的高性能接口，具有类型安全的 Rust 抽象。通过高效的批量操作支持代币元数据、DEX 池和 DeFi 协议。

### Base (Multicall3)

使用 Multicall3（`0xcA11bde05977b3631167028862bE2a173976CA11`）将多个合约调用批量打包到一个 RPC 请求中。

- 始终使用 `allow_failure: true` 以实现部分成功和详细错误信息
- 在同一区块内原子执行
- 错误类型：`RpcError`（网络问题）、`AbiDecodingError`（解码失败）

### ERC20

继承自 `BaseContract`，利用 Multicall3 实现高效的批量操作。获取代币元数据时对非标准实现有健壮的处理能力。

**方法：**

- `fetch_token_info`：获取单个代币元数据（内部使用 multicall 获取 name、symbol、decimals）
- `batch_fetch_token_info`：在一次 multicall 中获取多个代币（每个代币 3 次调用）
- `enforce_token_fields`：验证 name/symbol 非空

**错误类型：**

1. **`CallFailed`** - 合约不存在或函数未实现 → 跳过该代币
2. **`DecodingError`** - 原始字节而非 ABI 编码（例如 `0x5269636f...`）→ 跳过该代币
3. **`EmptyTokenField`** - 函数返回空字符串 → 如启用强制检查则跳过

**最佳实践：**

- 跳过包含任何代币错误的池
- `raw_data` 字段保留原始响应以供调试
- 非标准代币通常存在其他问题（转账手续费、变基等）

## 配置

| 选项                            | 默认值  | 描述 |
|---------------------------------|---------|------|
| `chain`                         | 必填    | 要同步的 `nautilus_trader.model.Chain`（例如 `Chain.ETHEREUM`）。 |
| `dex_ids`                       | 必填    | 描述要启用哪些 DEX 集成的 `DexType` 标识符序列。 |
| `http_rpc_url`                  | 必填    | 用于 EVM 调用和 Multicall 请求的 HTTPS RPC 端点。 |
| `wss_rpc_url`                   | `None`  | 可选的 WSS 端点，用于流式实时更新。 |
| `rpc_requests_per_second`       | `None`  | 可选的出站 RPC 调用限流（每秒请求数）。 |
| `multicall_calls_per_rpc_request` | `100` | 每个 RPC 请求中批量打包的最大 Multicall 目标数。 |
| `use_hypersync_for_live_data`   | `True`  | 设为 `True` 时，使用 Hypersync 进行引导和流式传输以获得更低延迟的差异数据。 |
| `from_block`                    | `None`  | 可选的历史回填起始区块高度。 |
| `pool_filters`                  | `DexPoolFilters()` | 选择要监控的 DEX 池时应用的过滤规则。 |
| `postgres_cache_database_config`| `None`  | 可选的 `PostgresConnectOptions`，用于启用已解码池状态的磁盘缓存。 |
| `http_proxy_url`                | `None`  | 可选的 RPC 请求 HTTP 代理 URL。 |
| `ws_proxy_url`                  | `None`  | 可选的 RPC 连接 WebSocket 代理 URL。 |
