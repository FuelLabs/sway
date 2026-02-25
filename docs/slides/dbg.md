---
layout: cover
marp: true
---

<!-- markdownlint-disable -->
# Sway `__dbg`

- What it does?
- How it works (High Level)
- How it works (Low Level)
- How to customize the `Debug` trait
- Debug vs Release

---
# What it does

`__dbg(...)` is the Sway version of Rust `dbg!(...)`. It prints the current line; it pretty prints its argument; it allows customization by the `Debug` trait; the compiler auto implement the `Debug` trait for all types; and in the end it returns its argument, allowing it to be used anywhere inside an expression.

```
> forc test --path sway-lib-std ok_all_primitives_must_be_debug --logs
[src/debug.sw:2679:13] = true
[src/debug.sw:2680:13] = false
[src/debug.sw:2691:13] = 0
[src/debug.sw:2692:13] = 18446744073709551615
[src/debug.sw:2697:13] = 0x0
[src/debug.sw:2698:13] = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
[src/debug.sw:2701:13] = "A"
[src/debug.sw:2703:13] = ("A", 0)
[src/debug.sw:2705:13] = [0, 1]
```

---
# How it works (High Level)

```rust
let a = __dbg(1u8);
```

is desugared into:

```rust
let a = {
    let mut f = Formatter { };
    f.print_str("[{current_file}:{current_line}:{current_col}] = ");
    let arg = arg;
    arg.fmt(f);
    arg
};
```

---

And we can call `fmt` on a `u8`  because of this `impl`.

```rust
pub trait Debug {
    fn fmt(self, ref mut f: Formatter);
}

impl Debug for u8 {
    fn fmt(self, ref mut f: Formatter) {
        f.print_u8(self);
    }
}
```

All other primitive `impls` live at `sway-lib-std/src/debug.sw`.

---
# How it works (Low Level)

Inside `Formatter`, we transform the number into a string; and call `syscall_write`.

```rust
pub fn print_u8(self, value: u8) {
        let mut value = value;
        let mut digits = [48u8; 64];
        let mut i = 63;
        while true {
            digits[i] = (value % 10) + 48; // ascii zero = 48
            value = value / 10;
            if value == 0 {
                break;
            }
            i -= 1;
        }
        syscall_write(STDERR, __addr_of(digits).add::<u8>(i), 64 - i);
    }
```

---

And `syscall_write` is very similar to `write(2)` (https://man7.org/linux/man-pages/man2/write.2.html), but instead of calling an actual `syscall`, we use `ecal` with the first argument as 1000.

We reserved [0, 1000), to the VM team; [1000, 2000) to the compiler team; And [2000, 3000) to the tooling team. So we can avoid a program having different behavior depending on the interpreter configuration.

So far, only 1000 is used.

```rust
// ssize_t write(int fd, const void buf[.count], size_t count);
fn syscall_write(fd: u64, buf: raw_ptr, count: u64) {
    asm(id: 1000, fd: fd, buf: buf, count: count) {
        ecal id fd buf count;
    }
}
```

---

And inside the interpreter, this `ecal` calls:

```rust
impl EcalHandler for EcalSyscallHandler {
    fn ecal<M: fuel_vm::prelude::Memory, S, Tx>(
        vm: &mut Interpreter<M, S, Tx, Self>, a: RegId, b: RegId, c: RegId, d: RegId,
    ) -> fuel_vm::error::SimpleResult<()>
    {
        let regs = vm.registers();
        let syscall = match regs[a.to_u8() as usize] {
            1000 => { ...  }
            _ => { ... }
        };
        Ok(())
    }
}
```
Real implementation at `forc-test/src/ecal.rs`

---

This means that output can vary by the interpreter configuration. The output shown below is from `forc-test`. Others interpreters can have different output, or none at all, as this is the default behavior.

```
> forc test --path sway-lib-std ok_all_primitives_must_be_debug --logs
[src/debug.sw:2679:13] = true
[src/debug.sw:2680:13] = false
[src/debug.sw:2691:13] = 0
[src/debug.sw:2692:13] = 18446744073709551615
[src/debug.sw:2697:13] = 0x0
[src/debug.sw:2698:13] = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
[src/debug.sw:2701:13] = "A"
[src/debug.sw:2703:13] = ("A", 0)
[src/debug.sw:2705:13] = [0, 1]
```

---
# How to customize the `Debug` trait

The compiler already generates `Debug`  impl for all `structs` and `enums`, but this implementation can be customized like this:

```rust
impl<T> Debug for Vec<T> where T: Debug
{
    fn fmt(self, ref mut f: Formatter) {
        let mut l = f.debug_list();
        for elem in self.iter() {
            let _ = l.entry(elem);
        }
        l.finish();
    }
}
```

---

This is the default implementation for the `Vec` struct;

```
[src/vec.sw:918:13] = Vec { buf: RawVec { ptr: 67108857, cap: 4 }, len: 3 }
```

versus the customized:

```
[src/vec.sw:931:13] = [1, 2, 3]
```

Different from `AbiEncode`, `Debug` auto implementation works for any primitive data type. Including: `raw_ptr`, `raw_slice`, `refs` etc... So the compiler will generate auto implementation for any type, unless a custom implementation is found.

---
# Debug vs Release

`__dbg` will never have any use when running on chain. The VM will never implement the `EcalHandler` as we saw above.
So deploying binaries with it, would be wasteful.
 
For this reason we completely strip everything when compiling in `--release` mode. So in release:

```rust
let a = __dbg(1u8);
```

desugars into 

```rust
let a = 1u8;
```

---

But, if there is a strange behavior on release one can really force the compiler using:

```toml
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
license = "Apache-2.0"
entry = "main.sw"
name = "some-project"
force-dbg-in-release = true
```

---

end