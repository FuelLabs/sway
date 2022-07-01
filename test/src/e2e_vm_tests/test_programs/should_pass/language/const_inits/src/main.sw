script;

use std::{assert::assert, contract_id::ContractId, logging::log};

const ETH_ID0 = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
fn contract_id_wrapper(b: b256) -> ContractId {
    ~ContractId::from(b)
}
const ETH_ID1 = contract_id_wrapper(0x0000000000000000000000000000000000000000000000000000000000000001);

const TUP1 = (2, 1, 21);
const ARR1 = [1, 2, 3];

fn tup_wrapper(a: u64, b: u64, c: u64) -> (u64, u64, u64) {
    (a, b, c)
}
const TUP2 = tup_wrapper(2, 1, 21);

fn arr_wrapper(a: u64, b: u64, c: u64) -> [u64;
3] {
    return [a, b, c];
}
const ARR2 = arr_wrapper(1, 2, 3);

enum En1 {
    Int: u64,
    Arr: [u64;
    3],
    NoVal: (),
}

const EN1a = En1::Int(101);
const EN1b = En1::Arr(ARR2);
const EN1c = En1::NoVal;

const ETH_ID0_VALUE = ETH_ID0.value;
const TUP1_idx2 = TUP1.2;

const INT1 = 1;

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const KEY = ZERO_B256;

fn main() -> u64 {
    const int1 = 1;
    assert(int1 == INT1 && ZERO_B256 == KEY);

    // initialization through function applications.
    const eth_id0 = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    const eth_id1 = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(eth_id0 == ETH_ID0 && eth_id1 == ETH_ID1);

    // tuples and arrays.
    const t1 = (2, 1, 21);
    assert(t1.0 == TUP1.0 && t1.1 == TUP1.1 && t1.2 == TUP1.2);
    assert(t1.0 == TUP2.0 && t1.1 == TUP2.1 && t1.2 == TUP2.2);
    const a1 = [1, 2, 3];
    assert(a1[0] == ARR1[0] && a1[1] == ARR1[1] && a1[2] == ARR1[2]);
    assert(a1[0] == ARR2[0] && a1[1] == ARR2[1] && a1[2] == ARR2[2]);

    // enum
    match EN1a {
        En1::Int(i) => assert(i == 101), En1::Arr(_) => assert(false), En1::NoVal => assert(false), 
    }
    match EN1b {
        En1::Int(i) => assert(false), En1::Arr(arr) => {
            assert(arr[0] == ARR1[0] && arr[1] == ARR1[1] && arr[2] == ARR1[2]);
        }
        En1::NoVal => assert(false), 
    }
    match EN1c {
        En1::Int(i) => assert(false), En1::Arr(_) => assert(false), En1::NoVal => assert(true), 
    }

    // Struct and enum field access.
    assert(ETH_ID0.value == ETH_ID0_VALUE);
    assert(TUP1_idx2 == TUP1.2);

    1
}
