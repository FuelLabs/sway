script;

struct Struct {
  x: u64,
}

fn main() -> u64 {
    let p = Struct {
        x: 0,
    };

    match p {
        Struct { x, nn_1 } => { x },
        Struct { x, nn_1, nn_2 } => { x },
    };

    0
}
