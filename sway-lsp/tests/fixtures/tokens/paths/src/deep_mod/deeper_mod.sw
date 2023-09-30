library;

pub fn deep_fun(){}

pub const DEEPER_CONST: u32 = 0;

pub enum DeepEnum {
    Variant: (),
    Number: u32,
}

pub struct DeepStruct<T> {
    field: T,
}
