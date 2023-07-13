script;

struct S {}

struct S2 {}

trait MySuperTrait {
    fn method();
}

trait MyTrait : MySuperTrait {
    fn method();
}

impl MySuperTrait for S {
    fn method() { }
}

impl MyTrait for S {
    fn method() { }
}

fn main() {
    <S as MyTrait>::asd::method();


    <S as S2>::method();


    S::method(); // ambiguous method call here          
}