library;

pub enum MyEnum1 {
    A: u64,
    B: u64,
}

pub type MyIdentity1 = MyEnum1;
pub type MyIdentity2 = MyIdentity1;