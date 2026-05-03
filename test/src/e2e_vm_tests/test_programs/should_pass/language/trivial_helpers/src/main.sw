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
    a: TrivialBool,
    b: TrivialEnum<SomeEnum>,
    c: TrivialVec<u64, 1>,
}

// These methods will be used to check how good the asm generation is
#[inline(never)]
fn trivial_vec_new_snapshot() -> TrivialVec<u64, 4> {
    TrivialVec::<u64, 4>::new()
}

#[inline(never)]
fn trivial_vec_push_snapshot(ref mut v: TrivialVec<u64, 4>, item: u64) {
    let _ = v.push(item);
}

#[inline(never)]
fn trivial_vec_as_slice_snapshot(ref v: TrivialVec<u64, 4>) -> &[u64] {
    v.as_slice()
}

fn encode_tuple_decode_my_struct(ref vvvvv: TrivialVec<u64, 1>) -> MyStruct {
    let bytes = encode((true, SomeEnum::A(1), vvvvv));
    abi_decode::<MyStruct>(bytes)
}

fn main() {
    let mut v: TrivialVec<u64, 1> = TrivialVec::<u64, 1>::new();
    let _ = v.push(1);

    encode_tuple_decode_my_struct(v);
    
    // let a = s.a.unwrap();
    // let b = s.b.unwrap();
    // let c = s.c.as_slice();

    // let mut v = trivial_vec_new_snapshot();
    // trivial_vec_push_snapshot(v, 1);
    // let _ = trivial_vec_as_slice_snapshot(v);
}

#[test]
fn unwrap_trivial_variant() {
    let _ = encode_decode(SomeEnum::A(1));
}

#[test(should_revert)]
fn unwrap_non_trivial_variant() {
    let _ = encode_decode(SomeEnum::B(2));
}
