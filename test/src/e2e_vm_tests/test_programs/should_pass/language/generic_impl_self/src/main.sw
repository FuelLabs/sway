script;

use std::u128::*;

struct Data<T> {
    value: T,
}

impl<T> Data<T> {
    fn new(v: T) -> Self {
        Data { value: v }
    }

    fn get_value(self) -> T {
        self.value
    }
}

struct DoubleIdentity<T, F> {
    first: T,
    second: F,
    third: u64,
}

impl<T, F> DoubleIdentity<T, F> {
    fn new(x: T, y: F) -> DoubleIdentity<T, F> {
        DoubleIdentity {
            first: x,
            second: y,
            third: 10u64,
        }
    }

    fn get_first(self) -> T {
        let x: T = self.first;
        x
    }

    fn get_second(self) -> F {
        let y: F = self.second;
        y
    }

    fn get_third(self) -> u64 {
        let z: u64 = self.third;
        z
    }
}

impl DoubleIdentity<u8, u8> {
    fn add(self) -> u8 {
        self.first + self.second
    }
}

fn double_identity2<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
    DoubleIdentity::<T, F>::new(x, y)
}

fn double_identity<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
    let inner: T = x;
    DoubleIdentity {
        first: inner,
        second: y,
        third: 20u64,
    }
}

fn crazy<T, F>(x: T, y: F) -> F {
    let foo = DoubleIdentity {
        first: x,
        second: y,
        third: 30u64,
    };
    foo.get_second()
}

enum MyResult<T> {
    Ok: T,
    Err: u8, // err code
}

impl<T> MyResult<T> {
    fn ok(value: T) -> Self {
        MyResult::Ok::<T>(value)
    }

    fn err(code: u8) -> Self {
        MyResult::Err::<T>(code)
    }
}

enum MyOption<T> {
    Some: T,
    None: (),
}

impl<T> MyOption<T> {
    fn some(value: T) -> Self {
        MyOption::Some::<T>(value)
    }

    fn none() -> Self {
        MyOption::None::<T>
    }

    fn to_result(self) -> MyResult<T> {
        if let MyOption::Some(value) = self {
            MyResult::<T>::ok(value)
        } else {
            MyResult::<T>::err(99u8)
        }
    }

    fn is_some(self) -> bool {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }

    fn is_none(self) -> bool {
        match self {
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }
}

pub struct MyResult2<T, E> {
    res: Result<T, E>
}

impl<T, E> MyResult2<T, E> {
    fn dummy(t: T) -> MyResult2<T, bool> {
        MyResult2 { res: Ok(t) }
    }
}

fn result_impl_test() {
    let res = U128::from((0, 13)).as_u64();
    assert(!MyResult2::dummy(false).res.unwrap());
    assert(res.unwrap_or(5) == 13);
}

fn generic_impl_self_test() {
    let a = double_identity(true, true);
    assert(a.first);
    assert(a.second);

    let b = double_identity(10u32, 43u64);
    assert(b.first == 10u32);
    assert(b.second == 43u64);

    let c = double_identity2(10u8, 1u8);
    assert(c.first == 10u8);
    assert(c.second == 1u8);

    let d = DoubleIdentity {
        first: 1u8,
        second: 2u8,
        third: 40u64,
    };
    assert(d.third == 40u64);

    let e = d.get_second();
    assert(e == 2u8);

    let f: DoubleIdentity<bool, bool> = double_identity(true, true);
    assert(f.first && f.second);

    let g: DoubleIdentity<u32, u64> = double_identity(10u32, 43u64);
    assert((g.first + 33u32).as_u64() == g.second);

    let h = DoubleIdentity::<u64, bool>::new(3u64, false);
    assert(!h.second);

    let i = crazy(7u8, 10u8);
    assert(i == 10u8);

    let k = d.add();
    assert(k == 3u8);

    let l = Data::<bool>::new(false);
    assert(!l.value);

    let m: DoubleIdentity<Data<u8>, Data<u64>> = DoubleIdentity {
        first: Data { value: 1u8 },
        second: Data { value: 2u64 },
        third: 1u64,
    };
    assert(m.second.value == (m.first.value.as_u64() + m.third));

    let n = DoubleIdentity::<Data<u8>, Data<u8>>::new(Data::<u8>::new(3u8), Data::<u8>::new(4u8));
    assert(n.third == 10u64);

    let o: DoubleIdentity<bool, bool> = double_identity(true, true);
    assert(o.first && o.second);

    let p = MyOption::Some::<bool>(false);
    assert(p.is_some());

    let q = MyOption::Some::<()>(());
    assert(q.is_some());

    let r = MyOption::<u32>::some(5u32);
    assert(r.is_some());

    let s = MyOption::Some(0u8);
    assert(s.is_some());

    let t = MyOption::<u64>::none();
    assert(t.is_none());

    let u = DoubleIdentity::<Data<u8>, Data<u8>>::new(Data::<u8>::new(3u8), Data::<u8>::new(4u8));
    assert(u.first.value + u.second.value == 7u8);
}

use std::vec::*;

struct MyVec<T> {
    vec: Vec<T>
}

impl<T> MyVec<T> {
    pub fn new() -> Self { MyVec{ vec: Vec::new() } }

    pub fn with(ref mut self, with_value: T) -> Self {
        self.vec.push(with_value);
        self
    }
}

fn main() -> u32 {
    generic_impl_self_test();
    result_impl_test();

    // data must be Vec<u256>
    let data = MyVec::new().with(0x333u256).with(0x222u256);
    assert(data.vec.len() == 2);

    10u32
}
