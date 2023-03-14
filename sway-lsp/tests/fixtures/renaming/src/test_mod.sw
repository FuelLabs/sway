library;

pub fn test_fun() -> u32 {
    42
}

pub struct DeepStruct {
    field: u32,
}

impl DeepStruct {
    pub fn new(field: u32) -> Self {
        Self { field: field }
    }
}

pub enum DeepEnum {
    Variant: (),
    Number: u32,
}

