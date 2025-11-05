contract;

use std::hash::*;

pub struct S {
    x: u64,
    y: u64,
    z: b256,
    u: u256,
    t: T,
    f_int8: F,
    f_int64: F,
    f_tuple: F,
}

pub struct T {
    x: u64,
    y: u64,
    z: b256,
    boolean: bool,
    int8: u8,
    int16: u16,
    int32: u32,
}

pub enum E {
    A: u64,
    B: T,
    Int8: u8,
    Int16: u16,
    Int32: u32,
    Bool: bool,
    Unit: (),
    Enum: F,
}

pub enum F {
    Int8: u8,
    Int64: u64,
    Tuple: (u8, u16, u32, u64, bool),
}

impl F {
    // Getting error that method named "eq" is not found for type "F"
    // when trying to use "F::eq" in the "eq" impls of other enums and structs.
    // That's why extracting of the "F::eq" logic here.
    pub fn equals(self, other: Self) -> bool {
        match (self, other) {
            (F::Int8(l), F::Int8(r)) => l == r,
            (F::Int64(l), F::Int64(r)) => l == r,
            (F::Tuple(l), F::Tuple(r)) => l.0 == r.0 && l.1 == r.1 && l.2 == r.2 && l.3 == r.3 && l.4 == r.4,
            _ => false,
        }
    }
}

impl PartialEq for T {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.boolean == other.boolean && self.int8 == other.int8 && self.int16 == other.int16 && self.int32 == other.int32
    }
}
impl Eq for T {}

impl PartialEq for S {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.u == other.u && self.t == other.t && self.f_int8.equals(other.f_int8) && self.f_int64.equals(other.f_int64) && self.f_tuple.equals(other.f_tuple)
    }
}
impl Eq for S {}

impl PartialEq for F {
    fn eq(self, other: Self) -> bool {
        self.equals(other)
    }
}
impl Eq for F {}

impl PartialEq for E {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (E::A(l), E::A(r)) => l == r,
            (E::B(l), E::B(r)) => l == r,
            (E::Int8(l), E::Int8(r)) => l == r,
            (E::Int16(l), E::Int16(r)) => l == r,
            (E::Int32(l), E::Int32(r)) => l == r,
            (E::Bool(l), E::Bool(r)) => l == r,
            (E::Unit, E::Unit) => true,
            (E::Enum(l), E::Enum(r)) => l.equals(r),
            _ => false,
        }
    }
}
impl Eq for E {}

storage {
    x: u64 = 64,
    y: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101,
    u: u256 = 0x0101010101010101010101010101010101010101010101010101010101010101u256,
    s: S = S {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        u: 0x0000000000000000000000000000000000000000000000000000000000000003u256,
        t: T {
            x: 4,
            y: 5,
            z: 0x0000000000000000000000000000000000000000000000000000000000000006,
            boolean: true,
            int8: 7,
            int16: 8,
            int32: 9,
        },
        f_int8: F::Int8(171),
        f_int64: F::Int64(123456789),
        f_tuple: F::Tuple((121, 11223, 12345678, 123456789, true)),
    },
    boolean: bool = true,
    int8: u8 = 8,
    int16: u16 = 16,
    int32: u32 = 32,
    e_a: E = E::A(777),
    e_b: E = E::B(T {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        boolean: true,
        int8: 4,
        int16: 5,
        int32: 6,
    }),
    e_int8: E = E::Int8(171),
    e_int16: E = E::Int16(12345),
    e_int32: E = E::Int32(123456789),
    e_bool: E = E::Bool(true),
    e_unit: E = E::Unit,
    e_enum_int8: E = E::Enum(F::Int8(123)),
    e_enum_int64: E = E::Enum(F::Int64(12345678)),
    e_enum_tuple: E = E::Enum(F::Tuple((121, 11223, 12345678, 123456789, true))),
    string: str[40] = __to_str_array("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
}

abi ExperimentalStorageInitTest {
    #[storage(read)]
    fn test_initializers() -> bool;
}

impl ExperimentalStorageInitTest for Contract {
    #[storage(read)]
    fn test_initializers() -> bool { /* Initializer values */
        let x: u64 = 64;
        let y: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101;
        let u: u256 = 0x0101010101010101010101010101010101010101010101010101010101010101u256;
        let s: S = S {
            x: 1,
            y: 2,
            z: 0x0000000000000000000000000000000000000000000000000000000000000003,
            u: 0x0000000000000000000000000000000000000000000000000000000000000003u256,
            t: T {
                x: 4,
                y: 5,
                z: 0x0000000000000000000000000000000000000000000000000000000000000006,
                boolean: true,
                int8: 7,
                int16: 8,
                int32: 9,
            },
            f_int8: F::Int8(171),
            f_int64: F::Int64(123456789),
            f_tuple: F::Tuple((121, 11223, 12345678, 123456789, true)),
        };
        let boolean: bool = true;
        let int8: u8 = 8;
        let int16: u16 = 16;
        let int32: u32 = 32;
        let e_a: E = E::A(777);
        let e_b: E = E::B(T {
            x: 1,
            y: 2,
            z: 0x0000000000000000000000000000000000000000000000000000000000000003,
            boolean: true,
            int8: 4,
            int16: 5,
            int32: 6,
        });
        let e_int8: E = E::Int8(171);
        let e_int16: E = E::Int16(12345);
        let e_int32: E = E::Int32(123456789);
        let e_bool: E = E::Bool(true);
        let e_unit: E = E::Unit;
        let e_enum_int8: E = E::Enum(F::Int8(123));
        let e_enum_int64: E = E::Enum(F::Int64(12345678));
        let e_enum_tuple: E = E::Enum(F::Tuple((121, 11223, 12345678, 123456789, true)));
        let string: str[40] = __to_str_array("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");

        assert(storage.x.read() == x);
        assert(storage.y.read() == y);
        assert(storage.u.read() == u);
        assert(storage.s.read() == s);
        assert(storage.boolean.read() == boolean);
        assert(storage.int8.read() == int8);

        assert(storage.int16.read() == int16);
        assert(storage.int32.read() == int32);
        assert(storage.s.x.read() == s.x);
        assert(storage.s.y.read() == s.y);
        assert(storage.s.z.read() == s.z);
        assert(storage.s.t.read() == s.t);
        assert(storage.s.t.x.read() == s.t.x);
        assert(storage.s.t.y.read() == s.t.y);
        assert(storage.s.t.z.read() == s.t.z);
        assert(storage.s.t.boolean.read() == s.t.boolean);
        assert(storage.s.t.int8.read() == s.t.int8);
        assert(storage.s.t.int16.read() == s.t.int16);
        assert(storage.s.t.int32.read() == s.t.int32);

        assert(storage.e_a.read() == e_a);
        assert(storage.e_b.read() == e_b);
        assert(storage.e_int8.read() == e_int8);
        assert(storage.e_int16.read() == e_int16);
        assert(storage.e_int32.read() == e_int32);
        assert(storage.e_bool.read() == e_bool);
        assert(storage.e_unit.read() == e_unit);
        assert(storage.e_enum_int8.read() == e_enum_int8);
        assert(storage.e_enum_int64.read() == e_enum_int64);
        assert(storage.e_enum_tuple.read() == e_enum_tuple);

        assert(sha256_str_array(storage.string.read()) == sha256_str_array(string));
        true
    }
}
