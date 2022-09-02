script;

fn foo() {}
    // do something
fn bar() {}


    // do something
enum SomeEnum {
    A: u64,
    B: bool,
    C: b256,
}

fn main() -> u64 {
    let x = 5;


    // Match as an expression.
    let a = match 8 {
        7 => {
            4
        },
        9 => {
            5
        },
        8 => {
            6
        },
        _ => {
            100
        },
    };


    // Match as a statement for control flow.
    match x {
        5 => {
            foo()
        },
        _ => {
            bar()
        },
    };


    // Match an enum
    let e = SomeEnum::A(42);
    let v = match e {
        SomeEnum::A(val) => {
            val
        },
        SomeEnum::B(true) => {
            1
        },
        SomeEnum::B(false) => {
            0
        },
        _ => {
            0
        },
    };


    // Match as expression used for a return.
    match 42 {
        0 => {
            24
        },
        foo => {
            foo
        },
    }
}
