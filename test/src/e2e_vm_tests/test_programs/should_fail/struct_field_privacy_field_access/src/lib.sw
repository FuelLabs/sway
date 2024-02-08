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