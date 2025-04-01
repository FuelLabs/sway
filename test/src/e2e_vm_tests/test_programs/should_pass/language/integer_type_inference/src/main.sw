script;

/* Test Constants */
const X1: u8 = 4u8;
const X2: u8 = 4;

const Y1: u16 = 4u16;
const Y2: u16 = 4;

const Z1: u32 = 4u32;
const Z2: u32 = 4;

const W1: u64 = 4u64;
const W2: u64 = 4;

const V1: u8 = 4u8;
const V2: u16 = 4u16;
const V3: u32 = 4u32;
const V4: u64 = 4u64;
const V5: u64 = 4;

/* Traits Specific to Individual Integer Types */
trait FooU8 {
    fn foo_u8(self);
}

trait FooU16 {
    fn foo_u16(self);
}

trait FooU32 {
    fn foo_u32(self);
}

trait FooU64 {
    fn foo_u64(self);
}

/* Trait Impls */
impl FooU8 for u8 {
    fn foo_u8(self) {}
}

impl FooU16 for u16 {
    fn foo_u16(self) {}
}

impl FooU32 for u32 {
    fn foo_u32(self) {}
}

impl FooU64 for u64 {
    fn foo_u64(self) {}
}

use std::option::Option as OptionAlias;

fn main() {
    /* Make sure that the resulting types of constants are correct */
    X1.foo_u8();
    X2.foo_u8();

    Y1.foo_u16();
    Y2.foo_u16();

    Z1.foo_u32();
    Z2.foo_u32();

    W1.foo_u64();
    W2.foo_u64();

    V1.foo_u8();
    V2.foo_u16();
    V3.foo_u32();
    V4.foo_u64();
    V5.foo_u64();

    /* Make sure that the resulting types of variables are correct */
    let x1: u8 = 4u8;
    let x2: u8 = 4u16.try_as_u8().unwrap();
    let x3: u8 = 4u32.try_as_u8().unwrap();
    let x4: u8 = 4u64.try_as_u8().unwrap();
    let x5: u8 = 4;
    let x6: Option<u8> = Option::Some(1);
    let x7: Option<u8> = OptionAlias::Some(1);
    let x8: OptionAlias<u8> = Option::Some(1);

    let y1: u16 = 4u8.as_u16();
    let y2: u16 = 4u16;
    let y3: u16 = 4u32.try_as_u16().unwrap();
    let y4: u16 = 4u64.try_as_u16().unwrap();
    let y5: u16 = 4;
    let y6: Option<u16> = Option::Some(1);

    let z1: u32 = 4u8.as_u32();
    let z2: u32 = 4u16.as_u32();
    let z3: u32 = 4u32;
    let z4: u32 = 4u64.try_as_u32().unwrap();
    let z5: u32 = 4;
    let z6: Option<u32> = Option::Some(1);

    let w1: u64 = 4u8.as_u64();
    let w2: u64 = 4u16.as_u64();
    let w3: u64 = 4u32.as_u64();
    let w4: u64 = 4u64;
    let w5: u64 = 4;
    let w6: Option<u64> = Option::Some(1);

    let v1 = 4u8;
    let v2 = 4u16;
    let v3 = 4u32;
    let v4 = 4u64;
    let v5 = 4;

    x1.foo_u8();
    x2.foo_u8();
    x3.foo_u8();
    x4.foo_u8();
    x5.foo_u8();
    x6.unwrap().foo_u8();
    x7.unwrap().foo_u8();
    x8.unwrap().foo_u8();

    y1.foo_u16();
    y2.foo_u16();
    y3.foo_u16();
    y4.foo_u16();
    y5.foo_u16();
    y6.unwrap().foo_u16();

    z1.foo_u32();
    z2.foo_u32();
    z3.foo_u32();
    z4.foo_u32();
    z5.foo_u32();
    z6.unwrap().foo_u32();

    w1.foo_u64();
    w2.foo_u64();
    w3.foo_u64();
    w4.foo_u64();
    w5.foo_u64();
    w6.unwrap().foo_u64();

    v1.foo_u8();
    v2.foo_u16();
    v3.foo_u32();
    v4.foo_u64();
    v5.foo_u64();
}
