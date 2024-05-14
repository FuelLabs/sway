script;

mod lib;

use lib::*;

struct MainStruct {
    pub x: u64,
    y: u64,
    other: LibOtherStruct,
}

fn main() {
    let ls = LibStruct::new();

    let _ = ls.x;

    let _ = ls.y;
    
    let _ = ls.other.x;
    
    let _ = ls.other.y;

    let ms = MainStruct { x: 0, y: 0, other: LibOtherStruct::new() };
    let _ = ms.x;
    let _ = ms.y;
    let _ = ms.other;
    
    let _ = ms.other.x;

    let _ = ms.other.y;
}
