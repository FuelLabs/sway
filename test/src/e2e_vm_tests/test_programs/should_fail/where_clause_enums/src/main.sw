library;

trait MyAdd {
    fn my_add(self, b: Self) -> Self;
}

impl MyAdd for u32 {
    fn my_add(self, b: u32) -> u32 {
        self
    }
}

enum MyEnum<T> where T: MyAdd {
    X: T,
}

struct Option<T> {
    x: T,
}

// Missing constraint T: MyAdd from impl self type
impl<T> Option<MyEnum<T>> {
    fn add(self, v: T) -> T {
        v
    }
}

// Trait "MyAdd" is not implemented for type "Option<T>".
impl<T> MyEnum<Option<T>> {
    fn add(self, v: T) -> T {
        v
    }
}

impl<T> Option<T> {
    // Missing constraint T: MyAdd from type param
    fn add2<G>(v: G, p: MyEnum<G>) -> G {
        v
    }
}

// Missing constraint T: MyAdd from parameter
fn add1<T>(e: MyEnum<T>, v: T) -> T {
    e.my_add(v);
}

// Missing constraint T: MyAdd from return
fn add2<T>(v: T) -> MyEnum<T> {
    MyEnum::X(v)
}

// Missing constraint T: MyAdd from variable
fn add3<T>(v: T) -> T {
    let p = MyEnum::X(v);
    v
}

pub fn main() {
    let foo = MyEnum::X(1u32);

    // Trait "MyAdd" is not implemented for type "u64".
    let bar = MyEnum::X(3u64);
}
