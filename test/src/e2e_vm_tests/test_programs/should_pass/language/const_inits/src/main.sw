script;

const ETH_ID0: ContractId = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
fn contract_id_wrapper(b: b256) -> ContractId {
    ContractId::from(b)
}
const ETH_ID1: ContractId = contract_id_wrapper(0x0000000000000000000000000000000000000000000000000000000000000001);

// test if-expressions
fn bool_to_num(b: bool) -> u64 {
    if b {
        1
    } else {
        0
    }
}

// test variable shadowing and local const
fn const_42(x: u64) -> u64 {
    const forty_two: u64 = 42;
    let x: u64 = forty_two;
    x
}

// test variable scopes and local const
fn id(x: u64) -> u64 {
    const forty_two: u64 = 42;
    {
        let x: u64 = forty_two;
    };
    x
}

const QUUX: u64 = id(0);
const BAZ: u64 = const_42(123456);

const TUP1: (u64, u64, u64) = (2, 1, 21);
const ARR1: [u64; 3] = [1, 2, 3];

fn tup_wrapper(a: u64, b: u64, c: u64) -> (u64, u64, u64) {
    (a, b, c)
}
const TUP2: (u64, u64, u64) = tup_wrapper(2, 1, 21);

fn arr_wrapper(a: u64, b: u64, c: u64) -> [u64; 3] {
    [a, b, c]
}

const ARR2: [u64; 3] = arr_wrapper(1, 2, 3);

enum En1 {
    Int: u64,
    Arr: [u64; 3],
    NoVal: (),
}

const X_SIZE: u64 = 4;
const Y_SIZE: u64 = 2;
const XPY: u64 = ((X_SIZE + Y_SIZE - 1) * 2) / 5;
const EN0A: En1 = En1::Int(XPY);
const TRUEB: bool = X_SIZE == 4;
const FALSEB: bool = X_SIZE == Y_SIZE;
const TRUEB1: bool = X_SIZE > Y_SIZE;
const FALSEB1: bool = X_SIZE < Y_SIZE;

const SO: u64 = __size_of::<u64>();
const SOV: u64 = __size_of_val("hello");

const EN1a: En1 = En1::Int(101);
const EN1b: En1 = En1::Arr(ARR2);
const EN1c: En1 = En1::NoVal;

const ETH_ID0_VALUE: b256 = ETH_ID0.bits();
const TUP1_idx2: u64 = TUP1.2;

const INT1: u64 = 1;

// b256
const ZERO_B256: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const ONE_B256: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const KEY: b256 = ZERO_B256;
const BITWISE_B256: b256 = !ZERO_B256 & ZERO_B256 | ZERO_B256 ^ ZERO_B256;
const SHIFT_B256: b256 = ZERO_B256 >> 1 << 1;

const BAR: u32 = 6;
const FOO: u32 = ((u32::min() + 1) * 12 / 2 - 1) % 6;
const MASK: u32 = 11;
const MASK2: u32 = 8;
const MASK3: u32 = 15;
const FOO_MIDDLE: u32 = ((FOO & MASK) | MASK2) ^ MASK3;
const OPS: u64 = 10 + 9 - 8 * 7 / 6 << 5 >> 4 ^ 3 | 2 & bool_to_num(true);

const CARR1: [u64; 4] = [X_SIZE - Y_SIZE + 1; 4];
// This doesn't work because const-eval happens after type-checking,
// and the type checker needs to know the size of the array.
// const CARR2 = [1; X_SIZE - Y_SIZE + 1];

// Const init with Self
struct WithSelf { value: u64 }
impl WithSelf {
    pub fn size() -> u64 {
        __transmute::<Self, u64>(Self { value: 1u64 })
    }
}
const WithSelfValue: u64 =  WithSelf::size();

fn main() -> u64 {
    const int1: u64 = 1;
    assert(int1 == INT1 && ZERO_B256 == KEY);

    // initialization through function applications.
    const eth_id0: ContractId = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    const eth_id1: ContractId = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(eth_id0 == ETH_ID0 && eth_id1 == ETH_ID1);
    assert(BAZ == 42);
    assert(QUUX == 0);

    // tuples and arrays.
    const t1: (u64, u64, u64) = (2, 1, 21);
    assert(t1.0 == TUP1.0 && t1.1 == TUP1.1 && t1.2 == TUP1.2);
    assert(t1.0 == TUP2.0 && t1.1 == TUP2.1 && t1.2 == TUP2.2);
    const a1: [u64; 3] = [1, 2, 3];
    assert(a1[0] == ARR1[0] && a1[1] == ARR1[1] && a1[2] == ARR1[2]);
    assert(a1[0] == ARR2[0] && a1[1] == ARR2[1] && a1[2] == ARR2[2]);
    assert(
        CARR1[0] == X_SIZE - Y_SIZE + 1 &&
        CARR1[1] == X_SIZE - Y_SIZE + 1 &&
        CARR1[2] == X_SIZE - Y_SIZE + 1 &&
        CARR1[3] == X_SIZE - Y_SIZE + 1
    );

    // enum
    match EN0A {
        En1::Int(i) => assert(i == 2),
        En1::Arr(_) => assert(false),
        En1::NoVal => assert(false),
    }
    match EN1a {
        En1::Int(i) => assert(i == 101),
        En1::Arr(_) => assert(false),
        En1::NoVal => assert(false),
    }
    match EN1b {
        En1::Int(_i) => assert(false),
        En1::Arr(arr) => {
            assert(arr[0] == ARR1[0] && arr[1] == ARR1[1] && arr[2] == ARR1[2]);
        }
        En1::NoVal => assert(false),
    }
    match EN1c {
        En1::Int(_i) => assert(false),
        En1::Arr(_) => assert(false),
        En1::NoVal => assert(true),
    }

    // Struct and enum field access.
    assert(ETH_ID0.bits() == ETH_ID0_VALUE);
    assert(TUP1_idx2 == TUP1.2);
    assert(XPY == 2);
    assert(SO == __size_of::<u64>());
    assert(SOV == __size_of_val("hello"));
    assert(TRUEB != FALSEB);
    assert(TRUEB1 != FALSEB1);
    assert(FOO == 5);
    assert(FOO_MIDDLE == BAR);
    assert(OPS == 23);

    test_not();

    assert(WithSelfValue == 1);

    1
}

const NOTA: u8 = !0u8;
const NOTB: u16 = !0u16;
const NOTC: u32 = !0u32;
const NOTD: u64 = !0u64;
const NOTE: bool = !false;

fn test_not() {
    assert(NOTA == 0xFFu8);
    assert(NOTB == 0xFFFFu16);
    assert(NOTC == 0xFFFFFFFFu32);
    assert(NOTD == 0xFFFFFFFFFFFFFFFFu64);
    assert(NOTE == true);
}
