contract;

dep testlib;

abi TestContr {
    fn foo();
}

fn foo() {
   testlib::foo();
}

impl TestContr for Contract {
    fn foo() {
       foo();
    }
}

fn main() {

}
