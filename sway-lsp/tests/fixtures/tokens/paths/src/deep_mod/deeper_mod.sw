library deeper_mod;

pub fn deep_fun(){}

pub enum DeepEnum {
    Variant: (),
    Number: u32,
}

pub struct DeepStruct<T> {
    field: T,
}
