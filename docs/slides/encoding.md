---
layout: cover
marp: true
---

<!-- markdownlint-disable -->
# Sway Encoding

---

- How it works for contracts?
  - Caller POV
  - Contract being called POV
- **Interlude**: Why even encode the data?
- Encoding for
    - scripts
    - predicates
    - receipts (logs)
    - configurables

---
# How it works? (caller POV)

At some point, someones calls a contract:

```rust
let contract = abi(TestContract, CONTRACT_ID);
contract.some_method(0);
```

Compiler will desugar this into

```rust
core::codec::contract_call(
    CONTRACT_ID,
    "some_method",
    (0,),
)
```

---

which will be expanded into:

```rust
let first_parameter = encode("some_method");
let second_parameter = encode((0,));
let params = encode((
    CONTRACT_ID,
    first_parameter.ptr(),
    second_parameter.ptr(),
));

let (ptr, len) = __contract_call(params.ptr(), coins, asset_id, gas);

let mut buffer = BufferReader::from_parts(ptr, len);
T::abi_decode(buffer)
```

---

and, in the end, will be compiled into a **fuelVM** `call` instruction.

```
      $hp
       │
       │                     ┌────────────────────────────┐
       ▼                     │                            │
       ┌──────────────┬──────┴──────┬──────────────┬──────▼────────┬───────────────┐
       │              │             │              │               │               │
HEAP   │  CONTRACT_ID │ method name │ method args  │ encoded bytes │ encoded bytes │
       │    32 bytes  │   param1    │   param2     │               │               │
       └──────▲───────┴─────────────┴──────┬───────┴───────────────┴───────▲───────┘
              │                            │                               │
              │                            └───────────────────────────────┘
              │
        ...   │
        call $ra:ptr $rb:u64 $rc:ptr $rd:u64
        ...           coins   │        gas
                              │
                             ┌▼─────────────┐
                             │              │
STACK  ....................  │   ASSET_ID   │
                             │    32 bytes  │
                             └──────────────┘
       ▲                                    ▲
       │                                    │
       │                                    │
     $ssp                                  $sp
```

---
# How it works? (contract being called POV)

The example above was calling a contract implemented like:

```rust
impl TestContract for Contract {
    fn some_method(qty: u64) {
        ...
    }
}
```

---

Compiler will desugar this into:

```rust
pub fn __entry() {
    let method_name = core::codec::decode_first_param::<str>();
    if method_name == "some_method" {
        let mut buffer = core::codec::BufferReader::from_second_parameter();
        let args: (u64,) = buffer.decode::<(u64,)>();
        let result: () = __contract_entry_some_method(args.0);
        let result: raw_slice = encode::<()>(result);
        __contract_ret(result.ptr(), result.len::<u8>());
    }
    __revert(123);
}
```

`__contract_entry_some_method` is the original `some_method` as is. 
`__contract_ret` is a special function that immediately returns the current context, that is why there is no `return` in the generated code.

---

This is the memory layout before the contract method is actually called

```
      $hp                                                                           
       |                                                                            
       |                     +----------------------------+                         
       v                     |                            |                         
       +--------------+------+------+--------------+------v--------+---------------+
       |              |             |              |               |               |
HEAP   |  CONTRACT_ID | method name | method args  | encoded bytes | encoded bytes |
       |    32 bytes  |   param1    |   param2     |               |               |
       +--------------+-------------+------+-------+------^--------+-------^--^----+
                                           |              |                |  |     
                                           +--------------+----------------+  |     
                                                          |                   |     
                                +-------------------------+                   |     
                                |                                             |     
                                |       +-------------------------------------+     
                                |       |                                           
        +--------------+--------+-------+--------+                                  
        |              | param1      param2      |                                  
STACK   |  ASSET_ID    |  @ 73 words  @ 74 words |                                  
        |    32 bytes  |  offset      offset     |                                  
        +--------------+-------------------------+                                  
                       ^   call frame metadata   ^                                  
                       |                         |                                  
                       |                         |                                  
                      $fp                    $ssp/$sp                               
```

---

# **Interlude**: Why even encode the data?

- All this begs the question... why?
    - Why all the extra cost of encoding and decoding?
    - why we don´t just pass all parameters directly?
- Answer:
    - ABI instability
    - Return Value Demotion
    - Aliasing

---
# Why? Avoiding `ABI instability`

APIs define the types that are exchanged:

```rust
abi MyContract {
    fn some_method(arg: Vec<u64>);
}
```

but they do not define how these types are passed around. In the example above, the only safe way to call a contract, would be to use the **exactly same version** of the `stdlib`  the contract implementation is using.

Why?

Any internal change to `Vec` could make calling a contract impossible.

--- 

Example: Suppose that caller and callee were compiled using `std-lib v1`.

```rust
struct Vec<T> {
    pointer: raw_ptr,
    cap: u64,
    len: u64,
}
```

But the contract implementation decide to update to `std-lib v2`, that for some reason was changed to:

```rust
struct Vec<T> {
    len: u64,
    pointer: raw_ptr,
    cap: u64,
}
```

---

Now caller will never be able to call the contract. Welcome to **ABI Hell**.


```
                                                         |                                                         
                        BEFORE                           |                          AFTER                          
                                                         |                                                         
           +-----> { 0x...., 16, 4 } <----+              |            +----->  { 0x...., 16, 4 } <---+             
           |                              |              |            |                              |             
           |                              |              |            |                              |             
{ pointer, capacity, len }    { pointer, capacity, len } | { pointer, capacity, len }    { len, pointer, capacity }
                                                         |                                                         
        What                           What              |         What                             What           
       CALLER                         CALLEE             |        CALLER                           CALLEE          
        sends                          sees              |         sends                            sees           
    Vec @ std-lib v1               Vec @ std-lib v1      |     Vec @ std-lib v1                  Vec @ std-lib v2  
```

Now imagine everything that can affect byte layouts: lib versions, compiler versions, compiler flags, compiler optimizations etc....

All this can break contract calls without anyone noticing. All this, actually, also breaks whoever is consuming these bytes: indexers, SDKs, receipts parsers etc...

---

So our encoding scheme avoid this instability being an issue by forcing types to specify how they are encoded, decoded:

```rust

impl<T> AbiEncode for Vec<T> where T: AbiEncode
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        ...
    }
}

impl<T> AbiDecode for Vec<T> where T: AbiDecode
{
    fn abi_decode(ref mut buffer: BufferReader) -> Vec<T> {
        ...
    }
}
```

And to improve dev experience, we automatically implement `AbiEncode`/`AbiDecode` for all types that do not contain pointers.

---
# Why? Issues with `Return Value Demotion`

Consider a simple contract method as:

```rust
impl TestContract for Contract {
    fn some_method() -> Vec<u64> {
        Vec::new()
    }
}
```

In reality this will be compiled into something like the code below. This is called "Return Value Demotion".

```rust
fn __contract_entry_some_method(return_value: &mut Vec<u64>) {
    *return_value = Vec::new();
}
```

---

The problems for contracts is that `return_value` would point to memory region of the callee. And contracts cannot write into it.

That would demand `return_value` pointing to somewhere allocated by the contract, and being copied over by the callee.

---
# Why? `Aliasing`

If we bypass encoding, and allow caller to pass pointers to contracts, that immediately creates a security issues, because malicious callers could craft aliased data structures and trick contracts.

For example:

```rust
impl TestContract for Contract {
    fn some_method(v: Vec<u64>) {
        let some_value1 = do_something(&v);
        // do some_thing that allocate memory
        let some_value2 = do_something(&v);
    }
}
```

---

A malicious caller can craft a `Vec` which its pointer points to the memory the contract uses.

In the case above, would be possible to have `some_value1` be different from `some_value2`, although this 100% not intuitive.

Allowing a caller to attack the contract somehow.

To avoid this, our encoding scheme, do not allow pointers, references etc... to be passed. You must always pass only the data.

---

# What we have left

- Encoding for
    - scripts
    - predicates
    - receipts (logs)
    - configurables

---
# Scripts and Predicates

`scripts` and `predicates` can have arguments on their `main` function.

```rust
fn main(v: u64) -> bool {
    ...
}
```

In both cases, the compiler will desugar into something like:

```rust
pub fn __entry() -> raw_slice {
    let args: (u64,) = decode_script_data::<(u64,)>(); // or decode_predicate_data
    let result: u64 = main(args.0); 
    encode::<u64>(result)
}
```

---
# Log

When the `std::log` is called we also encode its argument. So

```rust
    log(1);
```

will be desugared into

```rust
    __log(encode(1));
```

---
# Configurables

Configurables are a little bit more complex. What happens with configurables is that their initialization is evaluated at compile time.

```rust
configurable {
    SOMETHING: u64 = 1,
}
```

In the example above, it will evaluate to `1`. The compiler will then call `encode(1)` and append the result to the end of the binary.

---

And to allow SDKs to change this value just before deployment, the compiler generates an entry in the "ABI json", with the buffer offset in the binary.

```json
{
    "name": "SOMETHING",
    "configurableType": { ... },
    "offset": 7104
},
```

The last piece for configurables is how are they "decoded"?

This is done automatically by the compiler.

---

For example

```rust
configurable { SOMETHING: u64 = 1 }

fn main() -> u64 { SOMETHING }
```

will be desugared into something like

```rust
const SOMETHING: u64;

fn __entry() -> raw_slice {
    core::codec::abi_decode_in_place(&mut SOMETHING, 7104, 8);
    encode(main())
}

fn main() -> u64 {
    SOMETHING
}
```

This is not valid `sway`, but gives the idea of what is happening.

---

end