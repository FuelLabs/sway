// This test proves that https://github.com/FuelLabs/sway/issues/5383 is fixed.

script;

mod lib;

use lib::*;

enum Enum {
    A: (),
    B: (),
}

struct Struct {
   x: u64,
   y: u64,
}

impl Struct {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

pub struct PubStruct {
   x: u64,
   y: u64,
}

impl PubStruct {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

struct GenericStruct<T> {
    x: T,
    y: u64,
}

impl<T> GenericStruct<T> {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

pub const X: bool = true;

fn access_struct(s: Struct) {
   poke(s.y);
}

fn access_enum(e: Enum) {
   match e {
      Enum::B => poke(e),
      _ => (),
   };
}

fn main() {
   let s = Struct { x: 0, y: 0 };
   let _ = PubStruct { x: 0, y: 0 };
   let _ = GenericStruct { x: 0, y: 0 };
   let e = Enum::B;
   let _: bool = X;

   access_struct(s);
   access_enum(e);

   Struct { x: 0, y: 0 }.use_me();
   PubStruct { x: 0, y: 0 }.use_me();
   GenericStruct { x: 0, y: 0 }.use_me();
   poke(Enum::A);
}

fn poke<T>(_x: T) { }