script;

trait Get1 {
    fn get(self) -> u64;
}

trait Get2 {
    fn get(self) -> u64;
}

struct Data1 {
    value: u64
}

struct Data2 {
    value: u64
}

impl Get1 for Data1 {
    fn get(self) -> u64 {
        self.value
    }
}

impl Get2 for Data2 {
    fn get(self) -> u64 {
        self.value
    }
}

fn main() -> u64 {
    let a = Data1 {
        value: 7
    };
    let b = Data2 {
        value: 8
    };
    let c = a.get();

    0
}
