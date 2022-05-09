script;

enum X {
    Y: u64,
}

enum Sale {
    Cash: u64,
    Card: u64,
    Check: u64,
}

struct Point<T> {
    x: T,
    y: T,
}

enum Data<T> {
    One: T,
    Two: T,
    Three: T
}

pub enum Option<T> {
    Some: T,
    None: (),
}

impl<T> Option<T> {
    fn is_some(self) -> bool {
        match self {
            Option::Some(_) => {
                true
            },
            Option::None => {
                false
            }
        }
    }

    fn is_none(self) -> bool {
        match self {
            Option::Some(_) => {
                false
            },
            Option::None => {
                true
            }
        }
    }

    fn unwrap(self) -> T {
        match self {
            Option::Some(inner_value) => {
                inner_value
            },
            Option::None => {
                0
            }
        }
    }
}

struct S {
    n: str[17],
    v: u64,
}

struct A {
    a: u64,
    b: bool,
    s: S,
}

enum B {
    B: A,
}

struct C {
    a: u64,
    b: bool,
    d: S,
    c: str[17],
}

enum D {
    D: C,
}

enum E {
    E: bool
}

fn main() -> u64 {
    let a = X::Y(42);
    let b = match a {
        X::Y(hi) => { hi },
        _ => { 24 },
    };
    let c = match a {
        X::Y(10) => { 10 },
        _ => { 24 },
    };
    let d = Sale::Card(5);
    let e = match d {
        Sale::Check(_) => { 1 },
        Sale::Cash(_) => { 2 },
        Sale::Card(4) => { 3 },
        Sale::Card(_) => { 4 },
    };
    let f = Point {
        x: 0u64,
        y: 0u64
    };
    let g = match f {
        Point { x, y: 1 } => { 0 },
        Point { x: 1, y } => { 1 },
        Point { x, y } => { 2 },
    };
    let h = Data::One(true);
    let i = match h {
        Data::Two(true) => { 0 },
        Data::Three(false) => { 1 },
        Data::Two(false) => { 2 },
        Data::Three(_) => { 3 },
        Data::One(true) => { 4 },
        Data::One(false) => { 5 },
    };
    let j = Data::Two(Point {
        x: 7u8,
        y: 8u8
    });
    let k = match j {
        Data::One(Point { x: 7u8, y: 8u8 }) => { 0 },
        Data::Three(Point { x: 7u8, y: 8u8 }) => { 1 },
        Data::Two(Point { x: 0u8, y }) => { 2 },
        Data::Three(_) => { 3 },
        Data::One(Point { x: _, y: 8u8 }) => { 4 },
        Data::Two(Point { x, y }) => { 5 },
        Data::Two(Point { x: 7u8, y: 8u8 }) => { 6 },
        Data::One(Point { x, y }) => { 7 },
    };
    let l = if let Data::Two(Point { x, y }) = j {
        1
    } else {
        0
    };
    let m = Option::Some(4);
    let n = Option::None::<u64>();
    let o = if let D::D(x) = D::D(C {
        d: S {
            n: " an odd length",
            v: 18
        },
        a: 10,
        b: false,
        c: " an odd length",
    }) {
        x.a
    } else {
        0
    };
    let p = if let E::E(x) = E::E(false) {
        10
    } else {
        0
    };
    let q = match E::E(false) {
        E::E(x) => { 10 },
        _ => { 0 }
    };
    let r = if let B::B(b) = B::B(A { a: 10, b: false, s: S { n: " an odd length", v: 20 } }) {
        b.a
    } else {
        0
    };

    if b == 42
        && c == 24
        && e == 4
        && g == 2
        && i == 4
        && k == 5
        && l == 1
        && m.unwrap() == 4
        && n.unwrap() == 0
        && o == 10
        && p == 10
        && q == 10
        && r == 10
        {
        42
    } else {
        0
    }
}
