library;

// We should definitely implement something like the "fully qualified syntax",
// but until then, multiple methods with the same name is undefined behavior.
// https://doc.rust-lang.org/rust-by-example/trait/disambiguating.html

struct Data<T> {
    value: T
}

trait MyAdd {
    fn my_add(self, other: Self) -> Self;
}

impl<T> MyAdd for Data<T> {
    fn my_add(self, other: Self) -> Self {
        other
    }
}

impl<T> Data<T> {
    // duplicate definition
    fn my_add(self, other: Self) -> Self {
        other
    }
}

impl Data<u64> {
    fn get_value(self) -> u64 {
        self.value
    }
}

impl Data<u64> {
    // duplicate definition
    fn get_value(self) -> u64 {
        self.value
    }
}

impl Data<u64> {
    // duplicate definition
    fn my_add(self, other: Self) -> Self {
        Data {
            value: self.value
        }
    }
}

impl Data<u32> {
    // duplicate definition
    fn my_add(self, other: Self) -> Self {
        Data {
            value: self.value
        }
    }
}

impl Data<u8> {
    fn get_value(self) -> u8 {
        self.value
    }
}