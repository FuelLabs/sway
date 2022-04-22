library storage;

pub fn store<T>(key: b256, value: T) {
    asm(r1: key, r2: value) {
        sww r1 r2;
    };
}

pub fn get<T>(key: b256) -> T {
    asm(r1: key, r2) {
        srw r2 r1;
        r2: T
    }
}
