---
layout: cover
marp: true
---

# Sway Encoding

---

- How it works for contracts?
  - Callers POV
  - Callee POV
- **Interlude**: Why even encode?
  - API versus ABI
- How it works for
    - scripts/predicates
    - receipts

---

# How it works? (caller's POV)

At some point, someones calls a contract:

```rust
let contract = abi(TestContract, CONTRACT_ID);
contract.some_method(0);
```

This will be desugared as

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

and, in the end, will be compiled into a `call` instruction.

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

# How it works? (callee's POV)

Code above was calling a contract implemented like:

```rust
impl TestContract for Contract {
    fn some_method(qty: u64) {
        ...
    }
}
```

---

This will be desugared into:

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

`__contract_entry_some_method` is the original `some_method` as is. `__contract_ret` is a special function that immediately returns the current context, that is why there is no `return` in the generated code.

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

# **Interlude**: Why?

- All this begs the question... why?
    - Why all the extra cost of encoding and decoding?
    - why we don´t just pass all parameters directly?

---

API define the types that are exchanged:

```rust
abi MyContract {
    fn some_method(arg: Vec<u64>);
}
```

but they do not define how these types are passed around. In the example above, the only safe way to call a contract, would be to use the **exactly same version** of the stdlib the contract implementation is using.

Why?

Any internal change to `Vec` could make calling a contract impossible.

--- 

Example: Suppose that caller and callee were compiled using `std-lib v1`.

```rust
struct Vec<T> {
    items: raw_ptr,
    cap: u64,
    len: u64,
}
```

But callee decide to update to `std-lib v2`, that for some reason was changed to:

```rust
struct Vec<T> {
    len: u64,
    items: raw_ptr,
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

Now imagine everything that can affect byte layouts: lib versions, compiler versions, compiler flags, optimizations etc....

All this can break contract calls without anyone noticing. All this, actually, break also whoever is consuming these bytes: indexers, SDKs, whoever is interpreting receipts etc...



