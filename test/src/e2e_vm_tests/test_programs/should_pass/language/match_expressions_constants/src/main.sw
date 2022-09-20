script;

const NUMBER_1: u64 = 7;
const NUMBER_2: u64 = 14;
const NUMBER_3: u64 = 5;

fn main() -> u64 {
    let a = 5;

    match a {
        NUMBER_1 => 1,
        NUMBER_2 => 1,
        NUMBER_3 => 42,
        other => other,
    }
}
