script;

struct S {
    x: u64,
}

pub fn main() {
    let _ = S{
        x:1,
        x: "a",
    };
}
