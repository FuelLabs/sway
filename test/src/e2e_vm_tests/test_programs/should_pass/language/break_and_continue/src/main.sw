script;

fn main() -> u64 {
    let m = 5;
    let mut i = 0;
    // Expand this example to have many nested loops
    while i < 10 {
        i += 1;
        if i > 5 {
            break;
        }
    }
    i // 6
}
