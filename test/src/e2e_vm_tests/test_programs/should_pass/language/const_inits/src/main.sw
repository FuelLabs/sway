script;

const ETH_ID0 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
fn contract_id_wrapper(b: b256) -> ContractId {
    ContractId::from(b)
}
const ETH_ID1 = contract_id_wrapper(0x0000000000000000000000000000000000000000000000000000000000000001);

const TUP1 = (2, 1, 21);
const ARR1 = [1, 2, 3];

fn tup_wrapper(a: u64, b: u64, c: u64) -> (u64, u64, u64) {
    (a, b, c)
}
const TUP2 = tup_wrapper(2, 1, 21);

fn arr_wrapper(a: u64, b: u64, c: u64) -> [u64; 3] {
    return [a, b, c];
}
const ARR2 = arr_wrapper(1, 2, 3);

enum En1 {
    Int: u64,
    Arr: [u64; 3],
    NoVal: (),
}

const X_SIZE: u64 = 4;
const Y_SIZE: u64 = 2;
const XPY = ((X_SIZE + Y_SIZE - 1) * 2) / 5;
const EN0A = En1::Int(XPY);
const TRUEB: bool = X_SIZE == 4;
const FALSEB: bool = X_SIZE == Y_SIZE;
const TRUEB1: bool = X_SIZE > Y_SIZE;
const FALSEB1: bool = X_SIZE < Y_SIZE;

const SO = __size_of::<u64>();
const SOV = __size_of_val("hello");

const EN1a = En1::Int(101);
const EN1b = En1::Arr(ARR2);
const EN1c = En1::NoVal;

const ETH_ID0_VALUE = ETH_ID0.value;
const TUP1_idx2 = TUP1.2;

const INT1 = 1;

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const KEY = ZERO_B256;

const BAR: u32 = 6;
const FOO: u32 = 5;
const MASK: u32 = 11;
const MASK2: u32 = 8;
const MASK3: u32 = 15;
const FOO_MIDDLE: u32 = ((FOO & MASK) | MASK2) ^ MASK3;
const OPS: u64 = 10 + 9 - 8 * 7 / 6 << 5 >> 4 ^ 3 | 2 & 1;

const CARR1 = [X_SIZE - Y_SIZE + 1; 4];
// This doesn't work because const-eval happens after type-checking,
// and the type checker needs to know the size of the array.
// const CARR2 = [1; X_SIZE - Y_SIZE + 1];

fn main() -> u64 {
    const int1 = 1;
    assert(int1 == INT1 && ZERO_B256 == KEY);

    // initialization through function applications.
    const eth_id0 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    const eth_id1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(eth_id0 == ETH_ID0 && eth_id1 == ETH_ID1);

    // tuples and arrays.
    const t1 = (2, 1, 21);
    assert(t1.0 == TUP1.0 && t1.1 == TUP1.1 && t1.2 == TUP1.2);
    assert(t1.0 == TUP2.0 && t1.1 == TUP2.1 && t1.2 == TUP2.2);
    const a1 = [1, 2, 3];
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
    assert(ETH_ID0.value == ETH_ID0_VALUE);
    assert(TUP1_idx2 == TUP1.2);
    assert(XPY == 2);
    assert(SO == __size_of::<u64>());
    assert(SOV == __size_of_val("hello"));
    assert(TRUEB != FALSEB);
    assert(TRUEB1 != FALSEB1);
    assert(FOO_MIDDLE == BAR);
    assert(OPS == 23);

    1
}
