# Scripts

A script is a deployed bytecode on the chain which executes to perform some task. It does not represent ownership of any resources and it cannot be called like a contract.

This example script calls a contract.

```sway
script;

use example_contract::MyContract;

struct InputStruct { 
  field_1: bool,
  field_2: u64
}

fn main () {
  let x = abi(MyContract, 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);
  let color = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
  let input = InputStruct {
    field_1: true,
    field_2: 3,
  };
  x.foo(5000, 0, color, input);
}
```
