library;

use std::marker::*;

fn implements_error<T>(_t: T) where T: Error { }
fn implements_error_no_args<T>() where T: Error { }

pub fn test() {
    implements_error("str");
    implements_error_no_args::<str>();
}
