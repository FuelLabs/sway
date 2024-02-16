script;

fn main() -> u64 {
    let mut vector = Vec::new();

    vector.push(0);
    vector.push(1);
    vector.push(2);
    vector.push(3);
    vector.push(4);

    let mut i = 0;

    for (_n,_m) in vector.iter() {
        i += 1;
    }

    assert(i == 5);

    0
}
