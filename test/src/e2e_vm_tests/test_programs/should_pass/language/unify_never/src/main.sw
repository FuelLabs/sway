script;

// Test of unification of the Never type

fn test_1(){
 let mut v: u32 = 1;
 v = return;
}

fn test_2(){
 let mut v: u32 = return;
 v = 1;
}


fn main() {
    test_1();
    test_2();
}
