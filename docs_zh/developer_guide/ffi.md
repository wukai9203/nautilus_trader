# FFI 内存契约 (FFI Memory Contract)

NautilusTrader 暴露了若干 **C 兼容** 类型，以便 Cython 生成的 C 扩展或其他原生语言能够调用已编译的 Rust 代码。其中最重要的是 `CVec` —— 一个围绕 Rust `Vec<T>` 的*轻量*封装 (wrapper)，它通过 FFI (外部函数接口) 边界**按值传递**。

以下规则是*严格的*；违反这些规则会导致未定义行为 (undefined behaviour)（通常是双重释放或内存泄漏 (memory leak)）。

## FFI 边界的快速失败 panic

Rust 的 panic 绝不能跨 `extern "C"` 函数进行栈展开 (unwind)。向 C 或 Python 栈展开属于未定义行为，可能破坏外部调用栈或留下部分析构的资源。为了贯彻快速失败架构，我们用 `crate::ffi::abort_on_panic` 封装每个导出符号，该封装会执行函数体，并在发生 panic 时调用 `process::abort()`。panic 信息在终止前仍会被记录到日志，因此调试输出得以保留，同时避免了未定义行为。

添加新的 FFI 函数时，请在实现外层调用 `abort_on_panic(|| { … })`（或使用已封装好的辅助函数）以维持此保证。

## CVec 生命周期

| 步骤 | 所有者 | 操作 |
|-------|--------|------|
| **1** | Rust | 构建 (build) 一个 `Vec<T>` 并通过 `into()` 转换——这会*泄漏*该向量，并将原始分配的所有权转移给外部代码。 |
| **2** | 外部代码 (Python / Cython / C) | 在 `CVec` 值有效期间使用数据。**不要修改 `ptr`、`len`、`cap` 字段。** |
| **3** | 外部代码 | **恰好调用一次** Rust 导出的*类型特定*释放辅助函数（例如 `vec_drop_book_levels`、`vec_drop_book_orders`、`vec_time_event_handlers_drop`）。该辅助函数通过 `Vec::from_raw_parts` 重建原始 `Vec<T>` 并让其析构，从而释放内存。 |

:::warning
如果遗漏步骤 **3**，分配的内存将在进程的剩余生命周期内泄漏；如果步骤 **3** 被执行**两次**，程序将发生双重释放并很可能崩溃。
:::

## Python 侧创建的 Capsule

若干 Cython 辅助函数使用 `PyMem_Malloc` 分配临时 C 缓冲区，将其封装为 `CVec`，并将地址包装在 `PyCapsule` 中返回。**每个此类 capsule 在创建时都注册了析构函数** (`capsule_destructor` 或 `capsule_destructor_deltas`)，用于释放缓冲区和 `CVec`。因此调用者*不得*手动释放内存——否则会导致双重释放。

## Rust 侧创建的 Capsule *（PyO3 绑定 (binding)）*

当 Rust 代码将堆分配的值传递给 Python 时，**必须**使用 `PyCapsule::new_with_destructor`，以便 Python 知道在 capsule 不可达时如何释放该分配。闭包/析构函数负责重建原始的 `Box<T>` 或 `Vec<T>` 并让其析构。

```rust
use pyo3::types::PyCapsule;

Python::attach(|py| {
    // 在堆上分配值
    let my_data = Box::new(MyStruct::new());
    let ptr = Box::into_raw(my_data);

    // 将其移入 capsule 并注册释放内存的析构函数
    let capsule = PyCapsule::new_with_destructor(
        py,
        ptr,
        None,
        |ptr, _| {
            // 重建 Box 并让其析构，释放分配的内存
            let _ = unsafe { Box::from_raw(ptr) };
        },
    )
    .expect("capsule creation failed");

    // ... 将 `capsule` 传回 Python ...
});
```

**不要**使用 `PyCapsule::new(…, None)`；该变体*不会*注册析构函数，除非接收方手动提取并释放指针 (pointer)（而我们从不依赖这种做法），否则会泄漏内存。代码库已在所有位置更新以遵循此规则——新增的 FFI 模块必须遵循相同模式。

## 为何不再有通用的 `cvec_drop`

早期版本的代码库提供了一个通用的 `cvec_drop` 函数，它始终将缓冲区视为 `Vec<u8>`。对任何其他元素类型使用该函数会在释放时导致大小不匹配，从而破坏分配器的内部记录。由于该辅助函数在项目内部没有任何引用，已将其移除以避免意外误用。

应改为使用与你的元素类型对应的**类型特定**释放辅助函数（例如 `vec_drop_book_levels`、`vec_drop_book_orders`）。如果你的类型尚无对应的辅助函数，请参照 `crates/core/src/ffi/cvec.rs` 中的模式添加一个。

## 基于 Box 的 `*_API` 封装（拥有所有权的 Rust 对象）

当 Rust 核心需要将一个*复杂*值（例如 `OrderBook`、`SyntheticInstrument` 或 `TimeEventAccumulator`）传递给外部代码时，它会使用 `Box::new` 在堆上分配该值，并返回一个小型 `repr(C)` 封装，其唯一字段就是该 `Box`。

```rust
#[repr(C)]
pub struct OrderBook_API(Box<OrderBook>);

#[unsafe(no_mangle)]
pub extern "C" fn orderbook_new(id: InstrumentId, book_type: BookType) -> OrderBook_API {
    OrderBook_API(Box::new(OrderBook::new(id, book_type)))
}

#[unsafe(no_mangle)]
pub extern "C" fn orderbook_drop(book: OrderBook_API) {
    drop(book); // 释放堆分配的内存
}
```

因此，内存安全性 (memory safety) 要求如下：

1. 每个构造函数 (`*_new`) **必须**有一个对应的 `*_drop` 导出在其旁边。
2. 在堆分配之前验证参数，以便快速失败并避免分配无效对象。
3. *Python/Cython* 绑定必须保证 `*_drop` 被**恰好调用一次**。目前存在两种方式：

    - **新代码推荐方式**：将指针封装在通过 `PyCapsule::new_with_destructor` 创建的 `PyCapsule` 中，传入调用释放辅助函数的析构函数。

    - **遗留模式**（仅限 v1 Cython 模块）：在 Python 侧的 `__del__`/`__dealloc__` 中显式调用辅助函数：

      ```python
      cdef class OrderBook:
          cdef OrderBook_API _mem

          def __cinit__(self, ...):
              self._mem = orderbook_new(...)

          def __del__(self):
              if self._mem._0 != NULL:
                  orderbook_drop(self._mem)
      ```

无论使用哪种方式，请记住：**忘记调用 drop 会泄漏整个结构体**，而调用两次则会导致双重释放并崩溃。

新的 FFI 代码必须使用带析构函数的 `PyCapsule`，并遵循此模板，方可合并。
