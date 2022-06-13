script;

// Tests struct field reassignments and accessing fields from a returned struct.

fn main() -> u64 {
    let mut data = Data {
        uselessnumber: 42,
    };
    data.uselessnumber = 43;

    let other = ret_struct().uselessnumber;

    return data.uselessnumber;
}

struct Data {
    uselessnumber: u64,
}

fn ret_struct() -> Data {
    Data {
        uselessnumber: 44,
    }
}
