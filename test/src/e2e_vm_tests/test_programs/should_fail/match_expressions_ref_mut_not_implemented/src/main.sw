script;

fn main() -> u64 {
    let x = 0u8;
    let _ = match x {
        ref v => {
            v
        },
    };

    let _ = match x {
        mut v => {
            v
        },
    };

    let _ = match x {
        ref mut v => {
            v
        },
    };

    if let ref v = x { };

    if let mut v = x { };

    if let ref mut v = x { };
}
