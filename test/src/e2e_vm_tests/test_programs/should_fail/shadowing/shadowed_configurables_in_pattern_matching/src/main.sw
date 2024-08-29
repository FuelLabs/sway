script;

enum E {
    V: (u8, u8, u8),
}

struct S {
    x: u8,
    y: u8,
    z: u8,
}

struct SConfig {
    CONFIG: u8,
}

configurable {
    X: u8 = 10,
    Y: u8 = 11,
    Z: u8 = 12,
    CONFIG: u8 = 13,
}

fn main() {
    let _ = match 42 {
        X => 101,
    };

    let e = E::V((100, 100, 100));
    let _ = match e {
        E::V((_, X, _)) => 101,
        E::V((X, Y, Z)) => 102,
        E::V((X, _, _)) | E::V((_, _, X)) => 103,
    };

    let s = S { x: 100, y: 100, z: 100 };
    let _ = match s {
        S { x: X, .. } => 101,
        S { y: X, .. } => 102,
        S { x: X, y: Y, z: Z } => 103,
        S { x: X, .. } | S { z: X, .. } => 104,
    };

    let s = SConfig { CONFIG: 100 };
    let _ = match s {
        SConfig { CONFIG } => 101,
        SConfig { CONFIG: CONFIG } => 102,
        SConfig { CONFIG: CONFIG } | SConfig { CONFIG } => 103,
    };
}
