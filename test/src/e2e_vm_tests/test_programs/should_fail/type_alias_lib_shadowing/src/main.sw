script;

mod lib;

use lib::MyType;

#[allow(dead_code)]
struct MyStruct {}

#[allow(dead_code)]
type MyType = MyStruct;

fn main() {}
