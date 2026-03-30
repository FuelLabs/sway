script;

enum SomeEnum {
    A: u64,
    B: bool,
}

fn encode_decode(s: SomeEnum) -> SomeEnum {
    let bytes = encode(s);
    abi_decode::<TrivialEnum<SomeEnum>>(bytes).unwrap()
}

fn main() {
}



#[test]
fn unwrap_trivial_variant() {
    let _ = encode_decode(SomeEnum::A(1));
}

#[test(should_revert)]
fn unwrap_non_trivial_variant() {
    let _ = encode_decode(SomeEnum::B(true));
}
