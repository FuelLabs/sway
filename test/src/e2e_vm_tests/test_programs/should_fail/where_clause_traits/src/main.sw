script;

trait MyTrait {
    fn method(self);
}

trait MyTraitGeneric<T> where T : MyTrait {
    fn method(self);
}

struct S1 {
    s1: u64
}

impl MyTraitGeneric<S1> for u64 {
    fn method(self){
    }
}

impl<T> MyTraitGeneric<T> for u32 {
    fn method(self){
    }
}

fn main() {
}