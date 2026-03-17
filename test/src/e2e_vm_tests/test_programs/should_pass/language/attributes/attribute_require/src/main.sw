script;

mod another_file;
use another_file::InnerStruct;

#[require(trivially_decodable = "true")]
struct MyStruct {
    a: u64,
    b: InnerStruct,
    c: u64,
    d: u64,
}

fn main(s: MyStruct) {
    __log(s.a);
    __log(s.b.a);
    __log(s.c);
    __log(s.d);
}
