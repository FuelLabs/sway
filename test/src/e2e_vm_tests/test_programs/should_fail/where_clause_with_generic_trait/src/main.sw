script;

trait Getter<T> {
    fn get(self) -> T;
}

struct MyU32 {
    value: u32
}

struct MyU64 {
    value: u64
}

impl Getter<u32> for MyU32 {
    fn get(self) -> u32 {
        self.value
    }
}

impl Getter<u64> for MyU64 {
    fn get(self) -> u64 {
        self.value
    }
}

struct MyPoint<T> where T: Getter<T> {
    x: T,
    y: T,
}

impl<T> Getter<T> for MyPoint<T> {
    fn get(self) -> T {
        self.x.get()
    }
}

fn main() -> u8 {
    let foo = MyPoint {
        x: 1u32,
        y: 2u64,
    };
    let bar = MyPoint {
        x: 3u32,
        y: 4u64,
    };
    0u8
}
