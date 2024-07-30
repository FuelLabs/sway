---
layout: cover
marp: true
---

# Sway Encoding

---

- How it works?
    - Callers POV
    - Callee POV

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
STACK  ....................  │  CONTRACT_ID │                                       
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
    let method_name = decode_first_param::<str>();
    if method_name == "some_method" {
        let mut buffer = BufferReader::from_second_parameter();
        let args: (u64,) = buffer.decode::<(u64,)>();
        let _result: () = __contract_entry_some_method(args.0);
        let _result: raw_slice = encode::<()>(_result);
        __contract_ret(_result.ptr(), _result.len::<u8>());
    }
    __revert(123);
}
```

`__contract_entry_some_method` is the original `some_method` as is.  `__contract_ret` is a special function that immediately returns the current context, that is why there is not `return` in the generated code.


