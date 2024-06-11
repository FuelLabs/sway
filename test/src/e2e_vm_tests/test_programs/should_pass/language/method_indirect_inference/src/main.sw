script;

fn main() -> u64 {
    use std::bytes::Bytes;
    use std::convert::TryFrom;
    
    let mut bytes = Bytes::new();
    bytes.push(1_u64.try_into().unwrap());

    1
}
