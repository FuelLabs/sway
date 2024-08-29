script;

mod lib;

// Glob import should not result in any shadowing issues 
use lib::*;

// const shadowing an imported const with glob 
const X1: u64 = 0;

// types and traits shadowing imported items with glob 
struct MyStruct11 {}
enum MyStruct21 {}
trait MyStruct31 {}
abi MyStruct41 {}


struct MyEnum12 {}
enum MyEnum22 {}
trait MyEnum32 {}
abi MyEnum42 {}


struct MyTrait13 {}
enum MyTrait23 {}
trait MyTrait33 {}
abi MyTrait43 {}


struct MyAbi14 {}
enum MyAbi24 {}
trait MyAbi34 {}
abi MyAbi44 {}

configurable {
    X3: u64 = 22,
}

fn main() -> u64 {
    // var shadowing an imported const with glob
    let X2 = 20;

    X1 + X2 + X3
}
