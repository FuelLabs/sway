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


fn main() {
    test_1();
    test_2();
}
