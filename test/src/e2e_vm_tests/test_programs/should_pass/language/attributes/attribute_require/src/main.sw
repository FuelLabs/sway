script;

#[require(trivially_decodable = "true")]
struct MyStruct {
    f0: bool,
    f1: u8,
    f2: u16,
    f3: u32,
    f4: u64,
    f5: u256,
    f6: b256,
    f7: TrivialStruct,
    f8: NonTrivialStruct,
    f9: SomeEnum,

    f11: Vec<u64>,
    f12: Result<Vec<u64>, u64>,
    f13: Result<u64, u64>,

    f14: (u64, NonTrivialStruct),
    f15: [NonTrivialStruct; 1],
    f16: [(u64, NonTrivialStruct); 2],
}

struct TrivialStruct {
}

struct NonTrivialStruct {
    pub a: bool,
}

enum SomeEnum {
    A: ()
}

abi SomeAbi {
    #[require(trivially_decodable = "true")]
    fn some_fn_1(a: NonTrivialStruct, b: SomeEnum) -> SomeEnum;

    #[require(trivially_decodable = "true")]
    fn some_fn_2(a: u64, b: u32, c: u16, d: u8, e: bool);
}

fn main(s: MyStruct) {
    // To disable unused warnings
    __log(s.f0);
    __log(s.f1);
    __log(s.f2);
    __log(s.f3);
    __log(s.f4);
    __log(s.f5);
    __log(s.f6);
    __log(s.f7);
    __log(s.f8.a);
    __log(s.f9);
    __log(s.f11);
    __log(s.f12);
    __log(s.f13);
    __log(s.f14);
    __log(s.f15);
    __log(s.f16);

    let _ = TrivialStruct { };
    __log(SomeEnum::A);
}
