# 测试

我们的自动化测试（automated tests）作为交易平台的可执行规范。
一套健康的测试套件记录了预期行为，让贡献者有信心进行重构，并在问题到达生产环境之前捕获回归缺陷。
测试同时也是活的示例，有助于阐明复杂流程并提供快速的 CI 反馈，使问题尽早浮现。

测试套件涵盖以下类别：

- 单元测试（Unit tests）
- 集成测试（Integration tests）
- 验收测试（Acceptance tests）
- 性能测试（Performance tests）
- 内存泄漏测试（Memory leak tests）

性能测试有助于持续改进性能关键组件。

使用 [pytest](https://docs.pytest.org) 运行测试，它是我们的主要测试运行器。
使用参数化测试和 fixture（如 `@pytest.mark.parametrize`）以避免重复代码并提升清晰度。

## 运行测试

### Python 测试

从仓库根目录：

```bash
make pytest
# 或
uv run --active --no-sync pytest --new-first --failed-first
# 或
pytest
```

运行性能测试：

```bash
make test-performance
# 或
uv run --active --no-sync pytest tests/performance_tests --benchmark-disable-gc --codspeed
```

`--benchmark-disable-gc` 标志可防止垃圾回收影响结果。性能测试应单独运行（不与单元测试混合），以避免干扰。

### Rust 测试

```bash
make cargo-test
# 或
cargo nextest run --workspace --features "python,ffi,high-precision,defi" --cargo-profile nextest
```

#### 使用可选特性进行测试

使用 `EXTRA_FEATURES` 来包含可选特性，如 `capnp` 或 `hypersync`：

```bash
# 使用 capnp 特性测试
make cargo-test EXTRA_FEATURES="capnp"

# 使用多个特性测试
make cargo-test EXTRA_FEATURES="capnp hypersync"

# hypersync 的旧版简写
make cargo-test HYPERSYNC=true

# 使用特性测试特定 crate
make cargo-test-crate-nautilus-serialization FEATURES="capnp"
```

### IDE 集成

- **PyCharm**：右键点击测试文件夹或文件 -> "Run pytest"。
- **VS Code**：使用 Python Test Explorer 扩展。

## 测试风格

- 以测试所验证的内容命名测试函数；不需要在名称中编码预期的断言。
- 当文档字符串有助于阐明设置、场景或预期时，添加文档字符串。
- Python 测试优先使用 pytest 风格的自由函数，而非带有 setup 方法的测试类。
- **合并断言**：尽可能先执行所有 setup/act 步骤，然后统一断言，避免 act-assert-act 反模式。
- 在测试中使用 `unwrap`、`expect` 或直接的 `panic!`/`assert` 调用；此处清晰和简洁比防御性错误处理更重要。

有关 Rust 特定的测试约定（模块结构、`#[rstest]`、参数化），请参阅 [Rust 指南](rust.md#testing-conventions)。

## 等待异步效果

等待后台工作完成时，优先使用 `nautilus_trader.test_kit.functions` 中的轮询辅助函数 `await eventually(...)` 和 `nautilus_common::testing` 中的 `wait_until_async(...)`，而不是任意的 sleep。它们能更快地暴露失败，并减少 CI 中的不稳定性，因为它们会在条件满足时立即停止，或在超时时给出有用的错误。

## Mock

优先使用手写的 stub（返回固定值），而不是 mock 框架。仅在需要断言调用次数/参数或模拟复杂状态变化时使用 `MagicMock`。避免对被测试的对象本身进行 mock。

## 代码覆盖率

我们使用 `coverage` 生成覆盖率报告，并发布到 [codecov](https://about.codecov.io/)。

目标是在不牺牲适当错误处理或对架构造成"测试引发的损害"的前提下实现高覆盖率。

某些分支在不修改生产行为的情况下无法测试。
例如，防御性 if-else 块中的最终条件可能只在遇到意外值时才触发；保留这些检查以便未来的更改在需要时可以测试它们。

设计时的异常也可能不切实际地进行测试，因此 100% 覆盖率不是目标。

## 排除代码覆盖

我们使用 `pragma: no cover` 注释来[排除代码覆盖](https://coverage.readthedocs.io/en/coverage-4.3.3/excluding.html)，以避免冗余测试。
典型示例包括：

- 断言抽象方法在被调用时抛出 `NotImplementedError`。
- 断言 if-else 块中无法测试的最终条件检查（如上所述）。

此类测试维护成本高，因为它们必须跟踪重构但提供的价值很小。
确保抽象方法的具体实现保持完全覆盖。
当 `pragma: no cover` 不再适用时移除它，并将其使用限制在上述情况。

## 调试 Rust 测试

使用默认测试配置来调试 Rust 测试。

要使用调试符号运行完整套件以便后续分析，运行 `make cargo-test-debug` 而非 `make cargo-test`。

在 IntelliJ IDEA 中，为参数化 `#[rstest]` 用例调整运行配置，使其读取 `test --package nautilus-model --lib data::bar::tests::test_get_time_bar_start::case_1`
（移除 `-- --exact` 并追加 `::case_n`，其中 `n` 从 1 开始）。此解决方法与[此处](https://github.com/rust-lang/rust-analyzer/issues/8964#issuecomment-871592851)说明的行为一致。

在 VS Code 中，你可以直接选择特定的测试用例进行调试。

## Python + Rust 混合调试

此工作流允许你在 VS Code 的 Jupyter notebook 中同时调试 Python 和 Rust 代码。

### 设置

安装以下 VS Code 扩展：Rust Analyzer、CodeLLDB、Python、Jupyter。

### 步骤 0：以调试符号编译 `nautilus_trader`

   ```bash
   cd nautilus_trader && make build-debug-pyo3
   ```

### 步骤 1：设置调试配置

```python
from nautilus_trader.test_kit.debug_helpers import setup_debugging

setup_debugging()
```

此命令会创建所需的 VS Code 调试配置，并为 Python 调试器启动 `debugpy` 服务器。

默认情况下，`setup_debugging()` 期望 `.vscode` 文件夹位于 `nautilus_trader` 根目录的上一级。
如果你的工作区布局不同，请调整目标位置。

### 步骤 2：设置断点

- **Python 断点：** 在 VS Code 中的 Python 源文件中设置。
- **Rust 断点：** 在 VS Code 中的 Rust 源文件中设置。

### 步骤 3：启动混合调试

1. 在 VS Code 中选择 **"Debug Jupyter + Rust (Mixed)"** 配置。
2. 启动调试（F5）或点击绿色运行箭头。
3. Python 和 Rust 调试器都会附加到你的 Jupyter 会话。

### 步骤 4：执行代码

运行调用 Rust 函数的 Jupyter notebook 单元格。调试器会在 Python 和 Rust 代码的断点处停下。

### 可用配置

`setup_debugging()` 创建以下 VS Code 配置：

- **`Debug Jupyter + Rust (Mixed)`** - Jupyter notebook 的混合调试。
- **`Jupyter Mixed Debugging (Python)`** - notebook 的纯 Python 调试。
- **`Rust Debugger (for Jupyter debugging)`** - notebook 的纯 Rust 调试。

### 示例

打开并运行示例 notebook：`debug_mixed_jupyter.ipynb`。

### 参考

- [PyO3 调试](https://pyo3.rs/v0.25.1/debugging.html?highlight=deb#debugging-from-jupyter-notebooks)
