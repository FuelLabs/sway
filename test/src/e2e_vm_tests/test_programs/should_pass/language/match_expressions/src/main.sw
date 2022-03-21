script;

fn main() -> u64 {
    let x = 5;
    let a = match 8 {
        7 => { 4 },
        9 => { 5 },
        8 => { 6 },
        _ => { 100 },
    };
    let b = match x {
        5 => { 42 },
        _ => { 24 },
    };
    match 42 {
        0 => { 24 },
        foo => { foo },
    }
}
