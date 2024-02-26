script;

use std::option::Option as OptionAlias;

enum GenericEnum2<A, B> {
    A: A,
    B: B,
}

enum GenericEnum3<A, B, C> {
    A: A,
    B: B,
    C: C,
}

// These tests prove that https://github.com/FuelLabs/sway/issues/5492 is fixed.
fn check_5492_if() {
    let o = bar(true);
    match o {
        Some(x) => assert(x == 19),
        None => assert(false),
    };

    let o = bar(false);
    match o {
        Some(_) => assert(false),
        None => assert(true),
    };

    let g = generic_bar(true);
    match g {
        GenericEnum2::A(x) => assert(x == 123u8),
        GenericEnum2::B(x) => { 
            let _ = x == true;
            assert(false);
        },
    };

    let g = generic_bar(false);
    match g {
        GenericEnum2::A(x) => {
            let _ = x == 123u8;
            assert(false);
        },
        GenericEnum2::B(x) => assert(x == true),
    };
}

fn check_5492_match() {
    let o = foo(true);
    match o {
        Some(x) => assert(x == 17),
        None => assert(false),
    };

    let o = foo(false);
    match o {
        Some(_) => assert(false),
        None => assert(true),
    };

    let g = generic_foo(true);
    match g {
        GenericEnum2::A(x) => assert(x == 123u8),
        GenericEnum2::B(x) => { 
            let _ = x == true;
            assert(false);
        },
    };

    let g = generic_foo(false);
    match g {
        GenericEnum2::A(x) => {
            let _ = x == 123u8;
            assert(false);
        },
        GenericEnum2::B(x) => assert(x == true),
    };
}

fn bar(b: bool) -> Option<u8> {
   if(b) {
     Option::Some(19)
   } else {
     Option::None
   }
}

fn generic_bar(b: bool) -> GenericEnum2<u8, bool> {
   if(b) {
     GenericEnum2::A(123)
   } else {
     GenericEnum2::B(true)
   }
}

fn foo(b: bool) -> Option<u8> {
  match Some(b) {
    Option::Some(true) => Option::Some(17),
    Option::Some(false) => Option::None,
    Option::None => Option::None,
  }
}

fn generic_foo(b: bool) -> GenericEnum2<u8, bool> {
  match Some(b) {
    Option::Some(true) => GenericEnum2::A(123),
    Option::Some(false) => GenericEnum2::B(true),
    Option::None => GenericEnum2::B(true),
  }
}

fn main() -> u64 {
    let o: Option<u8> = Option::Some(123);
    let _ = match o {
        Some(x) => assert(x == 123u8),
        _ => assert(false),
    };

    let o: Option<_> = Option::Some(123);
    let _ = match o {
        Some(x) => assert(x == 123u64),
        _ => assert(false),
    };

    let o: OptionAlias<u8> = Option::Some(123);
    let _ = match o {
        Some(x) => assert(x == 123u8),
        _ => assert(false),
    };

    let o: Option<u8> = OptionAlias::Some(123);
    let _ = match o {
        Some(x) => assert(x == 123u8),
        _ => assert(false),
    };

    let o: GenericEnum2<_, _> = GenericEnum2::<_, bool>::A(123);
    let _ = match o {
        GenericEnum2::A(x) => assert(x == 123u64),
        GenericEnum2::B(x) => { 
            let _ = x == true;
            assert(false);
        },
    };

    let o: GenericEnum2<_, _> = GenericEnum2::<u64, bool>::A(123);
    let _ = match o {
        GenericEnum2::A(x) => assert(x == 123u64),
        GenericEnum2::B(x) => { 
            let _ = x == true;
            assert(false);
        },
    };

    let o: GenericEnum2<_, _> = GenericEnum2::<u8, bool>::A(123);
    let _ = match o {
        GenericEnum2::A(x) => assert(x == 123u8),
        GenericEnum2::B(x) => { 
            let _ = x == true;
            assert(false);
        },
    };

    let o: GenericEnum2<u8, _> = GenericEnum2::<_, bool>::A(123);
    let _ = match o {
        GenericEnum2::A(x) => assert(x == 123u8),
        GenericEnum2::B(x) => { 
            let _ = x == true;
            assert(false);
        },
    };

    let o: GenericEnum2<_, bool> = GenericEnum2::<u8, _>::A(123);
    let _ = match o {
        GenericEnum2::A(x) => assert(x == 123u8),
        GenericEnum2::B(x) => { 
            let _ = x == true;
            assert(false);
        },
    };

    let o: GenericEnum3<_, _, _> = GenericEnum3::<u8, bool, u32>::A(123);
    let _ = match o {
        GenericEnum3::A(x) => assert(x == 123u8),
        GenericEnum3::B(x) => { 
            let _ = x == true;
            assert(false);
        },
        GenericEnum3::C(x) => { 
            let _ = x == 0u32;
            assert(false);
        },
    };

    let o: GenericEnum3<u8, bool, u32> = GenericEnum3::<_, _, _>::A(123);
    let _ = match o {
        GenericEnum3::A(x) => assert(x == 123u8),
        GenericEnum3::B(x) => { 
            let _ = x == true;
            assert(false);
        },
        GenericEnum3::C(x) => { 
            let _ = x == 0u32;
            assert(false);
        },
    };

    let o: GenericEnum3<_, _, _> = GenericEnum3::<u8, bool, u32>::A(123u8);
    let _ = match o {
        GenericEnum3::A(x) => assert(x == 123u8),
        GenericEnum3::B(x) => { 
            let _ = x == true;
            assert(false);
        },
        GenericEnum3::C(x) => { 
            let _ = x == 0u32;
            assert(false);
        },
    };

    let o: GenericEnum3<_, bool, _> = GenericEnum3::<u8, _, u32>::A(123);
    let _ = match o {
        GenericEnum3::A(x) => assert(x == 123u8),
        GenericEnum3::B(x) => { 
            let _ = x == true;
            assert(false);
        },
        GenericEnum3::C(x) => { 
            let _ = x == 0u32;
            assert(false);
        },
    };

    let o: GenericEnum3<_, bool, _> = GenericEnum3::<u8, _, u32>::B(true);
    let _ = match o {
        GenericEnum3::A(x) => { 
            let _ = x == 0u8;
            assert(false);
        }
        GenericEnum3::B(x) => assert(x == true),
        GenericEnum3::C(x) => { 
            let _ = x == 0u32;
            assert(false);
        },
    };

    check_5492_if();
    check_5492_match();

    // Remove dead code warnings.
    let _ = GenericEnum2::<u8, u8>::B(0);
    let _ = GenericEnum3::<u8, u8, u8>::B(0);
    let _ = GenericEnum3::<u8, u8, u8>::C(0);

    42
}
