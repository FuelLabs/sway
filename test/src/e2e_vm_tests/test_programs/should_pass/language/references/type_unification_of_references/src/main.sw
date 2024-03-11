script;

struct S<T> {
    x: T,
}

fn test_referencing_numeric() {
    let r = &123;
    assert(*r == 123);

    let r = &123u8;
    assert(*r == 123u8);

    let r: &u32 = &123;
    assert(*r == 123u32);

    let r: &u64 = &123u64;
    assert(*r == 123u64);

    let r = &mut 123;
    assert(*r == 123);

    let r = &mut 123u8;
    assert(*r == 123u8);

    let r: &mut u32 = &mut 123;
    assert(*r == 123u32);

    let r: &mut u64 = &mut 123u64;
    assert(*r == 123u64);

    // ----------------

    let r = &S { x: 123 };
    assert(r.x == 123);

    let r = &S { x: 123u8 };
    assert(r.x == 123u8);

    let r: &S<u32> = &S { x: 123 };
    assert(r.x == 123u32);

    let r: &S<u64> = &S { x: 123u64 };
    assert(r.x == 123u64);

    let r = &mut S { x: 123 };
    assert(r.x == 123);

    let r = &mut S { x: 123u8 };
    assert(r.x == 123u8);

    let r: &mut S<u32> = &mut S { x: 123 };
    assert(r.x == 123u32);

    let r: &mut S<u64> = &mut S { x: 123u64 };
    assert(r.x == 123u64);

    // ----------------

    let r = &S { x: &123 };
    assert(*r.x == 123);

    let r = &S { x: &123u8 };
    assert(*r.x == 123u8);

    let r: &S<&u32> = &S { x: &123 };
    assert(*r.x == 123u32);

    let r: &S<&u64> = &S { x: &123u64 };
    assert(*r.x == 123u64);

    let r = &mut S { x: &123 };
    assert(*r.x == 123);

    let r = &mut S { x: &123u8 };
    assert(*r.x == 123u8);

    let r: &mut S<&u32> = &mut S { x: &123 };
    assert(*r.x == 123u32);

    let r: &mut S<&u64> = &mut S { x: &123u64 };
    assert(*r.x == 123u64);

    // ----------------
    
    let r = &Option::Some(123);
    match *r {
        Some(x) => assert(x == 123),
        None => assert(false),
    };

    let r = &Option::Some(123u8);
    match *r {
        Some(x) => assert(x == 123u8),
        None => assert(false),
    };

    let r: &Option<u8> = &Option::Some(123);
    match *r {
        Some(x) => assert(x == 123u8),
        None => assert(false),
    };

    let r: &Option<u32> = &Option::Some(123);
    match *r {
        Some(x) => assert(x == 123u32),
        None => assert(false),
    };

    let r: &Option<u64> = &Option::Some(123u64);
    match *r {
        Some(x) => assert(x == 123u64),
        None => assert(false),
    };
    
    let r = &mut Option::Some(123);
    match *r {
        Some(x) => assert(x == 123),
        None => assert(false),
    };

    let r = &mut Option::Some(123u8);
    match *r {
        Some(x) => assert(x == 123u8),
        None => assert(false),
    };

    let r: &mut Option<u8> = &mut Option::Some(123);
    match *r {
        Some(x) => assert(x == 123u8),
        None => assert(false),
    };

    let r: &mut Option<u32> = &mut Option::Some(123);
    match *r {
        Some(x) => assert(x == 123u32),
        None => assert(false),
    };

    let r: &mut Option<u64> = &mut Option::Some(123u64);
    match *r {
        Some(x) => assert(x == 123u64),
        None => assert(false),
    };

    // ----------------
    
    let r = &S { x: Option::Some(123) };
    match r.x {
        Some(x) => assert(x == 123),
        None => assert(false),
    };

    let r = &S { x: Option::Some(123u8) };
    match r.x {
        Some(x) => assert(x == 123u8),
        None => assert(false),
    };

    let r: &S<Option<u8>>  = &S { x: Option::Some(123) };
    match r.x {
        Some(x) => assert(x == 123u8),
        None => assert(false),
    };

    let r: &S<Option<u32>>  = &S { x: Option::Some(123) };
    match r.x {
        Some(x) => assert(x == 123u32),
        None => assert(false),
    };

    let r: &S<Option<u64>>  = &S { x: Option::Some(123u64) };
    match r.x {
        Some(x) => assert(x == 123u64),
        None => assert(false),
    };
    
    let r = &mut S { x: Option::Some(123) };
    match r.x {
        Some(x) => assert(x == 123),
        None => assert(false),
    };

    let r = &mut S { x: Option::Some(123u8) };
    match r.x {
        Some(x) => assert(x == 123u8),
        None => assert(false),
    };

    let r: &mut S<Option<u8>>  = &mut S { x: Option::Some(123) };
    match r.x {
        Some(x) => assert(x == 123u8),
        None => assert(false),
    };

    let r: &mut S<Option<u32>>  = &mut S { x: Option::Some(123) };
    match r.x {
        Some(x) => assert(x == 123u32),
        None => assert(false),
    };

    let r: &mut S<Option<u64>>  = &mut S { x: Option::Some(123u64) };
    match r.x {
        Some(x) => assert(x == 123u64),
        None => assert(false),
    };
}

fn main() -> u64 {
    test_referencing_numeric();

    42
}
