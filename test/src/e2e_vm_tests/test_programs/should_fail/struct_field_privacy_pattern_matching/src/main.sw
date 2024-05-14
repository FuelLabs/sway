script;

mod lib;

use lib::*;

struct MainStruct {
    pub x_1: u64,
    pub x_2: u64,
    y_1: u64,
    y_2: u64,
}

impl MainStruct {
    pub fn new() -> Self {
        Self { x_1: 0, x_2: 0, y_1: 0, y_2: 0 }
    }
}

struct EmptyStruct { }

fn main() {
    let ls = LibStruct::new();

    let _ = match ls {
        LibStruct { x_1: 0, x_2: 0 } => 1,
        LibStruct { x_1: 0, x_2: 0, y_1: 0 } => 1,
        LibStruct { x_1: 0 } => 1,
        LibStruct { y_1: 0, y_2: 0 } => 1,
        LibStruct { x_1: 0, y_2: 0, .. } => 1, // There should be no suggestion to use `..`.
        LibStruct { .. } => 1,
        LibStruct { } => 1,
        _ => 0,
    };

    let ms = MainStruct::new();

    let _ = match ms {
        MainStruct { x_1: 0, x_2: 0 } => 1,
        MainStruct { x_1: 0, x_2: 0, y_1: 0 } => 1,
        MainStruct { x_1: 0 } => 1,
        MainStruct { x_1: 0, y_2: 0, .. } => 1,
        MainStruct { .. } => 1,
        MainStruct { } => 1,
        _ => 0,
    };

    let es = EmptyStruct { };

    let _ = match es {
        EmptyStruct { } => 1,
        EmptyStruct { .. } => 1,
    };
}