script;

trait MyTrait {
    fn call_trait(self) -> Self;
}

impl<A, B> MyTrait for (A, B) where A:MyTrait, B:MyTrait  {
    fn call_trait(self) -> Self {
        self
    }
}

impl MyTrait for u64 {
    fn call_trait(self) -> Self {
        0
    }
}

struct MyStruct<T> {
    x: T,
}

impl<T> MyStruct<T> {
    fn call_trait<M>(self, b: MyStruct<T>, c: MyStruct<M>) where T: MyTrait, M:MyTrait{
        let _ = (b.x, c.x).call_trait();
    }
}

fn main() -> u64 {
    let s = MyStruct { x: 0u64 };
    s.call_trait(s, s);

    42
}
