predicate;

use std::inputs::input_predicate_data;

#[cfg(experimental_error_type = true)]
#[error_type]
enum Errors {
    #[error(m = "Error A.")]
    A: (),
}

#[cfg(experimental_error_type = true)]
fn main() -> bool {
    let received: u32 = match input_predicate_data::<u32>(0) {
        Some(data) => data,
        None => return false,
    };
    
    match received {
        0 => panic,
        1 => panic (),
        2 => panic "str",
        3 => panic Errors::A,
        _ => true,
    }
}

#[cfg(experimental_error_type = false)]
fn main() -> bool {
    let received: u32 = match input_predicate_data::<u32>(0) {
        Some(data) => data,
        None => return false,
    };
    
    match received {
        0 => __revert(0),
        1 => __revert(1),
        2 => __revert(2),
        3 => __revert(3),
        _ => true,
    }
}
