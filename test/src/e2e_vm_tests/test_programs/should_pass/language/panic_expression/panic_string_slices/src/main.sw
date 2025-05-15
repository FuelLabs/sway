script;

// TODO: Uncomment once str constants are supported.
// const CONST: str = "This is a constant string.";

fn main(choice: u8) {
    // panic_const();
    call_return_const_str_0();
    call_return_const_str_1();
    call_return_const_str_2();
    panic_if_result();
    call_return_const_str_no_const_eval(choice);
    // TODO: Uncomment this line once https://github.com/FuelLabs/sway/issues/7107 is fixed.
    // panic return_const_str(42);
    panic "This is a panic string.";
}

fn return_const_str(choice: u8) -> str {
    match choice {
        0 => "This is the first panic string.",
        1 => "This is the second panic string.",
        _ => "This is a default panic string.",
    }
}

fn call_return_const_str_no_const_eval(choice: u8) {
    panic return_const_str(choice);
}

fn call_return_const_str_0() {
    panic return_const_str(0);
}

fn call_return_const_str_1() {
    panic return_const_str(1);
}

fn call_return_const_str_2() {
    panic return_const_str(2);
}

fn panic_if_result() {
    panic if true {
        "This is a panic string in then branch."
    } else {
        "This is a panic string in else branch."
    }
}

// fn panic_const() {
//     panic CONST;
// }
