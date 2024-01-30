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

fn main() {
    let ls = LibStruct::new();

    let LibStruct { x_1, x_2 } = ls;
    let LibStruct { x_1, x_2, y_1 } = ls;
    let LibStruct { x_1 } = ls;
    let LibStruct { y_1, y_2 } = ls;
    let LibStruct { x_1, y_2, .. } = ls; 
    let LibStruct { .. } = ls;
    let LibStruct { } = ls;

    let ms = MainStruct::new();

    let MainStruct { x_1, x_2 } = ms;
    let MainStruct { x_1, x_2, y_1 } = ms;
    let MainStruct { x_1 } = ms;
    let MainStruct { y_1, y_2 } = ms;
    let MainStruct { x_1, y_2, .. } = ms;
    let MainStruct { .. } = ms;
    let MainStruct { } = ms;
}