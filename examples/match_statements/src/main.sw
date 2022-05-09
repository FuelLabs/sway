script;

fn foo() {
    // do something
}
fn bar() {
    // do something
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
