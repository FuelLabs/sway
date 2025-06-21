predicate;

use std::inputs::input_predicate_data;

#[error_type]
enum Errors {
    #[error(m = "Error A.")]
    A: (),
}

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
