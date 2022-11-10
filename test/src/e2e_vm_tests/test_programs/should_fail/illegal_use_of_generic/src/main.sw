script;

// We want this to error without crashing the compiler.
fn test_function<T>(value: T) {
    T::new();
}

struct Data<T> {
    value: T
}

impl<T> Data<T> {
    fn test_function(self) {
        T::new();
    }
}

fn main() {

}
