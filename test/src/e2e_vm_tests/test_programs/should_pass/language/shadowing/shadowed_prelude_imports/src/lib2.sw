library;

struct T {
    a: u64
}

// Add from std::prelude
impl Add for T {
    fn add(self, other: Self) -> Self {
        Self {
            a: self.a + other.a
        }
    }
}

pub fn log_tester(value: T) {
    log::<T>(value);
}
   
