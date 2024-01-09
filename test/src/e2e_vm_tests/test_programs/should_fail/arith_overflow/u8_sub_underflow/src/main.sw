script;

fn main() -> bool {
    let a: u8 = u8::min();
    let b: u8 = 1;

    let result: u8 = a - b;
    log(result);

    true
}
