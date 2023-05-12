script;

mod lib_a;

fn func() -> bool {
    true
}

struct S {}

impl S {
    fn new2() -> Self {
        S {}
    }
}

fn main() -> u64 {

    // check that calling function and methods with no parameters still requires parenthesis
    let _b = func;
    let _b = func();


    let _s = S::new;
    let _s = S::new();


    let _b = lib_a::inner_lib::func;
    let _b = lib_a::inner_lib::func();


    let _s = lib_a::inner_lib::S2::new2;
    let _s = lib_a::inner_lib::S2::new2();


    let _n: Option<u64> = None();


    let _n = None::<u64>();


    let _n = lib_a::inner_lib::MyEnum::VariantA;
    let _n = lib_a::inner_lib::MyEnum::VariantA();

    5
}
