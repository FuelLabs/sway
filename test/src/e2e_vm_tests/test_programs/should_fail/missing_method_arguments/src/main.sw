script;

struct Data {
    value: u64
}

impl Data {
    fn add_values(self, other: Data) -> u64 {
        self.value + other.value
    }
}

fn main() -> u64 {
    let data1 = Data {
        value: 42u64
    };
    let data2 = Data {
        value: 1u64
    };
    data1.add_values()
}