script;

dep my_add;
dep my_a;

impl my_add::MyAdd for u64 {
    fn my_add(self, other: Self) -> Self {
        other
    }
}

struct MyPoint<T> {
    x: T,
    y: T,
}

fn add_points<T>(a: MyPoint<T>, b: MyPoint<T>) -> MyPoint<T> where T: my_add::MyAdd {
    MyPoint {
        x: a.x.my_add(b.x),
        y: a.y.my_add(b.y),
    }
}

trait B: my_a::A {

} {

}

fn main() {

}
