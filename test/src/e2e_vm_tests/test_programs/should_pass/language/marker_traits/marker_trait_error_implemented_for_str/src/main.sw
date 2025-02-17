library;

mod use_error_explicitly;
mod use_error_via_glob;

// Using `Error` from core library prelude.
fn implements_error<T>(_t: T) where T: Error { }
fn implements_error_no_args<T>() where T: Error { }

pub fn main() {
    implements_error("str");
    implements_error_no_args::<str>();
    use_error_explicitly::test();
    use_error_via_glob::test();
}
