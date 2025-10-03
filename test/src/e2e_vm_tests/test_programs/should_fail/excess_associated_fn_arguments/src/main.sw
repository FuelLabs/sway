library;

struct Data { }

impl Data {
    fn add_values(_first: Data, _other: Data) -> u64 {
        0
    }
}

pub fn main() {
    let data1 = Data { };
    let data2 = Data { };
    Data::add_values(data1, data2, data2);
}