script;

trait Setter<T> {
    fn set(self, new_value: T) -> Self;
}

struct FooBarData<T> {
    value: T
}

impl<T> Setter<T> for FooBarData<T> {
    fn set(self, new_value: T) -> Self {
        FooBarData {
            value: new_value,
        }
    }
}

trait Returner<T> {
    fn return_it(self, the_value: T) -> T;
}

impl<T, F> Returner<T> for FooBarData<F> {
    fn return_it(self, the_value: T) -> T {
        the_value
    }
}

trait MyAdd<T> {
    fn my_add(self, a: T, b: T) -> T;
}

impl<T> MyAdd<u8> for FooBarData<T> {
    fn my_add(self, a: u8, b: u8) -> u8 {
        a + b
    }
}

impl<T> MyAdd<u64> for FooBarData<T> {
    fn my_add(self, a: u64, b: u64) -> u64 {
        a + b
    }
}

trait MySub<T> {
    fn my_sub(a: T, b: T) -> T;
}

impl<T> MySub<u8> for FooBarData<T> {
    fn my_sub(a: u8, b: u8) -> u8 {
        if a >= b {
            a - b
        } else {
            b - a
        }
    }
}

impl<T> MySub<u64> for FooBarData<T> {
    fn my_sub(a: u64, b: u64) -> u64 {
        if a >= b {
            a - b
        } else {
            b - a
        }
    }
}

fn main() -> u64 {
    let a = FooBarData {
        value: 1u8
    };
    let b = a.set(42);
    let c = b.value;
    let d = b.return_it(true);
    let e = b.return_it(9u64);
    let f = FooBarData {
        value: 1u64
    };
    let g = f.my_add(a.my_add(1u8, 2u8), a.my_add(3u8, 4u8));
    let h = ~FooBarData::<u64>::my_sub(
        ~FooBarData::<u8>::my_sub(100, 10),
        ~FooBarData::<u8>::my_sub(50, 10),
    );

    if c == 42u8 && d && e == 9u64 && g == 10 && h == 50 {
        42
    } else {
        7
    }
}
