library;

pub struct LibStruct {
    pub x_1: u64,
    pub x_2: u64,
    y_1: u64,
    y_2: u64,
}

impl LibStruct {
    pub fn new() -> Self {
        Self { x_1: 0, x_2: 0, y_1: 0, y_2: 0 }
    }
}
