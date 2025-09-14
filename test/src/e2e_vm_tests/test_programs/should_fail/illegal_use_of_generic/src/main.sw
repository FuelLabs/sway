library;

pub fn test_function<T>(_value: T) {
    T::new(); // 1.
}

struct Data<T> {
    value: T
}

impl<T> Data<T> {
    fn test_function(self) {
        T::new(); // 2.
    }
}
