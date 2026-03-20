# Trivially Encodable & Decodable Types

When a contract calls another contract, all arguments are **encoded** just before the call is actually executed, 
and the callee **decodes** these arguments right before the target method starts.
This adds a small but non‑negligible gas cost, from hundreds to thousands of gas depending on the complexity of the arguments.

The Sway compiler mitigates this overhead for a subset of types that can be **trivially encoded** and/or **trivially decoded** –
that is, types that their *runtime representation*, how the type bytes are laid out inside the VM, is *identical* to their *encoded representation*,
how their bytes are laid out in the encoded buffer.

For such types the compiler can skip the encoding/decoding process entirely, saving gas and simplifying the generated code.

> **Trivial encoding** – encoding is replaced with a simple "transmute".
> **Trivial decoding** – encoding is replaced with a simple "transmute".

The compiler can skip each individually, but the whole gain comes only when both are skipped together.

## Checking Triviality

Each struct that should be treated as trivially encodable/decodable can be annotated with the `#[trivial]` attribute:

```sway
#[trivial(encode = "require", decode = "require")]
pub struct SomeArgument {
    a: bool,
    b: SomeEnum,
}
```

- `encode = "require"` – the compiler will check if the type is trivially encodable; if not, the build fails.
- `decode = "require"` – similarly for decoding.

Possible values are:

- required: compiler will check and error if the check fails;
- optional: compiler will only warn non-compliances;
- any: nothing will be checked.

This attributed can be used directly on types, but also on entry points such as "main" function for scripts and predicates; and contract methods for contracts.

## Which Types Are Trivial?

| Type | Trivially Encodable | Trivially Decodable | Notes |
|------|---------------------|---------------------|-------|
| `bool` | ✅ | ❌ | `bool` encodes to a single byte (`0` or `1`), but decoding must validate that the byte is a legal value. |
| `u8`, `u64`, `u256`, `b256` | ✅ | ✅ |  |
| `u16`, `u32` | ❌ | ❌ | Their runtime representation is actually a `u64` |
| Structs | ✅ If all their members are trivial | ✅ If all their member are trivial | Recursively evaluated. |
| Enums | ✅ If all variants are trivial | ❌ | Enums have an `u64` discriminant that cannot be trivially decodable. |
| Arrays | ✅ If the item type is trivial | ✅ if the item type is trivial |
| String Arrays | ✅ See * | ✅ See * |  |
| Vec, Dictionary, String, etc. | ❌ | ❌ | Data Structures are never trivial |

* Only when the feature "str_array_no_padding" is turned on. When the feature toggle is off, only string arrays that its length is multiple of 8.

### Why `bool` and `enum` are not trivially decodable

Probably the most surprising non trivial base data type is `bool`. Mainly because `bool` is obviously trivially encodable. But there is no guarantee
that buffer does not have a value like `2`, that being "transmuted" into a bool would be allow its runtime representation to be `2`, which is **undefined behaviour**.

The same limitation applies to enums. Enums are implemented as "tagged unios" which means that their runtime representation has a discriminant value as `u64`. There
is no guarantee that the buffer  would have a valid value for its discriminant.

---

## 3. Work‑arounds for Non‑trivial Types

If you need to expose a `bool` or an enum as a public argument, you can either:

1. **Manual validation** – expose a raw `u64` (or `u8`) and check its value in the callee.
   ```sway
   #[trivial(encode = "require", decode = "require")]
   pub struct Flag(u8);  // manually validate that value <= 1
   ```

2. **Custom wrappers** – Sway ships with `TrivialBool` and `TrivialEnum<T>` that enforce the bounds at compile time.

   ```sway
   use sway::primitive::TrivialBool;
   use sway::primitive::TrivialEnum;

   #[trivial(encode = "require", decode = "require")]
   pub struct SomeArgument {
       a: TrivialBool,
       b: TrivialEnum<SomeEnum>,
   }
   ```

   These wrappers automatically provide the guard checks and still let the compiler treat them as trivial.
   Their usage is veryy similar to `Option<bool>`.

   ```sway
   let a: bool = some_argument.a.unwrap();
   let b: SomeEnum = some_argument.b.unwrap();
   ```
