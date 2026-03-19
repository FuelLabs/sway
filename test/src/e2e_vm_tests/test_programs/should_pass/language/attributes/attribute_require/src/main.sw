script;

mod another_file;
use another_file::InnerStruct;

#[require(trivially_decodable = "true")]
struct MyStruct {
    a: bool,
    b: u16,
    c: u32,
    d: InnerStruct,
    e: EnumThatCanUseTrivialEnum,
    f: EnumThatCannotUseTrivialEnum,
    g: Vec<u64>,
    h: Result<Vec<u64>, u64>,
}

enum EnumThatCanUseTrivialEnum {
    A: ()
}

enum EnumThatCannotUseTrivialEnum {
    A: Vec<u64>,
}

fn main(s: MyStruct) {
    // To disable unused warnings
    __log(s.a);
    __log(s.b);
    __log(s.c);
    __log(s.d);
    __log(s.d.a);
    __log(s.e);
    __log(s.f);
    __log(s.g);
    __log(s.h);
    __log(EnumThatCanUseTrivialEnum::A);
    __log(EnumThatCannotUseTrivialEnum::A(Vec::new()));
}
