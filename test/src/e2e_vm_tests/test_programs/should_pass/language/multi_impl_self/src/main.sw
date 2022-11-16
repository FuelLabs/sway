script;

struct MyStruct {
}

impl MyStruct {
    pub fn my_fun() -> u64 {
        fun()
    }
}

impl MyStruct {
}

fn fun() -> u64 {
    42
}

fn main() -> u64 {
    MyStruct::my_fun()
}
