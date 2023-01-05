script;

struct Data {
    value: u64
}

impl Data {
    fn the_value(self) -> u64 {
        fn double(n: u64) -> u64 {
            n // + n
        }

        double(self.value)
    }
}

fn main() {
    fn bla() { }
}
