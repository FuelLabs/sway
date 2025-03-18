script;

trait MyTrait {
    fn f(self) -> bool;
} {
    fn f2(self) -> bool {
        self.f()
    }
}

trait MyTrait2: MyTrait {
}

// A stack overflow could happen here due to using MyTrait2 vs MyTrait for T1
// This ended up going through the ReplaceDecl machinery and choosing the wrong
// trait method due to incorrect trait constraint type checking ordering logic.


impl<T1> MyTrait for (T1,)
where
    T1: MyTrait2,
{
    fn f(self) -> bool { self.0.f() }
} 

fn main() -> bool {
    true
}
