script;

pub fn as_raw_ptr<T>(val: &T) -> raw_ptr {
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

const A_U256: u256 = 0x0;
const B_U256: u256 = 0x1;

const A_U32: u32 = 0;
const B_U32: u32 = 1;

const A_BOOL: bool = true;
const B_BOOL: bool = false;

const A_U64: u64 = 0;
const B_U64: u64 = 1;

const A_X: X = X::A(0);
const B_X: X = X::B(1);

const A_Y: Y = Y { a: 0, b: 1 };
const B_Y: Y = Y { a: 1, b: 2 };

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
    assert (!addr_addr_of.is_null());
    assert(__addr_of(__addr_of(x)) != __addr_of(x));

    let some_temp = __addr_of(1 + 4);
    assert(!some_temp.is_null());

    let raw_ptr = __addr_of(x);
    assert(!raw_ptr.is_null());

    const X1: u32 = 42;
    const Y1: u32 = 43;
    assert(__addr_of(X1) == as_raw_ptr(&X1));
    assert(__addr_of(X1) != as_raw_ptr(&Y1)); 
    assert(__addr_of(X1) != __addr_of(Y1));

    const X2: u64 = 42;
    const Y2: u64 = 43;
    assert(__addr_of(X2) == as_raw_ptr(&X2)); 
    assert(__addr_of(X2) != as_raw_ptr(&Y2));
    assert(__addr_of(X2) != __addr_of(Y2));

    const X3: u256 = 42u256;
    const Y3: u256 = 43u256;
    assert(__addr_of(X3) == as_raw_ptr(&X3));
    assert(__addr_of(X3) != as_raw_ptr(&Y3));
    assert(__addr_of(X3) != __addr_of(Y3));
    
    const X4: bool = true;
    const Y4: bool = false;
    assert(__addr_of(X4) == as_raw_ptr(&X4));
    assert(__addr_of(X4) != as_raw_ptr(&Y4));
    assert(__addr_of(X4) != __addr_of(Y4));

    const X5: X = X::A(42);
    const Y5: X = X::B(43);
    assert(__addr_of(X5) == as_raw_ptr(&X5));
    assert(__addr_of(X5) != as_raw_ptr(&Y5));
    assert(__addr_of(X5) != __addr_of(Y5));

    const X6: Y = Y { a: 42, b: 43 };
    const Y6: Y = Y { a: 44, b: 45 };
    assert(__addr_of(X6) == as_raw_ptr(&X6));
    assert(__addr_of(X6) != as_raw_ptr(&Y6));
    assert(__addr_of(X6) != __addr_of(Y6));

    const X7: [u32; 3] = [1, 2, 3];
    const Y7: [u32; 3] = [4, 5, 6];
    assert(__addr_of(X7) == as_raw_ptr(&X7));
    assert(__addr_of(X7) != as_raw_ptr(&Y7));

    const X8: (u32, u32) = (1, 2);
    const Y8: (u32, u32) = (3, 4);
    assert(__addr_of(X8) == as_raw_ptr(&X8));
    assert(__addr_of(X8) != as_raw_ptr(&Y8));

    const X10: u8 = 8;
    const Y10: u8 = 9;
    assert(__addr_of(X10) == as_raw_ptr(&X10));
    assert(__addr_of(X10) != as_raw_ptr(&Y10));

    assert(__addr_of(A_U256) == as_raw_ptr(&A_U256));
    assert(__addr_of(B_U256) == as_raw_ptr(&B_U256));
    assert(__addr_of(A_U256) != __addr_of(B_U256));

    assert(__addr_of(A_U32) == as_raw_ptr(&A_U32));
    assert(__addr_of(B_U32) == as_raw_ptr(&B_U32));
    assert(__addr_of(A_U32) != __addr_of(B_U32));
    assert(__addr_of(A_U32) != __addr_of(A_U256));

    assert(__addr_of(A_BOOL) == as_raw_ptr(&A_BOOL));
    assert(__addr_of(B_BOOL) == as_raw_ptr(&B_BOOL));
    assert(__addr_of(A_BOOL) != __addr_of(B_BOOL));

    assert(__addr_of(A_U64) == as_raw_ptr(&A_U64));
    assert(__addr_of(B_U64) == as_raw_ptr(&B_U64));
    assert(__addr_of(A_U64) != __addr_of(B_U64));

    assert(__addr_of(A_X) == as_raw_ptr(&A_X));
    assert(__addr_of(B_X) == as_raw_ptr(&B_X));
    assert(__addr_of(A_X) != __addr_of(B_X));

    assert(__addr_of(A_Y) == as_raw_ptr(&A_Y));
    assert(__addr_of(B_Y) == as_raw_ptr(&B_Y));
    assert(__addr_of(A_Y) != __addr_of(B_Y));
}
