library;

pub struct LibStruct {
    pub x: u64,
    y: u64,
    pub nested: LibNestedStruct,
}

impl LibStruct {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            nested: LibNestedStruct {
                x: 0,
                y: 0,
            }
        }
    }
}

pub struct LibNestedStruct {
    pub x: u64,
    y: u64,
}