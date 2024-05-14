script;

mod lib;

use ::lib::Struct;
use ::lib::Struct as StructAlias;

struct GenericStruct2<A, B> {
    a: A,
    b: B,
}

struct GenericStruct3<A, B, C> {
    a: A,
    b: B,
    c: C,
}

// These tests prove that https://github.com/FuelLabs/sway/issues/5492 is fixed for structs as well.
fn check_5492_if() {
    let s = bar(true);
    assert(s.x == 19u8);

    let s = bar(false);
    assert(s.x == 91u8);

    let g = generic_bar(true);
    assert(g.a == 123u8);
    assert(g.b == true);

    let g = generic_bar(false);
    assert(g.a == 111u8);
    assert(g.b == false);
}

fn check_5492_match() {
    let s = foo(true);
    assert(s.x == 17u8);

    let s = foo(false);
    assert(s.x == 71u8);

    let g = generic_foo(true);
    assert(g.a == 123u8);
    assert(g.b == true);

    let g = generic_foo(false);
    assert(g.a == 111u8);
    assert(g.b == false);
}

fn bar(b: bool) -> Struct<u8> {
   if(b) {
     Struct { x: 19 }
   } else {
     Struct { x: 91 }
   }
}

fn generic_bar(b: bool) -> GenericStruct2<u8, bool> {
   if(b) {
     GenericStruct2 { a: 123, b: true }
   } else {
     GenericStruct2 { a: 111, b: false }
   }
}

fn foo(b: bool) -> Struct<u8> {
  match Some(b) {
    Option::Some(true) => Struct { x: 17 },
    Option::Some(false) => Struct { x: 71 },
    Option::None => Struct { x: 71 },
  }
}

fn generic_foo(b: bool) -> GenericStruct2<u8, bool> {
  match Some(b) {
    Option::Some(true) => GenericStruct2 { a: 123, b: true },
    Option::Some(false) => GenericStruct2 { a: 111, b: false },
    Option::None => GenericStruct2 { a: 111, b: false },
  }
}

fn main() -> u64 {
    let s: Struct<u8> = Struct { x: 123 };
    assert(s.x == 123u8);

    let s: Struct<_> = Struct { x: 123 };
    assert(s.x == 123u64);

    let s: StructAlias<u8> = Struct { x: 123 };
    assert(s.x == 123u8);

    let s: Struct<u8> = StructAlias { x: 123 };
    assert(s.x == 123u8);

    let s: GenericStruct2<_, _> = GenericStruct2::<_, bool> { a: 123, b: true };
    assert(s.a == 123u64);
    assert(s.b == true);

    let s: GenericStruct2<_, _> = GenericStruct2::<u64, bool> { a: 123, b: true };
    assert(s.a == 123u64);
    assert(s.b == true);

    let s: GenericStruct2<_, _> = GenericStruct2::<u8, bool> { a: 123, b: true };
    assert(s.a == 123u8);
    assert(s.b == true);

    let s: GenericStruct2<u8, _> = GenericStruct2::<_, bool> { a: 123, b: true };
    assert(s.a == 123u8);
    assert(s.b == true);

    let s: GenericStruct2<_, bool> = GenericStruct2::<u8, _> { a: 123, b: true };
    assert(s.a == 123u8);
    assert(s.b == true);

    let s: GenericStruct3<_, _, _> = GenericStruct3::<u8, bool, u32> { a: 123, b: true, c: 456 };
    assert(s.a == 123u8);
    assert(s.b == true);
    assert(s.c == 456u32);

    let s: GenericStruct3<u8, bool, u32> = GenericStruct3::<_, _, _> { a: 123, b: true, c: 456 };
    assert(s.a == 123u8);
    assert(s.b == true);
    assert(s.c == 456u32);

    let s: GenericStruct3<_, _, _> = GenericStruct3::<u8, bool, u32> { a: 123u8, b: true, c: 456 };
    assert(s.a == 123u8);
    assert(s.b == true);
    assert(s.c == 456u32);

    let s: GenericStruct3<_, bool, _> = GenericStruct3::<u8, _, u32> { a: 123, b: true, c: 456 };
    assert(s.a == 123u8);
    assert(s.b == true);
    assert(s.c == 456u32);

    check_5492_if();
    check_5492_match();

    42
}
