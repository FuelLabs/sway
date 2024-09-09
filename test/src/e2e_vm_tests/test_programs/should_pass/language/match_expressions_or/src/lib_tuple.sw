library;

fn match_tuple(t: (u64, u64)) -> u64 {
    match t {
        (11, 111) | (22, 222) | (33, 333) => 112233,
        (11, x) | (22, x) | (33, x) => x,
        _ => {
            return 9999;
        },
    }
}

pub fn test() -> u64 {
    let x = match_tuple((11, 111));
    assert(x == 112233);

    let x = match_tuple((22, 222));
    assert(x == 112233);

    let x = match_tuple((33, 333));
    assert(x == 112233);

    let x = match_tuple((11, 42));
    assert(x == 42);

    let x = match_tuple((22, 42));
    assert(x == 42);

    let x = match_tuple((33, 42));
    assert(x == 42);

    let x = match_tuple((0, 0));
    assert(x == 9999);

    42
}
