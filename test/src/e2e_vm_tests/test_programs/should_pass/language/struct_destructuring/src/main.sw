script;

// Tests struct destructuring

fn main() -> u64 {
    let data = Data {
        value: 42,
    };

    let Data { value } = data;
    return value;
}

struct Data { 
    value: u64,
}
