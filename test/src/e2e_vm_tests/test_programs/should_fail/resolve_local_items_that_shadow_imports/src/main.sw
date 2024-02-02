// This test proves that https://github.com/FuelLabs/sway/issues/5383 is fixed.

script;

mod lib;

use lib::Enum;
use lib::Struct;
use lib::PubStruct;
use lib::GenericStruct;

enum Enum {
    A: (),
    B: (),
}

// TODO: Remove all the `pub`s from all the structs once https://github.com/FuelLabs/sway/issues/5500 is fixed.
struct Struct {
   pub x: u64,
   pub y: u64,
}

impl Struct {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

pub struct PubStruct {
   pub x: u64,
   pub y: u64,
}

impl PubStruct {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

struct GenericStruct<T> {
    pub x: T,
    pub y: u64,
}

impl<T> GenericStruct<T> {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

pub const X: bool = true;

fn main() {
    // We must get errors for defining items multiple times,
    // but that should be all errors.
    // We shouldn't have any errors below, because the
    // below items will resolve to local ones.
    let _ = Struct { x: 0, y: 0 };
    let _ = PubStruct { x: 0, y: 0 };
    let _ = GenericStruct { x: 0, y: 0 };
    let _ = Enum::B;
    let _: bool = X;

    Struct { x: 0, y: 0 }.use_me();
    PubStruct { x: 0, y: 0 }.use_me();
    GenericStruct { x: 0, y: 0 }.use_me();
    poke(Enum::A);
}

fn poke<T>(_x: T) { }