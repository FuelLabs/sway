script;

mod another_file;
use another_file::InnerStruct;

#[require(trivially_decodable = "true")]
struct MyStruct {
    a: bool,
    b: InnerStruct,
    c: SomeEnum,
    d: Vec<u64>,
}

enum SomeEnum {
    A: ()
}

fn main(s: MyStruct) {
    __log(s.a);
    __log(s.b.a);
    __log(s.c);
    __log(s.d);
    let a = SomeEnum::A;
    __log(a);
}
