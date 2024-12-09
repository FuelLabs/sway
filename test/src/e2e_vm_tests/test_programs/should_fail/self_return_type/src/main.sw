script;
trait MyTrait {
    fn foo(self, other: Self) -> Self;
}
impl MyTrait for u8 {
    fn foo(self, other: Self) -> self {
        self
    }
}
fn main() -> () {
    let a = 1u8;
    let _ = a.foo(2u8);
}
