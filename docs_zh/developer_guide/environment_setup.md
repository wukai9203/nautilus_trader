# 环境搭建

开发时建议使用 PyCharm *Professional* 版 IDE，因为它能解析 Cython 语法。或者也可以使用安装了 Cython 扩展的 Visual Studio Code。

[uv](https://docs.astral.sh/uv) 是管理所有 Python 虚拟环境和依赖（dependency）的首选工具。

[pre-commit](https://pre-commit.com/) 用于在提交时自动运行各种检查、自动格式化和代码检查（linting）工具。

NautilusTrader 越来越多地使用 [Rust](https://www.rust-lang.org)，因此你的系统上也需要安装 Rust
（[安装指南](https://www.rust-lang.org/tools/install)）。

[Cap'n Proto](https://capnproto.org/) 是序列化 schema 编译所必需的。所需版本在仓库根目录的 `capnp-version` 文件中指定。Ubuntu 的默认包通常版本过旧，你可能需要从源码安装（见下文）。

:::info
NautilusTrader *必须*能在 **Linux、macOS 和 Windows** 上编译和运行。请注意可移植性
（使用 `std::path::Path`，避免 shell 脚本中的 Bash 特有用法等）。
:::

## 搭建步骤

以下步骤适用于类 UNIX 系统，只需执行一次。

1. 按照[安装指南](../getting_started/installation.md)搭建项目，最后一步命令改为安装开发和测试依赖：

```bash
uv sync --active --all-groups --all-extras
```

或

```bash
make install
```

如果你需要频繁开发和迭代，以调试模式（debug mode）编译通常就足够了，而且比完全优化构建*快得多*。
以调试模式安装，使用：

```bash
make install-debug
```

2. 设置 pre-commit 钩子，之后提交时会自动运行：

```bash
pre-commit install
```

在发起 Pull Request 之前，先在本地运行格式化和代码检查套件，以确保 CI 一次通过：

```bash
make format
make pre-commit
```

确保 Rust 编译器报告**零错误** -- 构建失败会拖慢所有人的进度。

3. **Rust/PyO3 所需（Linux 和 macOS）**：在 Linux 或 macOS 上使用 `uv` 安装的 Python 时，设置以下环境变量：

```bash
# 添加到你的 shell 配置文件（如 ~/.zshrc 或 ~/.bashrc）

# 仅 Linux：设置 Python 解释器的库路径
export LD_LIBRARY_PATH="$(python -c 'import sys; print(sys.base_prefix)')/lib:$LD_LIBRARY_PATH"

# 设置 PyO3 使用的 Python 可执行文件路径
export PYO3_PYTHON=$(pwd)/.venv/bin/python

# 设置 Python home 路径（Rust 测试所需）
export PYTHONHOME=$(python -c "import sys; print(sys.base_prefix)")
```

:::note
`LD_LIBRARY_PATH` 导出仅适用于 Linux，macOS 或 Windows 上不需要。

- `PYO3_PYTHON` 告诉 PyO3 使用哪个 Python 解释器，减少不必要的重新编译。
- `PYTHONHOME` 在使用 `uv` 安装的 Python 运行 `make cargo-test` 时是必需的。
  没有它，依赖 PyO3 的测试可能无法找到 Python 运行时。

:::

验证你的环境配置是否正确：

```bash
python -c "import sys; print('Python:', sys.executable, sys.version)"
echo "PYO3_PYTHON: $PYO3_PYTHON"
echo "PYTHONHOME: $PYTHONHOME"
```

## 构建

对 `.rs`、`.pyx` 或 `.pxd` 文件做出任何更改后，可以通过以下方式重新编译：

```bash
uv run --no-sync python build.py
```

或

```bash
make build
```

如果你需要频繁开发和迭代，以调试模式编译通常就足够了，而且比完全优化构建*快得多*。
以调试模式编译，使用：

```bash
make build-debug
```

## Cap'n Proto

[Cap'n Proto](https://capnproto.org/) 是序列化 schema 编译所必需的。
所需版本在仓库根目录的 `capnp-version` 文件中定义。

在 **macOS** 上，通过 Homebrew 安装：

```bash
brew install capnp
```

验证安装的版本是否与 `capnp-version` 一致。如果 Homebrew 提供的版本较旧，
请按照下面的 Linux 说明从源码安装。

在 **Ubuntu/Linux** 上，默认包通常版本过旧。从源码安装：

```bash
CAPNP_VERSION=$(cat capnp-version)
cd ~
wget https://capnproto.org/capnproto-c++-${CAPNP_VERSION}.tar.gz
tar xzf capnproto-c++-${CAPNP_VERSION}.tar.gz
cd capnproto-c++-${CAPNP_VERSION}
./configure
make -j$(nproc)
sudo make install
sudo ldconfig
```

验证安装：

```bash
capnp --version
```

在 **Windows** 上，通过 Chocolatey 安装：

```bash
choco install capnproto
```

验证安装的版本是否与 `capnp-version` 一致。如果 Chocolatey 提供的版本较旧，
请参阅 [Cap'n Proto 安装指南](https://capnproto.org/install.html)了解其他安装方式。

## 加速构建

cranelift 后端可以显著减少开发、测试和 IDE 检查的构建时间。不过 cranelift 需要 nightly 工具链和额外配置。安装 nightly 工具链：

```
rustup install nightly
rustup override set nightly
rustup component add rust-analyzer # 安装 nightly LSP
rustup override set stable # 重置为 stable
```

在工作区 `Cargo.toml` 的 dev 和 testing 配置中激活 nightly 特性并使用 "cranelift" 后端。你可以使用 `git apply <patch>` 来应用下面的补丁，推送前使用 `git apply -R <patch>` 来撤销。

:::warning
不要提交这些更改。cranelift 补丁仅用于本地开发，推送后会导致 CI 失败。
:::

```
diff --git a/Cargo.toml b/Cargo.toml
index 62b78cd8d0..beb0800211 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -1,3 +1,6 @@
+# This line needs to come before anything else in Cargo.toml
+cargo-features = ["codegen-backend"]
+
 [workspace]
 resolver = "2"
 members = [
@@ -140,6 +143,7 @@ lto = false
 panic = "unwind"
 incremental = true
 codegen-units = 256
+codegen-backend = "cranelift"

 [profile.test]
 opt-level = 0
@@ -150,11 +154,13 @@ strip = false
 lto = false
 incremental = true
 codegen-units = 256
+codegen-backend = "cranelift"

 [profile.nextest]
 inherits = "test"
 debug = false # Improves compile times
 strip = "debuginfo" # Improves compile times
+codegen-backend = "cranelift"

 [profile.release]
 opt-level = 3
```

运行 `make build-debug` 等命令时传入 `RUSTUP_TOOLCHAIN=nightly`，并在所有 [rust analyzer 设置](#rust-analyzer-设置)中包含该变量，以加速构建和 IDE 检查。

## 服务

你可以使用 `.docker` 目录中的 `docker-compose.yml` 文件来启动 Nautilus 开发环境。这将启动以下服务：

```bash
docker-compose up -d
```

如果你只想运行特定服务（例如 `postgres`），可以使用以下命令启动：

```bash
docker-compose up -d postgres
```

使用的服务包括：

- `postgres`：Postgres 数据库，root 用户为 `POSTRES_USER`（默认 `postgres`），`POSTGRES_PASSWORD`（默认 `pass`），`POSTGRES_DB`（默认 `postgres`）。
- `redis`：Redis 服务器。
- `pgadmin`：用于数据库管理和维护的 PgAdmin4。

:::info
请仅将此作为开发环境使用。生产环境请使用更正式、更安全的配置。
:::

服务启动后，你需要使用 `psql` 命令行工具登录并创建 `nautilus` Postgres 数据库。
运行以下命令，并输入 Docker 服务配置中的 `POSTGRES_PASSWORD`：

```bash
psql -h localhost -p 5432 -U postgres
```

以 `postgres` 管理员身份登录后，运行 `CREATE DATABASE` 命令创建目标数据库（这里使用 `nautilus`）：

```
psql (16.2, server 15.2 (Debian 15.2-1.pgdg110+1))
Type "help" for help.

postgres=# CREATE DATABASE nautilus;
CREATE DATABASE

```

## Nautilus CLI 开发者指南

## 简介

Nautilus CLI 是一个用于与 NautilusTrader 生态系统交互的命令行界面工具。
它提供了管理 PostgreSQL 数据库和处理各种交易操作的命令。

:::warning
在使用 GNOME 桌面的 Linux 系统上，`nautilus` 命令通常指向 GNOME 文件管理器（`/usr/bin/nautilus`）。
安装 NautilusTrader CLI 后，你可能需要确保 Cargo 二进制文件优先，方法如下：

- 在 shell 配置中添加别名：`alias nautilus="$HOME/.cargo/bin/nautilus"`
- 使用完整路径：`~/.cargo/bin/nautilus`
- 确保 `~/.cargo/bin` 在 `PATH` 中位于 `/usr/bin` 之前

:::

:::note
Nautilus CLI 命令仅在类 UNIX 系统上受支持。
:::

## 安装

你可以使用下面的 Makefile 目标安装 Nautilus CLI，底层使用 `cargo install`。
这会将 nautilus 二进制文件放入系统的 PATH 中，前提是 Rust 的 `cargo` 已正确配置。

```bash
make install-cli
```

## 命令

运行 `nautilus --help` 可以查看 CLI 结构和可用的命令组：

### 数据库

这些命令用于引导（bootstrap）PostgreSQL 数据库。
使用时需要提供正确的连接配置，
可以通过命令行参数或放在根目录或当前工作目录的 `.env` 文件中。

- `--host` 或 `POSTGRES_HOST` 数据库主机
- `--port` 或 `POSTGRES_PORT` 数据库端口
- `--user` 或 `POSTGRES_USERNAME` root 管理员（通常是 postgres 用户）
- `--password` 或 `POSTGRES_PASSWORD` root 管理员密码
- `--database` 或 `POSTGRES_DATABASE` 数据库**名称和新用户**，该用户拥有该数据库的权限
    （例如，如果你提供 `nautilus` 作为值，将创建一个名为 nautilus 的新用户，密码来自 `POSTGRES_PASSWORD`，并以该用户为所有者引导 `nautilus` 数据库）。

`.env` 文件示例：

```
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USERNAME=postgres
POSTGRES_PASSWORD=pass
POSTGRES_DATABASE=nautilus
```

命令列表：

1. `nautilus database init`：引导 schema、角色以及 `schema` 根目录中的所有 sql 文件（如 `tables.sql`）。
2. `nautilus database drop`：删除目标 Postgres 数据库中的所有表、角色和数据。

## Rust analyzer 设置

Rust analyzer 是一个流行的 Rust 语言服务器，集成了许多 IDE。建议将 rust analyzer 配置为与 `make build-debug` 相同的环境变量以加快编译速度。下面提供了 VSCode 和 Astro Nvim 的经过测试的配置。更多信息请参阅 [PR](https://github.com/nautechsystems/nautilus_trader/pull/2524) 或 rust analyzer [配置文档](https://rust-analyzer.github.io/book/configuration.html)。

### VSCode

你可以将以下设置添加到 VSCode 的 `settings.json` 文件中：

```
    "rust-analyzer.restartServerOnConfigChange": true,
    "rust-analyzer.linkedProjects": [
        "Cargo.toml"
    ],
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.check.workspace": false,
    "rust-analyzer.check.extraEnv": {
        "VIRTUAL_ENV": "<path-to-your-virtual-environment>/.venv",
        "CC": "clang",
        "CXX": "clang++"
    },
    "rust-analyzer.cargo.extraEnv": {
        "VIRTUAL_ENV": "<path-to-your-virtual-environment>/.venv",
        "CC": "clang",
        "CXX": "clang++"
    },
    "rust-analyzer.runnables.extraEnv": {
        "VIRTUAL_ENV": "<path-to-your-virtual-environment>/.venv",
        "CC": "clang",
        "CXX": "clang++"
    },
    "rust-analyzer.check.features": "all",
    "rust-analyzer.testExplorer": true
```

### Astro Nvim (Neovim + AstroLSP)

你可以将以下内容添加到 astro lsp 配置文件中：

```
    config = {
      rust_analyzer = {
        settings = {
          ["rust-analyzer"] = {
            restartServerOnConfigChange = true,
            linkedProjects = { "Cargo.toml" },
            cargo = {
              features = "all",
              extraEnv = {
                VIRTUAL_ENV = "<path-to-your-virtual-environment>/.venv",
                CC = "clang",
                CXX = "clang++",
              },
            },
            check = {
              workspace = false,
              command = "check",
              features = "all",
              extraEnv = {
                VIRTUAL_ENV = "<path-to-your-virtual-environment>/.venv",
                CC = "clang",
                CXX = "clang++",
              },
            },
            runnables = {
              extraEnv = {
                VIRTUAL_ENV = "<path-to-your-virtual-environment>/.venv",
                CC = "clang",
                CXX = "clang++",
              },
            },
            testExplorer = true,
          },
        },
      },
```
