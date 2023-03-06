script;

mod lib;

// Check for shadowing library imports
use lib::{MyAbi0 as MyAbi1, MyEnum0 as MyEnum1, MyStruct0 as MyStruct1, MyTrait0 as MyTrait1};

// Make sure that each of these declarations is problematic because of name
// conflicts with imported items
struct MyStruct1 {}
enum MyStruct1 {}
trait MyStruct1 {}
abi MyStruct1 {}


struct MyEnum1 {}
enum MyEnum1 {}
trait MyEnum1 {}
abi MyEnum1 {}


struct MyTrait1 {}
enum MyTrait1 {}
trait MyTrait1 {}
abi MyTrait1 {}


struct MyAbi1 {}
enum MyAbi1 {}
trait MyAbi1 {}
abi MyAbi1 {}

fn main() {}
