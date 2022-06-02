contract;

struct S {
    x: u64,
    y: u64,
    z: u64,
    w: u64,
    b: u64,
}

pub enum E {
    A: S,
    B: u64,
}

abi StorageAccess {
    // Setters
    fn set_e(s: S, u: u64);
    fn get_e() -> (E, E);
}

storage {
    e1: E,
    e2: E,
}

impl StorageAccess for Contract {
    fn set_e(s: S, u: u64) {
        storage.e1 = E::A(s);
        storage.e2 = E::B(u);
    }

    fn get_e() -> (E, E) {
        (storage.e1, storage.e2)
    }
}
