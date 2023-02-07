library more_consts;

struct Value {
    a: u32,
}

enum Data {
    A: bool,
    B: Value,
}

pub const CONSTANT_3: u32 = 300;
pub const CONSTANT_4: u32 = 400;
pub const MY_DATA1: Data = Data::A(true);
