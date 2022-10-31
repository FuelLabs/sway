script;

// We should definitely implement something like the "fully qualified syntax",
// but until then, multiple methods with the same name is undefined behavior.
// https://doc.rust-lang.org/rust-by-example/trait/disambiguating.html

dep my_double;
dep my_point;
dep my_triple;

use my_point::MyPoint;
use my_triple::MyTriple;

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

// impl<T> MyAdd<u8> for FooBarData<T> {
//     fn my_add(self, a: u8, b: u8) -> u8 {
//         a + b
//     }
// }

impl<T> MyAdd<u64> for FooBarData<T> {
    fn my_add(self, a: u64, b: u64) -> u64 {
        a + b
    }
}

trait MySub<T> {
    fn my_sub(a: T, b: T) -> T;
}

// impl<T> MySub<u8> for FooBarData<T> {
//     fn my_sub(a: u8, b: u8) -> u8 {
//         if a >= b {
//             a - b
//         } else {
//             b - a
//         }
//     }
// }

impl<T> MySub<u64> for FooBarData<T> {
    fn my_sub(a: u64, b: u64) -> u64 {
        if a >= b {
            a - b
        } else {
            b - a
        }
    }
}

struct OtherData<T> {
    a: T,
    b: T,
}

// impl<T> MyAdd<u8> for OtherData<T> {
//     fn my_add(self, a: u8, b: u8) -> u8 {
//         a + b
//     }
// }

impl<T> MyAdd<u64> for OtherData<T> {
    fn my_add(self, a: u64, b: u64) -> u64 {
        a + b
    }
}

// impl<T> MySub<u8> for OtherData<T> {
//     fn my_sub(a: u8, b: u8) -> u8 {
//         if a >= b {
//             a - b
//         } else {
//             b - a
//         }
//     }
// }

impl<T> MySub<u64> for OtherData<T> {
    fn my_sub(a: u64, b: u64) -> u64 {
        if a >= b {
            a - b
        } else {
            b - a
        }
    }
}

impl MyTriple<u64> for MyPoint<u64> {
    fn my_triple(self, value: u64) -> u64 {
        (self.x*3) + (self.y*3) + (value*3)
    }
}

struct MyU64 {
    inner: u64
}

impl MyTriple<u64> for MyU64 {
    fn my_triple(self, value: u64) -> u64 {
        (self.inner*3) + (value*3)
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
    let i = OtherData {
        a: true,
        b: false,
    };
    let j = OtherData {
        a: 10u32,
        b: 11u32,
    };
    let k = j.my_add(i.my_add(1u8, 2u8), i.my_add(3u8, 4u8));
    let l = ~FooBarData::<u16>::my_sub(
        ~FooBarData::<u32>::my_sub(100, 10),
        ~FooBarData::<u32>::my_sub(50, 10),
    );    
    let m = MyPoint {
        x: 10u64,
        y: 10u64,
    };
    let n = m.my_double(100);
    let o = m.my_triple(100);
    let p = MyU64 {
        inner: 30u64
    };
    let q = p.my_triple(1);

    if c == 42u8
        && d
        && e == 9u64
        && g == 10
        && h == 50
        && k == 10
        && l == 50
        && n == 240 
        && o == 360
        && q == 93 {
        42
    } else {
        7
    }
}
