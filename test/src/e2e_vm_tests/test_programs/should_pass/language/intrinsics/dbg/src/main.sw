script;

struct SomeStruct {
    a: u64,
    b: str,
}

fn main() -> u64 {
    let s = __dbg(SomeStruct { a: 1u64, b: "Hello dbg!"});
    __dbg(s.a + s.b.len())
}