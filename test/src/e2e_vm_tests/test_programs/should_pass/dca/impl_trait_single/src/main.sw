script;

trait Get1 {
    fn get(self) -> u64;
}

struct Data1 {
    value: u64
}

impl Get1 for Data1 {
    fn get(self) -> u64 {
        self.value
    }
}

fn main() -> u64 {
    let a = Data1 {
        value: 7
    };
    let c = a.get();
    0
}
