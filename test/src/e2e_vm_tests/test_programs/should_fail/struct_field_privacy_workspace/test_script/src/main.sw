script;

mod lib01;
mod test;

use lib01::*;
use lib01::lib01_nested::*;

use test_lib::lib01::Lib01PublicStruct as ExternalLib01PublicStruct;
use test_lib::lib01::lib01_nested::Lib01PublicNestedStruct as ExternalLib01PublicNestedStruct;

use test_lib::*;

struct ScriptLocalStruct {
    pub x: u64,
    y: u64,
}

fn main() {
    let _ = Lib01PublicStruct { x: 0, y: 0 };
    let _ = Lib01PublicNestedStruct { x: 0, y: 0 };

    let local = ScriptLocalStruct { x: 0, y: 0 };
    poke(local.x);
    poke(local.y);

    let _ = ExternalLib01PublicStruct { x: 0, y: 0 };
    let _ = ExternalLib01PublicNestedStruct { x: 0, y: 0 };

    ::test::test_me();

    ::lib01::use_me();
    ::lib01::lib01_nested::use_me();

    test_lib::use_me();
}

fn poke<T>(_x: T) { }