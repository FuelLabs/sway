script;

const NUMBER_1: u64 = 7;
const NUMBER_2: u64 = 14;
const NUMBER_3: u64 = 5;

const TRUE: bool = true;
const FALSE: bool = false;

fn main() -> u64 {
    let a = 5;

    let b = match a {
        NUMBER_1 => 1,
        NUMBER_2 => 1,
        NUMBER_3 => 42,
        other => other,
    };

    let c = true;
    let d = match c {
        TRUE => 42,
        FALSE => 1,
    };

    if b == 42 && d == 42 {
        42
    } else {
        1
    }
}
