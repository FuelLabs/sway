script;

struct S<T> {
    x: T,
}

fn main() {
    let x: <u8>::S::<u8> = S::<u8>{x: 8};
}