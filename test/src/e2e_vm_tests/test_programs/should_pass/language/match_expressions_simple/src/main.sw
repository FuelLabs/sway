script;

fn main() -> u64 {
    let a = 5;
    let b = match 8 {
        7 => { 4 },
        9 => { 5 },
        8 => { 6 },
        _ => { 100 },
    };
    let c = match a {
        5 => { 42 },
        _ => { 24 },
    };
    let d = match 42 {
        0 => { 24 },
        foo => { foo },
    };

    let e = (
        (1, 2),
        (
            (3, 4),
            5
        )
    );
    let f = match e {
        ((3, _), _) => { 99 },
        (_, (_, 5)) => { 42 },
        _ => { 0 },
    };

    f
}
