library;

pub struct Data<T> {
    value: T,
}

impl<T> Data<T> {
    pub fn new(v: T) -> Self {
        Data { value: v }
    }

    pub fn get_value(self) -> T {
        self.value
    }
}

pub struct DoubleIdentity<T, F> {
    first: T,
    second: F,
    third: u64,
}

impl<T, F> DoubleIdentity<T, F> {
    pub fn new(x: T, y: F) -> DoubleIdentity<T, F> {
        DoubleIdentity {
            first: x,
            second: y,
            third: 10u64,
        }
    }

    pub fn get_first(self) -> T {
        let x: T = self.first;
        x
    }

    pub fn get_second(self) -> F {
        let y: F = self.second;
        y
    }

    pub fn get_third(self) -> u64 {
        let z: u64 = self.third;
        z
    }
}

impl DoubleIdentity<u8, u8> {
    pub fn add(self) -> u8 {
        self.first + self.second
    }
}

pub fn double_identity2<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
    DoubleIdentity::<T, F>::new(x, y)
}

pub fn double_identity<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
    let inner: T = x;
    DoubleIdentity {
        first: inner,
        second: y,
        third: 20u64,
    }
}

pub fn crazy<T, F>(x: T, y: F) -> F {
    let foo = DoubleIdentity {
        first: x,
        second: y,
        third: 30u64,
    };
    foo.get_second()
}

pub enum MyResult<T> {
    Ok: T,
    Err: u8, // err code
}

impl<T> MyResult<T> {
    pub fn ok(value: T) -> Self {
        MyResult::Ok::<T>(value)
    }

    pub fn err(code: u8) -> Self {
        MyResult::Err::<T>(code)
    }
}

pub enum MyOption<T> {
    Some: T,
    None: (),
}

impl<T> MyOption<T> {
    pub fn some(value: T) -> Self {
        MyOption::Some::<T>(value)
    }

    pub fn none() -> Self {
        MyOption::None::<T>
    }

    pub fn to_result(self) -> MyResult<T> {
        if let MyOption::Some(value) = self {
            MyResult::<T>::ok(value)
        } else {
            MyResult::<T>::err(99u8)
        }
    }

    pub fn is_some(self) -> bool {
        match self {
            MyOption::Some(_) => true,
            MyOption::None => false,
        }
    }

    pub fn is_none(self) -> bool {
        match self {
            MyOption::Some(_) => false,
            MyOption::None => true,
        }
    }
}

impl<T, E> Result<T, E> {
    pub fn dummy(t: T) -> Result<T, bool> {
        Result::Ok(t)
    }
}
