# 发布说明

本指南记录了在 `RELEASES.md` 中编写发布说明（Release Notes）的规范。

## 章节

按以下顺序使用章节：

1. 功能增强（Enhancements）
2. 破坏性变更（Breaking Changes）
3. 安全（Security）
4. 修复（Fixes）
5. 内部改进（Internal Improvements）
6. 文档更新（Documentation Updates）
7. 弃用（Deprecations）

某次发布中没有条目的章节应省略。

### 功能增强

新功能和用户可见的改进。

**格式**：

```markdown
- Added `subscribe_order_fills(...)` and `unsubscribe_order_fills(...)` for `Actor`
- Added BitMEX conditional orders support
- Added support for `OrderBookDepth10` requests (#2955), thanks @faysou
```

**指南**：

- 以 "Added" 开头。
- 代码元素使用反引号。
- 具体说明添加了什么，而非如何实现。

### 破坏性变更

可能破坏现有代码的更改。

**格式**：

```markdown
- Removed `nautilus_trader.analysis.statistics` subpackage - must import from `nautilus_trader.analysis`
- Renamed `BinanceAccountType.USDT_FUTURE` to `USDT_FUTURES`
- Changed `start` parameter to required for `Actor` data request methods
```

**指南**：

- 以 "Removed"、"Renamed" 或 "Changed" 开头。
- 简要说明迁移路径。

### 安全

安全加固和修复，防止崩溃、未定义行为或数据损坏。
包括从内部改进中提升的重要加固改进。

**格式**：

```markdown
- Fixed non-executable stack for Cython extensions to support hardened Linux systems
- Fixed divide-by-zero and overflow bugs in model crate that could cause crashes
- Fixed core arithmetic operations to reject NaN/Infinity values and improve overflow handling
```

**指南**：

- 包括溢出/下溢修复、内存安全改进、FFI 防护、数据完整性修复。
- 关注用户影响：可能会发生什么。
- 排除例行的依赖更新、小型加固或仅测试的修复。
- 如果某次发布没有安全条目，则完全省略此章节。

### 修复

改进正确性但不属于安全问题的 bug 修复。

**格式**：

```markdown
- Fixed reduce-only order panic when quantity exceeds position
- Fixed Binance order status parsing for external orders (#3006), thanks for reporting @bmlquant
```

**指南**：

- 以 "Fixed" 开头。

### 内部改进

实现细节和基础设施变更。

**格式**：

```markdown
- Added ARM64 support to Docker builds
- Ported `PortfolioAnalyzer` to Rust
- Improved clock and timer thread safety
- Upgraded Rust (MSRV) to 1.90.0
- Upgraded `pyo3` crates to v0.26.0
```

**指南**：

- 使用 "Added"、"Implemented"、"Improved"、"Optimized"、"Upgraded"、"Refined"、"Standardized"。
- 依赖升级需包含版本号。

### 文档更新

指南和示例的变更。

**格式**：

```markdown
- Added rate limit tables with links to official docs
- Improved dark and light themes for readability
- Fixed broken links
```

### 弃用

标记为将移除的功能。

**格式**：

```markdown
- Deprecated `convert_quote_qty_to_base`; disable (`False`) to maintain consistent behaviour. Will be removed in future version
```

**指南**：

- 说明迁移路径并提供替代方案。

## 归属

- 致谢外部贡献者：`thanks @username` 或 `thanks for reporting @username`。
- 社区贡献和复杂功能包含 issue/PR 编号：`(#1234)`。

## 风格

- 使用句首大写（仅首字母大写）。
- 不以句号结尾。
- 代码元素使用反引号。
- 关注**变更了什么**，而非如何变更。

**要具体**：

```markdown
不推荐：Improved Binance adapter
推荐：Improved Binance fill handling when instrument not cached
```

## 安全分类

如果变更涉及以下内容，应归入安全章节：

- 内存安全（威胁稳定性的溢出、下溢、除零）。
- 未定义行为或可能损坏状态的崩溃。
- 数据完整性（NaN/Infinity 传播、导致损坏的竞态条件）。
- 防止注入或利用的输入验证（SQL 注入、命令注入、路径遍历）。
- 构建加固（不可执行栈、FFI 防护）。
- 用户应了解的重要加固措施。

否则归入修复（用于逻辑 bug 和 panic）或内部改进（用于小型加固）。

注意：普通的逻辑 panic 归入修复，除非它们威胁系统稳定性或数据损坏。

## 示例

**安全**（可能导致崩溃/损坏）：

```markdown
- Fixed divide-by-zero in margin calculations that could crash the engine
- Fixed non-executable stack for Cython extensions to support hardened systems
```

**修复**（不正确但安全）：

```markdown
- Fixed Binance order status parsing for external orders
- Fixed position purge logic to prevent purging re-opened position
```

**功能增强**（面向用户）：

```markdown
- Added BitMEX conditional orders support
```

**内部改进**（实现层面）：

```markdown
- Implemented BitMEX ping/pong handling
```

## 发布工作流

发布后：

1. 在 `RELEASES.md` 中更新已发布版本的章节，填入实际发布日期。
2. 在已完成的发布版本下方添加水平分隔线 `---`。
3. 复制下面的模板粘贴到 `RELEASES.md` 顶部，用于下一个版本。
4. 将 `<VERSION>` 更新为下一个版本号。
5. 随着开发推进，向各章节添加条目。

## 发布说明模板

```markdown
# NautilusTrader <VERSION> Beta

Released on TBD (UTC).

### Enhancements

### Breaking Changes

### Security

### Fixes

### Internal Improvements

### Documentation Updates

### Deprecations

---
```
