script;

dep lib_a;

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
    let b = func;
    let b = func();


    let s = S::new;
    let s = S::new();


    let b = lib_a::inner_lib::func;
    let b = lib_a::inner_lib::func();


    let s = lib_a::inner_lib::S2::new2;
    let s = lib_a::inner_lib::S2::new2();


    let n: Option<u64> = Option::None();


    let n = Option::None::<u64>();


    let n = lib_a::inner_lib::MyEnum::VariantA;
    let n = lib_a::inner_lib::MyEnum::VariantA();

    5
}
