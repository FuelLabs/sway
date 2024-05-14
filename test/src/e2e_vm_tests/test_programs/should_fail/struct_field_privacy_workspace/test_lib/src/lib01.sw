library;

pub mod lib01_nested;

struct Lib01PrivateStruct {
    pub x: u64,
    y: u64,
}

impl Lib01PrivateStruct {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

pub struct Lib01PublicStruct {
    pub x: u64,
    y: u64,
}

impl Lib01PublicStruct {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

pub fn use_me() {
    let s = Lib01PrivateStruct { x: 0, y: 0 };
    s. use_me();

    let s = Lib01PublicStruct { x: 0, y: 0 };
    s. use_me();
}

fn poke<T>(_x: T) { }