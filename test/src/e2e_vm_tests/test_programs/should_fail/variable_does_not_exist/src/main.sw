script;

struct S {}

impl S {
    fn associated(a: u64, b: u64, c: u64) -> u64 {
        a + b + c
    }
}

fn function(a: u64, b: u64, c: u64) -> u64 {
    a + b + c
}

fn main() {
    let _ = S::associated(x, y, z);


    let _ = function(x, y, z);


    let _ = x + y + z;
}