library;

use std::marker::Error;

#[error_type]
enum Enum {
    #[error(m = "error message")]
    A: (),
}

fn implements_error<T>(_t: T) where T: Error { }
fn implements_error_no_args<T>() where T: Error { }

pub fn test() {
    implements_error("str");
    implements_error_no_args::<str>();
    implements_error(());
    implements_error_no_args::<()>();
    implements_error(Enum::A);
    implements_error_no_args::<Enum>();
}
