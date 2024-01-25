library;

pub struct LibStruct {
    pub x: u64,
    y: u64,
    pub other: LibOtherStruct,
}

impl LibStruct {
    pub fn new() -> Self {
        Self { x: 0, y: 0, other: LibOtherStruct { x: 0, y: 0 } }
    }

    pub fn constructor(v: u64) -> Self {
        Self { x: v, y: v, other: LibOtherStruct { x: v, y: v } }
    }
}

pub struct LibOtherStruct {
    pub x: u64,
    y: u64,
}

impl LibOtherStruct {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}