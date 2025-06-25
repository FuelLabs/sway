script;

pub fn as_raw_ptr<T>(val: T) -> raw_ptr {
    asm(ptr: val) {
        ptr: raw_ptr
    }
}

enum X {
     A: u32,
     B: u64,
}

struct Y {
    a: u32,
    b: u64,
}

fn main() {
    let x = X::A(2);
    let y = X::B(22);
    assert(__addr_of(x) == as_raw_ptr(&x));
    assert(__addr_of(x) != as_raw_ptr(&y));

    let y = Y { a: 2, b: 22 };
    let z = Y { a: 3, b: 23 };
    assert(__addr_of(y) == as_raw_ptr(&y));
    assert(__addr_of(y) != as_raw_ptr(&z));

    let addr_y_a = __addr_of(y.a);
    let addr_y_b = __addr_of(y.b);
    assert(addr_y_a == as_raw_ptr(&y.a));
    assert(addr_y_b == as_raw_ptr(&y.b));
    assert(addr_y_a != addr_y_b);

    let a = [1,2,3];
    assert(__addr_of(a) == as_raw_ptr(&a));

    let b = "hello";
    assert(__addr_of(b) == as_raw_ptr(&b));

    let c = (1, 2);
    assert(__addr_of(c) == as_raw_ptr(&c));

    let i1 = 42u64;
    let i2 = 43u64;
    assert(__addr_of(i1) == as_raw_ptr(&i1));
    assert(__addr_of(i1) != as_raw_ptr(&i2));

    let b1 = true;
    let b2 = false;
    assert(__addr_of(b1) == as_raw_ptr(&b1));
    assert(__addr_of(b1) != as_raw_ptr(&b2));

    let u8_1 = 8u8;
    let u8_2 = 9u8;
    assert(__addr_of(u8_1) == as_raw_ptr(&u8_1));
    assert(__addr_of(u8_1) != as_raw_ptr(&u8_2));

    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000001u256;
    let u256_2: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    assert(__addr_of(u256_1) == as_raw_ptr(&u256_1));
    assert(__addr_of(u256_1) != as_raw_ptr(&u256_2));

    let addr_addr_of = __addr_of(__addr_of(x));
    assert (addr_addr_of != as_raw_ptr(0));

    let some_temp = __addr_of(1 + 4);
    assert(some_temp != as_raw_ptr(0));

    let raw_ptr = __addr_of(x);
    assert(raw_ptr != as_raw_ptr(0));
}
