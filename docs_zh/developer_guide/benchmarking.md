# 基准测试

本指南介绍 NautilusTrader 如何衡量 Rust 性能（performance），何时使用各种工具，
以及添加新基准测试时应遵循的约定。

---

## 工具概览

NautilusTrader 依赖**两个互补的基准测试框架**：

| 框架 | 简介 | 测量内容 | 适用场景 |
|------|------|----------|----------|
| [**Criterion**](https://docs.rs/criterion/latest/criterion/) | 统计型基准测试工具，生成详细的 HTML 报告并执行异常值检测。 | 带置信区间的实际运行时间。 | 端到端场景、大于约 100 ns 的测试、可视化比较。 |
| [**iai**](https://docs.rs/iai/latest/iai/) | 确定性微基准测试工具，通过硬件计数器统计已退役的 CPU 指令数。 | 精确指令计数（无噪声）。 | 超快函数、CI 中通过指令差异进行门控。 |

大多数热路径代码都能从**两种**测量方式中获益。

:::note
iai 是确定性的（不受系统噪声影响），但结果是机器特定的。请将其用于 CI 内的回归检测，而非跨机器比较。
:::

---

## 目录布局

每个 crate 将其性能测试放在本地的 `benches/` 文件夹中：

```text
crates/<crate_name>/
└── benches/
    ├── foo_criterion.rs   # Criterion 组
    └── foo_iai.rs         # iai 微基准测试
```

`Cargo.toml` 必须显式列出每个基准测试，这样 `cargo bench` 才能发现它们：

```toml
[[bench]]
name = "foo_criterion"
path = "benches/foo_criterion.rs"
harness = false

[[bench]]
name = "foo_iai"
path = "benches/foo_iai.rs"
harness = false
```

---

## 编写 Criterion 基准测试

1. 在计时循环（`b.iter`）**之外**完成所有耗时的初始化工作。
2. 用 `black_box` 包装输入/输出以防止优化器移除计算。
3. 使用 `benchmark_group!` 对相关用例进行分组，并在默认值不理想时设置 `throughput` 或
   `sample_size`。

```rust
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_my_algo(c: &mut Criterion) {
    let data = prepare_data(); // 耗时的初始化只执行一次

    c.bench_function("my_algo", |b| {
        b.iter(|| my_algo(black_box(&data)));
    });
}

criterion_group!(benches, bench_my_algo);
criterion_main!(benches);
```

---

## 编写 iai 基准测试

`iai` 要求函数**不带参数**并返回一个值（该值可以忽略）。尽量保持函数足够小，
使测量到的指令计数有意义。

```rust
use std::hint::black_box;

fn bench_add() -> i64 {
    let a = black_box(123);
    let b = black_box(456);
    a + b
}

iai::main!(bench_add);
```

---

## 本地运行基准测试

- **单个 crate**：`cargo bench -p nautilus-core`。
- **单个基准测试模块**：`cargo bench -p nautilus-core --bench time`。
- **CI 性能基准测试**：`make cargo-ci-benches`（逐个运行 CI 性能工作流中包含的 crate，
  以避免混合 panic 策略导致的链接器问题）。

Criterion 将 HTML 报告写入 `target/criterion/`；在浏览器中打开 `target/criterion/report/index.html` 即可查看。

### 生成火焰图

`cargo-flamegraph` 允许你查看单个基准测试的采样调用栈分析。在 Linux 上使用 `perf`，在 macOS 上使用 `DTrace`。

1. 每台机器安装一次 `cargo-flamegraph`（它会自动安装 `cargo flamegraph` 子命令）。

   ```bash
   cargo install flamegraph
   ```

2. 使用包含丰富符号信息的 `bench` profile 运行特定基准测试。

   ```bash
   # 示例：nautilus-common 中的 matching 基准测试
   cargo flamegraph --bench matching -p nautilus-common --profile bench
   ```

3. 在浏览器中打开生成的 `flamegraph.svg` 并放大热路径。

#### Linux

在 Linux 上，`perf` 必须可用。在 Debian/Ubuntu 上，可以通过以下方式安装：

```bash
sudo apt install linux-tools-common linux-tools-$(uname -r)
```

如果看到关于 `perf_event_paranoid` 的错误，需要放宽当前会话的内核 perf 限制（需要 root 权限）：

```bash
sudo sh -c 'echo 1 > /proc/sys/kernel/perf_event_paranoid'
```

值 `1` 通常就足够了；如需恢复默认值 `2` 或使更改永久生效，
可通过 `/etc/sysctl.conf` 进行设置。

#### macOS

在 macOS 上，`DTrace` 需要 root 权限，因此必须使用 `sudo` 运行 `cargo flamegraph`。

:::warning
使用 `sudo` 运行会在 `target/` 中创建 root 所有的文件，导致后续 `cargo` 命令出现权限错误。你可能需要手动删除 root 所有的文件或运行 `sudo cargo clean`。
:::

```bash
sudo cargo flamegraph --bench matching -p nautilus-common --profile bench
```

因为 `[profile.bench]` 保留了完整的调试符号，所以 SVG 会显示可读的函数名，
而不会使生产二进制文件膨胀（生产构建仍然使用 `panic = "abort"` 并通过 `[profile.release]` 构建）。

> **注意** 基准测试二进制文件使用工作区 `Cargo.toml` 中定义的自定义 `[profile.bench]` 编译。
> 该 profile 继承自 `release-debugging`，在保留完全优化的同时保留调试符号，
> 使 `cargo flamegraph` 或 `perf` 等工具能够生成人类可读的调用栈信息。

---

## 模板

可直接复制的起始文件位于 [`docs/dev_templates/`](../dev_templates/)：

- **Criterion**：[`criterion_template.rs`](../dev_templates/criterion_template.rs)
- **iai**：[`iai_template.rs`](../dev_templates/iai_template.rs)

将模板复制到 `benches/`，调整导入和名称，即可开始测量！
