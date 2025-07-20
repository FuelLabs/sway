library;

struct Data { }

impl Data {
    fn add_values(self, other: Data) -> u64 {
        0
    }
}

fn main() -> u64 {
    let data1 = Data { };
    let data2 = Data { };
    data1.add_values(data2, data2)
}