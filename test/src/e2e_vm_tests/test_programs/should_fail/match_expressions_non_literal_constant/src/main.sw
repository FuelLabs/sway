script;

enum Data {
    A: bool,
    B: bool,
}

const MY_DATA1: Data = Data::A(true);
const MY_DATA2: Data = Data::A(false);
const MY_DATA3: Data = Data::B(true);
const MY_DATA4: Data = Data::B(false);

fn main() -> u64 {
    let d = Data::B(true);
    match d {
        MY_DATA1 => 1,
    }
}
