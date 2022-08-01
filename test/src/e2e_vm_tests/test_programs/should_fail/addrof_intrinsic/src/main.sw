script;

fn main() -> u64 {
    let number0 = 1u8;
    let xyz = __addr_of(number0);

    let x = ();
    let _ = __addr_of(x);
    0
}
