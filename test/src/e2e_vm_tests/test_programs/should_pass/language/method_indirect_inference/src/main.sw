script;

fn main() -> u64 {
    use std::bytes::Bytes;
    use std::convert::TryFrom;
    use std::primitive_conversions::u8::*;

    let mut bytes = Bytes::new();
    bytes.push(1_u64.try_into().unwrap());

    1
}
