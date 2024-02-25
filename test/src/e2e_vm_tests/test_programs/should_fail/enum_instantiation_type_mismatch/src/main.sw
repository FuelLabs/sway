// This test proves that https://github.com/FuelLabs/sway/issues/5583 is fixed.
script;

use std::option::Option as OptionAlias;

enum MyOption {
    Some: bool,
    None: (),
}

enum MyOtherOption {
    NoSome: bool,
    None: (),
}

enum GenericEnum<A, B> {
    A: A,
    B: B,
}

fn main() {
    let _: MyOption = Option::Some(123);
    let _: MyOption = Option::None;

    let _: MyOtherOption = Option::Some(123);
    let _: MyOtherOption = Option::None;

    let o: OptionAlias<u8> = Option::Some(123u64);
    let _ = match o {
        Some(x) => x == 123u8,
        _ => false,
    };

    let o: Option<u8> = OptionAlias::Some(123u64);
    let _ = match o {
        Some(x) => x == 123u8,
        _ => false,
    };

    let _: Option<u8> = Option::Some(123u64);

    let _: Option<u8> = Option::Some::<bool>(true);

    let _: Option<u8> = Option::Some::<bool>("not bool");

    let o: GenericEnum<_, _> = GenericEnum::A(123); // ERROR: Cannot infer "B".
    let _ = match o {
        GenericEnum::A(x) => x == 123u64, // No error here.
        _ => false,
    };

    // TODO: Unfortunately, at the moment we do not provide a help message here that
    // the error is coming from the enum declaration and not variable annotation.
    // This will wait until we introduce a separate type inference step.
    let o: GenericEnum<_, bool> = GenericEnum::<u8, _>::A(123u64);
    let _ = match o {
        GenericEnum::A(x) => x == 123u8,
        GenericEnum::B(x) => x == false,
    };

    let o: GenericEnum<u8, bool> = GenericEnum::<u8, u32>::A(123u8);
    let _ = match o {
        GenericEnum::A(x) => x == 123u8, // No error here.
        // TODO: This should not be an error but it is because of this bug:
        //       https://github.com/FuelLabs/sway/issues/5606
        //       Extend the check after the bug is solved.
        GenericEnum::B(x) => x == true, // No error here.
    };

    // Remove dead code warnings.
    let _ = MyOption::Some(true);
    let _ = MyOption::None;
    let _ = MyOtherOption::NoSome(true);
    let _ = MyOtherOption::None;
    let _ = GenericEnum::<u8, u8>::B(0);
}
