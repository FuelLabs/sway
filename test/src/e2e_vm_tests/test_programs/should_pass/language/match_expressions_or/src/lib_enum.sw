library;

enum Enum {
    A: (u64, u64, u64),
    B: (u64, u64, u64),
    C: (u64, u64, u64),
}

fn match_enum(e: Enum) -> u64 {
    match e {
        Enum::A((11, 11, 11)) | Enum::B((22, 22, 22)) | Enum::C((33, 33, 33)) => 112233,
        Enum::A((_, _, x)) | Enum::B((_, x, _)) | Enum::C((x, _, _)) => x,
    }
}

fn match_option(o: Option<Enum>) -> u64 {
    match o {
        Some(
            Enum::A((11, 11, 11))
            | Enum::A((22, 22, 22))
            | Enum::B((111, 111, 111))
            | Enum::B((222, 222, 222))
        ) => 111111222222,
        Some(
           Enum::A((x, 11, 11))
           | Enum::A((22, x, 22))
           | Enum::B((111, 111, x))
           | Enum::B((x, 222, 222))
        ) => x,
        None => 5555,
        _ => 9999,
    }
}

pub fn test() -> u64 {
    let x = match_enum(Enum::A((11, 11, 11)));
    assert(x == 112233);

    let x = match_enum(Enum::B((22, 22, 22)));
    assert(x == 112233);

    let x = match_enum(Enum::C((33, 33, 33)));
    assert(x == 112233);

    let x = match_enum(Enum::A((0, 0, 42)));
    assert(x == 42);

    let x = match_enum(Enum::B((0, 42, 0)));
    assert(x == 42);

    let x = match_enum(Enum::C((42, 0, 0)));
    assert(x == 42);

    let x = match_option(Some(Enum::A((11, 11, 11))));
    assert(x == 111111222222);

    let x = match_option(Some(Enum::A((22, 22, 22))));
    assert(x == 111111222222);

    let x = match_option(Some(Enum::B((111, 111, 111))));
    assert(x == 111111222222);

    let x = match_option(Some(Enum::B((222, 222, 222))));
    assert(x == 111111222222);

    let x = match_option(Some(Enum::A((42, 11, 11))));
    assert(x == 42);

    let x = match_option(Some(Enum::A((22, 42, 22))));
    assert(x == 42);

    let x = match_option(Some(Enum::B((111, 111, 42))));
    assert(x == 42);

    let x = match_option(Some(Enum::B((42, 222, 222))));
    assert(x == 42);

    let x = match_option(None);
    assert(x == 5555);

    let x = match_option(Some(Enum::C((0, 0, 0))));
    assert(x == 9999);

    42
}
