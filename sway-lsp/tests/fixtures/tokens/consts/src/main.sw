contract;

mod more_consts;
use more_consts::{Data, Value};

/// documentation for CONSTANT_1
const CONSTANT_1: u64 = 100;
/// CONSTANT_2 has a value of 200
const CONSTANT_2: u32 = 200;
const BASE_TOKEN: ContractId = ContractId::from(0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c);
const MY_DATA: Data = Data::B(Value {a: 100});
const EXAMPLE: Option<Option<u32>> = Option::None;

struct Point {
    x: u64,
    y: u32,
}

fn test() {
    // Constants defined in the same module
    let point = Point { x: CONSTANT_1, y: CONSTANT_2 };
    let contract_id = BASE_TOKEN;

    // Constants defined in a different module
    let point = Point { x: more_consts::CONSTANT_3, y: more_consts::CONSTANT_4 };
    let data = more_consts::MY_DATA1;
}
