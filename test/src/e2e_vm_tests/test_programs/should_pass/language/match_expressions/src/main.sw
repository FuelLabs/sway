script;

fn main() -> u64 {
    let x = 5;
    let _a = match 8 {
        7 => { 4 },
        9 => { 5 },
        8 => { 6 },
        _ => { 100 },
    };
    let _b = match x {
        5 => { 42 },
        _ => { 24 },
    };
    match 42 {
        0 => { 24 },
        foo => { foo },
    }
}
