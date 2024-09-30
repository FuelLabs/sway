library;

struct S {}

impl S {
    fn assoc() -> u8 {
        Self::S_ASSOC
    }
}

impl S {
    const S_ASSOC: u8 = Self::assoc();
}