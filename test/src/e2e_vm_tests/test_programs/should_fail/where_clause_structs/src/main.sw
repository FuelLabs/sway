library;

trait MyAdd {
    fn my_add(self, b: Self) -> Self;
}

impl MyAdd for u32 {
    fn my_add(self, b: u32) -> u32 {
        self + b
    }
}

struct MyPoint<T> where T: MyAdd {
    x: T,
    y: T,
}

struct Option<T> {
    x: T,
}

// Missing constraint T: MyAdd from impl self type
impl<T> Option<MyPoint<T>> {
    fn add(self, v: T) -> T {
        //self.x.my_add(v)
        v
    }
}

// Trait "MyAdd" is not implemented for type "Option<T>".
impl<T> MyPoint<Option<T>> {
    fn add(self, v: T) -> T {
        //self.x.my_add(v)
        v
    }
}

impl<T> Option<T> {
    // Missing constraint T: MyAdd from type param
    fn add2<G>(v: G, p: MyPoint<G>) -> G {
        v
    }
}

// Missing constraint T: MyAdd from parameter
fn add1<T>(point: MyPoint<T>, v: T) -> T {
    point.x.my_add(v);
    v
}

// Missing constraint T: MyAdd from return
fn add2<T>(v: T) -> MyPoint<T> {
    MyPoint {
        x: v,
        y: v,
    }
}

// Missing constraint T: MyAdd from variable
fn add3<T>(v: T) -> T {
    let p = MyPoint {
        x: v,
        y: v,
    };
    v
}

pub fn main() {
    let foo = MyPoint {
        x: 1u32,
        y: 1u32,
    };
    // Trait "MyAdd" is not implemented for type "u64".
    let bar = MyPoint {
        x: 3u64,
        y: 1u64,
    };
}
