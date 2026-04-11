// ignore garbage_collection_all_language_tests - needs a experimental feature
script;

enum SomeEnum {
    A: u64,
    B: u16,
}

fn encode_decode(s: SomeEnum) -> SomeEnum {
    let bytes = encode(s);
    abi_decode::<TrivialEnum<SomeEnum>>(bytes).unwrap()
}

#[require(trivially_decodable = "true")]
struct MyStruct {
    a: TrivialEnum<SomeEnum>,
}

fn main() {
    let bytes = encode(SomeEnum::A(1));
    let s = abi_decode::<MyStruct>(bytes);
    let e = s.a.unwrap();
}

#[test]
fn unwrap_trivial_variant() {
    let _ = encode_decode(SomeEnum::A(1));
}

#[test(should_revert)]
fn unwrap_non_trivial_variant() {
    let _ = encode_decode(SomeEnum::B(2));
}
