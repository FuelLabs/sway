script;
trait Cat {
 fn speak(self) -> u64;
}
trait Dog {
 fn speak(self) -> u64;
}
struct S<T> {
 x: T,
}
impl<T> S<T>
where
 T: Cat,
{
 fn foo(self) -> u64 {
 self.x.speak() + 1
 }
}
impl<T> S<T>
where
 T: Dog,
{
 fn foo(self) -> u64 {
 self.x.speak() + 2
 }
}
impl Dog for u64 {
 fn speak(self) -> u64 {
 2
 }
}
impl Cat for u64 {
 fn speak(self) -> u64 {
 1
 }
}
fn main() -> u64 {
 let s = S::<u64> { x: 1 };
 s.foo();

 42
}