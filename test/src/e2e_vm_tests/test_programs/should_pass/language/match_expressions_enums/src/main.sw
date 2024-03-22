script;

enum E {
    F: u64,
    G: (u64, u64),
    H: (u64, u64, u64),
}

struct S {
    x: u64,
    y: u64,
    z: u64,
}

enum X {
    Y: u64,
    Z: bool,
    K: E,
    S: S,
}

fn match_me(me: X) -> u64 {
    match me {
        X::Y(10) | X::Y(20) | X::Y(30) => { 102030 },
        X::Y(hi) => { hi },
        X::Z(false) => { 1000 },
        X::K(E::F(x) | E::G((101, x)) | E::H((202, 303, x))) => x,
        X::S(S {x: a, y: 102, z: 103}) | X::S(S {x: 201, y: a, z: 203}) | X::S(S {x: 301, y: 302, z: a}) => a,
        X::S(_) => 8888,
        _ => { 9999 },
    }
}

enum GenericE<T> {
    A: (),
    B: T,
    C: T,
}

fn main() -> u64 {
    let x = match_me(X::Y(42));
    assert (x == 42);

    let x = match_me(X::Y(20));
    assert (x == 102030);

    let x = match_me(X::Z(false));
    assert (x == 1000);

    let x = match_me(X::K(E::F(42)));

    assert (x == 42);

    let x = match_me(X::K(E::G((101, 42))));

    assert (x == 42);

    let x = match_me(X::K(E::H((202, 303, 42))));

    assert (x == 42);

    let x = match_me(X::K(E::G((202, 42))));

    assert (x == 9999);

    let x = match_me(X::K(E::H((303, 303, 42))));

    assert (x == 9999);

    let x = match_me(X::S(S { x:42, y: 102, z:103 }));
    assert (x == 42);

    let x = match_me(X::S(S { x:201, y: 42, z:203 }));
    assert (x == 42);

    let x = match_me(X::S(S { x:301, y: 302, z: 42 }));
    assert (x == 42);

    let x = match_me(X::S(S { x:42, y: 902, z:103 }));
    assert (x == 8888);

    let x = match_me(X::S(S { x:901, y: 42, z:203 }));
    assert (x == 8888);

    let x = match_me(X::S(S { x:301, y: 902, z: 42 }));
    assert (x == 8888);

    let x = match_me(X::Z(true));
    assert (x == 9999);

    let x = match_generic_of_u64(GenericE::A);
    assert (x == 11);

    let x = match_generic_of_u64(GenericE::B(42u64));
    assert (x == 42);

    let x = match_generic_of_u64(GenericE::B(10));
    assert (x == 102030);

    let x = match_generic_of_u64(GenericE::B(333));
    assert (x == 222333);

    let x = match_generic_of_u64(GenericE::C(222));
    assert (x == 222333);

    let x = match_generic_of_u64(GenericE::B(1234));
    assert (x == 1234);

    match_me(X::Y(42))
}

fn match_generic_of_u64(e: GenericE<u64>) -> u64 {
    match e {
        GenericE::A => 11,
        GenericE::B(42u64) => 42,
        GenericE::B(10u64) | GenericE::B(20u64) | GenericE::B(30u64) => 102030,
        GenericE::C(222u64) | GenericE::B(333u64) => 222333,
        GenericE::B(x) => x,
        _ => 9999,
    }
}