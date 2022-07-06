script;

// Tests struct destructuring

fn gimme_a_struct() -> Dummy {
    Dummy { value1: 1, value2: true }
}

fn main() -> u64 {
    let Dummy { value1, value2 } = gimme_a_struct();
    let Dummy { value1, value2 }: Dummy = gimme_a_struct();
    let data = Data {
        value: 42,
    };
    let Data { value }: Data = data;
    return value;
}

struct Data { 
    value: u64,
}

struct Dummy {
    value1: u64,
    value2: bool,
}
