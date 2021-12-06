script;

fn main() -> u64 {
    let x = 5;
    let y = match x {
        5 => 42,
        _ => 24,
    };
    y
}
