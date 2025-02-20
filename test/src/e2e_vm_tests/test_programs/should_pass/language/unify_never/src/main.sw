script;

// Test of unification of the Never type

fn test_1(){
    let mut v1: u32 = 1;
    v1 = return;
}

fn test_2(){
    let mut v2: u32 = return;
    v2 = 1;
}


impl [u32;0] {
    fn foo(self) -> u64 {
        32
    }
}

impl [!;0] {
    fn foo(self) -> u64{
        64
    }
}

fn main() -> u64 {
    test_1();
    test_2();

    let x:[u32;0] = [];
    x.foo() // should return 32
}
