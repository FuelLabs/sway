library;

pub struct LibStruct {
    x: bool,
    y: u64,
}

impl LibStruct {
    pub fn new() -> Self {
        Self {
            x: false,
            y: 0,
        }
    }

    pub fn use_me(self) -> () {
        poke(self.x);
        poke(self.y);
    }
}

pub type LibStructAlias = LibStruct;

fn poke<T>(_x: T) { }