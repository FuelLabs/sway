library;

fn conditional() {
    // ANCHOR: conditional
    let number = 5;

    if number % 3 == 0 {
        // call function 1
    } else if number % 4 == 0 {
        // call function 2
    } else {
        // call function 3
    }

    // more code here
    // ANCHOR_END: conditional
}

fn single_loop() {
    // ANCHOR: single_loop
    let mut counter = 0;
    let mut condition = true;
    while counter < 10 && condition {
        counter += 1;
        if 5 < counter {
            condition = false;
        }
    }
    // ANCHOR_END: single_loop
}

fn nested_loop() {
    // ANCHOR: nested_loop
    while true {
        // computation here
        while true {
            // more computation here
        }
    }
    // ANCHOR_END: nested_loop
}

fn break_example() {
    // ANCHOR: break_example
    let mut counter = 0;
    while counter < 10 {
        counter += 1;
        if 5 < counter {
            break;
        }
    }
    // ANCHOR_END: break_example
}

fn continue_example() {
    // ANCHOR: continue_example
    let mut counter = 0;
    while counter < 10 {
        counter += 1;
        if counter % 2 == 0 {
            continue;
        }
        // "other code"
    }
    // ANCHOR_END: continue_example
}

// ANCHOR: if_let_enum
enum Foo {
    One: (),
    Two: (),
}
// ANCHOR_END: if_let_enum

fn example1() {
    // ANCHOR: if_let_example1
    let one = Foo::One;
    let mut result = 0;
    
    if let Foo::One = one {
        result = 1;
    }
    // ANCHOR_END: if_let_example1
}

fn example2() {
    // ANCHOR: if_let_example2
    let one = Foo::One;
    let result = if let Foo::One = one {
        1
    } else {
        2
    };
    // ANCHOR_END: if_let_example2
}


fn simple_match() {
    // ANCHOR: simple_match
    let number = 5;

    let result = match number {
        0 => 10,
        1 => 20,
        5 => 50,
        6 | 7 => 60,
        catch_all => 0,
    };
    // ANCHOR_END: simple_match
}

fn multi_line_match() {
    // ANCHOR: multi_line_match
    let number = 5;

    let result = match number {
        0 => {
            // Multiple lines of code here then return 10
            10
        },
        1 => 20,
        5 => 50,
        catch_all => 0,
    };
    // ANCHOR_END: multi_line_match
}

// ANCHOR: nested_enum_match
enum TopLevel {
    One: (),
    Two: SecondLevel,
}

enum SecondLevel {
    Value1: u64,
    Value2: (),
}

fn nested_match(input: TopLevel) -> u64 {
    match input {
        TopLevel::One => 1,
        TopLevel::Two(second) => {
            match second {
                SecondLevel::Value1(2) => 2,
                SecondLevel::Value1(_) => 3,
                SecondLevel::Value2 => 42,
            }
        },
    }
}
// ANCHOR_END: nested_enum_match

// ANCHOR: complex_multi_arg_enum_match
enum Binary {
    True: (),
    False: (),
}

impl PartialEq for Binary {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Binary::True, Binary::True) => true,
            (Binary::False, Binary::False) => true,
            _ => false,
        }
    }
}
// ANCHOR_END: complex_multi_arg_enum_match

// ANCHOR: complex_struct_unpacking_match
struct Point {
    x: u64,
    y: u64
}

fn struct_matching() {
    let point = Point {
        x: 1u64,
        y: 2u64,
    };

    let result = match point {
        Point { x: 5, y } => y + 1,
        Point { x, .. } => x,
        Point { y, .. } => y,
        _ => 42,
    };
}
// ANCHOR_END: complex_struct_unpacking_match

// ANCHOR: complex_enum_match
enum Color {
    Red: (),
    Green: (),
    Blue: (),
}

fn enum_match(input: Color) {
    let result = match input {
        Color::Red => 0,
        Color::Green => 1,
        Color::Blue => 2,
    };
}
// ANCHOR_END: complex_enum_match

// ANCHOR: complex_constant_match
const NUMBER_1: u64 = 7;
const NUMBER_2: u64 = 14;

fn constant_match() {
    let number = 5;

    let result = match number {
        NUMBER_1 => 1,
        NUMBER_2 => 42,
        other => other,
    };
}
// ANCHOR_END: complex_constant_match
