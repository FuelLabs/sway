library;

struct Data { }

impl Data {
    fn add_values(self, _other: Data) -> u64 {
        0
    }
}

pub fn main() -> u64 {
    let data1 = Data { };
    data1.add_values()
}