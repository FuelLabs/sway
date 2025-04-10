script;

struct ExampleStruct {
    variable: u32,
}

enum ExampleEnum {
    Variants: u32,
}

const EXAMPLE_CONST: u64 = 0;

fn main() {
    let _ = match 0 {
        EXAMPLE_CONST => 1,
        a => a + 1,
        _ => 0,
    };

    match EXAMPLE_CONST {
        _ => {}
    }
    if let EXAMPLE_CONST = EXAMPLE_CONST {}
    if EXAMPLE_CONST == EXAMPLE_CONST {}

    let _ = match Option::Some(Option::Some(EXAMPLE_CONST)) {
        Option::None => 1,
        Option::Some(Option::None) => 2,
        Option::Some(Option::Some(EXAMPLE_CONST)) => 2,
        _ => 4,
    };

    let a = ExampleStruct { variable: 0 };
    let _ = match a {
        ExampleStruct { variable } => 0,
    };
}
