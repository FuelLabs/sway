script;

struct Data1 {
    value: u64
}

impl Data1 {
    fn get(self) -> u64 {
        self.value
    }


    fn get2(self) -> u64 {
        self.value
    }

    fn get3(self) -> u64 {
        self.value
    }
}

fn main() -> u64 {
    let d = Data1 { value: 42 };
    d.get3()
}
