script;

trait MyAdd<T> {
    fn my_add(self, a: T, b: T) -> T;
}

struct Struct<A> where A: MyAdd<A> {
    data: A,
}

struct Struct2<A, B> where A: MyAdd<B>, B: MyAdd<A> {
    data_a: A,
    data_b: B,
}

impl MyAdd<u64> for u64 {
    fn my_add(self, a: u64, b: u64) -> u64 {
        a + b
    }
}

pub trait MyFrom<T> {
    fn from(b: T) -> Self;
}


pub trait MyInto<T> {
    fn my_into(self) -> T;
}


impl<T, U> MyInto<U> for T
where
    U: MyFrom<T>,
{
    fn my_into(self) -> U {
        U::from(self)
    }
}

struct Struct3 {
    data: u64,
}

impl MyFrom<u64> for Struct3 {
    fn from(i: u64) -> Struct3 {
        Struct3 {data: i}
    }
}

struct Struct4 {
    data: u64,
}

impl MyFrom<Struct4> for Struct3 {
    fn from(i: Struct4) -> Struct3 {
        Struct3 {data: i.data}
    }
}

// call an associated function through generic constraints
pub trait SizeInBytes {
    fn size() -> u64;
}

impl SizeInBytes for u64 {
    fn size() -> u64 {
        8
    }
}

fn call_size<T>() -> u64 where T: SizeInBytes {
    T::size()
}

fn main() -> bool {
    let s1 = Struct {data: 1_u64 };
    assert_eq(s1.data.my_add(1,2),3);

    let s2 = Struct2 {data_a: 1_u64, data_b: 1_u64 };
    assert_eq(s2.data_a.my_add(1,2),3);
    assert_eq(s2.data_b.my_add(1,2),3);

    let s3: Struct3 = 42_u64.my_into();
    assert_eq(s3.data,42);

    let s4: Struct3 = Struct4{data:42}.my_into();
    assert_eq(s4.data,42);

    // call an associated function through generic constraints
    assert_eq(call_size::<u64>(), 8);

    true
}