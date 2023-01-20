script;

dep lib;

// Check for shadowing library imports
use lib::{MyAbi1, MyEnum1, MyStruct1, MyTrait1};

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


// Check for shadowing local declarations 
pub struct MyStruct2 {}
pub enum MyEnum2 {}
trait MyTrait2 {}
abi MyAbi2 {}


// Make sure that each of these declarations is problematic because of name
// conflicts with the local declarations above 
struct MyStruct2 {}
enum MyStruct2 {}
trait MyStruct2 {}
abi MyStruct2 {}


struct MyEnum2 {}
enum MyEnum2 {}
trait MyEnum2 {}
abi MyEnum2 {}


struct MyTrait2 {}
enum MyTrait2 {}
trait MyTrait2 {}
abi MyTrait2 {}


struct MyAbi2 {}
enum MyAbi2 {}
trait MyAbi2 {}
abi MyAbi2 {}


fn main() {}
