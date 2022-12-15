script;

dep lib;

use lib::*;

fn main() {
    MyStruct { x: 42 }.foo();


    MyStruct::bar();
}