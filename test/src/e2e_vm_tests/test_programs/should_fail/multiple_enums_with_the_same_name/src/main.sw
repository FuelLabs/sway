script;

dep module0;
dep module1;

fn main() {
    let mut x = module0::MyEnum::A;
    let y = module1::MyEnum::A;
    x = y;
}
