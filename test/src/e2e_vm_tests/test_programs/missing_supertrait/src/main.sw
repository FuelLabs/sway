script;

trait Type {
    fn get_number_of_bytes(self) -> u64;
}

trait Numeric {} // Should be `trait Numeric : Type {} 

struct U64 {
    n: u64,
}

impl Numeric for U64 {
    fn get_number_of_bytes(self) -> u64 {
        8
    }
}

fn main() {
    let n = U64 { n : 0 };
    let bytes1 = n.get_number_of_bytes();
}
