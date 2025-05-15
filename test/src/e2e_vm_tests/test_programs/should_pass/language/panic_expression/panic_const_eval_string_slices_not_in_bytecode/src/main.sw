script;

// TODO: Move this test to ir_generation tests once experimental features are supported there.

fn main() {
    call_return_const_str_0();
    panic "This is a panic string.";
}

fn return_const_str(choice: u8) -> str {
    match choice {
        0 => "This is the first panic string.",
        1 => "This is the second panic string.",
        _ => "This is a default panic string.",
    }
}

fn call_return_const_str_0() {
    panic return_const_str(0);
}