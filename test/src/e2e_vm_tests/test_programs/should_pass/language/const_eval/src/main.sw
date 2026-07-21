script;

const ETH_ID0 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
fn contract_id_wrapper(b: b256) -> ContractId {
    ContractId::from(b)
}
const ETH_ID1 = contract_id_wrapper(0x0000000000000000000000000000000000000000000000000000000000000001);

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
    const forty_two = 42;
    let x: u64 = forty_two;
    x
}

// test variable scopes and local const
fn id(x: u64) -> u64 {
    const forty_two = 42;
    {
        let x: u64 = forty_two;
    };
    x
}

const QUUX: u64 = id(0);
const BAZ: u64 = const_42(123456);

const TUP1 = (2, 1, 21);
const ARR1 = [1, 2, 3];

fn tup_wrapper(a: u64, b: u64, c: u64) -> (u64, u64, u64) {
    (a, b, c)
}
const TUP2 = tup_wrapper(2, 1, 21);

fn arr_wrapper(a: u64, b: u64, c: u64) -> [u64; 3] {
    [a, b, c]
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

const ETH_ID0_VALUE = ETH_ID0.bits();
const TUP1_idx2 = TUP1.2;

const INT1 = 1;

// b256
const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const ONE_B256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const KEY = ZERO_B256;
const BITWISE_B256: b256 = !ZERO_B256 & ZERO_B256 | ZERO_B256 ^ ZERO_B256;
const SHIFT_B256: b256 = ZERO_B256 >> 1 << 1;

const BAR: u32 = 6;
const FOO: u32 = ((u32::min() + 1) * 12 / 2 - 1) % 6;
const MASK: u32 = 11;
const MASK2: u32 = 8;
const MASK3: u32 = 15;
const FOO_MIDDLE: u32 = ((FOO & MASK) | MASK2) ^ MASK3;
const OPS: u64 = 10 + 9 - 8 * 7 / 6 << 5 >> 4 ^ 3 | 2 & bool_to_num(true);

const CARR1 = [X_SIZE - Y_SIZE + 1; 4];
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

// Zero-sized enum variants, mirroring `match_expressions_enums_zero_sized_variants` test.
trait GetVal {
    fn get_val(self) -> u64;
}

impl GetVal for () {
    fn get_val(self) -> u64 { 42 }
}

impl GetVal for [u8; 0] {
    fn get_val(self) -> u64 { 8 }
}

impl GetVal for [u64; 0] {
    fn get_val(self) -> u64 { 64 }
}

impl GetVal for [u256; 0] {
    fn get_val(self) -> u64 { 256 }
}

impl GetVal for str[0] {
    fn get_val(self) -> u64 { 512 }
}

// Empty structs that are themselves made only of zero-sized fields. This ensures we also
// cover zero-sized types that have structure, not just the trivial `{}` case.
struct EmptyStruct01 {}
struct EmptyStruct02 { es01: EmptyStruct01 }
struct EmptyStruct03 { es02: EmptyStruct02, empty_arr: [u8; 0] }

impl GetVal for EmptyStruct01 {
    fn get_val(self) -> u64 { 1 }
}

impl GetVal for EmptyStruct02 {
    fn get_val(self) -> u64 { self.es01.get_val() }
}

impl GetVal for EmptyStruct03 {
    fn get_val(self) -> u64 { self.es02.get_val() + self.empty_arr.get_val() }
}

impl GetVal for [EmptyStruct03; 100] {
    fn get_val(self) -> u64 { self[0].get_val() }
}

enum AllVariantsUnit {
    A: (),
    B: (),
    C: (),
}

enum AllVariantsEmptyStructs {
    A: EmptyStruct01,
    B: EmptyStruct02,
    C: EmptyStruct03,
}

enum AllVariantsEmptyArrays {
    A: [u8; 0],
    B: [u64; 0],
    C: [u256; 0],
    D: [EmptyStruct03; 100],
}

enum AllVariantsDifferentTypes {
    A: (),
    B: EmptyStruct01,
    C: [u8; 0],
    D: str[0],
}

enum GenericAllVariantsEmpty<T1, T2, T3> {
    A: T1,
    B: T2,
    C: T3,
}

const AVU_A = AllVariantsUnit::A;
const AVU_B = AllVariantsUnit::B;
const AVU_C = AllVariantsUnit::C;

// Sample zero-sized values reused when instantiating the enum variants below.
const ES02: EmptyStruct02 = EmptyStruct02 { es01: EmptyStruct01 {} };
const ES03: EmptyStruct03 = EmptyStruct03 { es02: EmptyStruct02 { es01: EmptyStruct01 {} }, empty_arr: [] };

const AVES_A = AllVariantsEmptyStructs::A(EmptyStruct01 {});
const AVES_B = AllVariantsEmptyStructs::B(ES02);
const AVES_C = AllVariantsEmptyStructs::C(ES03);

const AVEA_A = AllVariantsEmptyArrays::A([]);
const AVEA_B = AllVariantsEmptyArrays::B([]);
const AVEA_C = AllVariantsEmptyArrays::C([]);
const AVEA_D = AllVariantsEmptyArrays::D([ES03; 100]);

const AVDT_A = AllVariantsDifferentTypes::A;
const AVDT_B = AllVariantsDifferentTypes::B(EmptyStruct01 {});
const AVDT_C = AllVariantsDifferentTypes::C([]);
const AVDT_D = AllVariantsDifferentTypes::D(__to_str_array(""));

const AVES_C_PAYLOAD: EmptyStruct03 = match AVES_C {
    AllVariantsEmptyStructs::C(s) => s,
    _ => ES03,
};
const AVEA_D_PAYLOAD: [EmptyStruct03; 100] = match AVEA_D {
    AllVariantsEmptyArrays::D(a) => a,
    _ => [ES03; 100],
};
const AVDT_D_PAYLOAD: str[0] = match AVDT_D {
    AllVariantsDifferentTypes::D(s) => s,
    _ => __to_str_array(""),
};

const GAVE_A = GenericAllVariantsEmpty::<EmptyStruct01, EmptyStruct02, EmptyStruct03>::A(EmptyStruct01 {});
const GAVE_B = GenericAllVariantsEmpty::<EmptyStruct01, EmptyStruct02, EmptyStruct03>::B(ES02);
const GAVE_C = GenericAllVariantsEmpty::<EmptyStruct01, EmptyStruct02, EmptyStruct03>::C(ES03);

fn main() -> u64 {
    const int1 = 1;
    assert(int1 == INT1 && ZERO_B256 == KEY);

    // Initialization through function applications.
    const eth_id0 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    const eth_id1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(eth_id0 == ETH_ID0 && eth_id1 == ETH_ID1);
    assert(BAZ == 42);
    assert(QUUX == 0);

    // Tuples and arrays.
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

    // Enums.
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

    // Matching on consts of enums with zero-sized variants.
    match AVU_A {
        AllVariantsUnit::A => assert(().get_val() == 42),
        AllVariantsUnit::B => assert(false),
        AllVariantsUnit::C => assert(false),
    }
    match AVU_B {
        AllVariantsUnit::A => assert(false),
        AllVariantsUnit::B => assert(().get_val() == 42),
        AllVariantsUnit::C => assert(false),
    }
    match AVU_C {
        AllVariantsUnit::A => assert(false),
        AllVariantsUnit::B => assert(false),
        AllVariantsUnit::C => assert(().get_val() == 42),
    }

    match AVES_A {
        AllVariantsEmptyStructs::A(s) => assert(s.get_val() == 1),
        AllVariantsEmptyStructs::B(_) => assert(false),
        AllVariantsEmptyStructs::C(_) => assert(false),
    }
    match AVES_B {
        AllVariantsEmptyStructs::A(_) => assert(false),
        AllVariantsEmptyStructs::B(s) => assert(s.get_val() == 1),
        AllVariantsEmptyStructs::C(_) => assert(false),
    }
    match AVES_C {
        AllVariantsEmptyStructs::A(_) => assert(false),
        AllVariantsEmptyStructs::B(_) => assert(false),
        AllVariantsEmptyStructs::C(s) => assert(s.get_val() == 9),
    }

    match AVEA_A {
        AllVariantsEmptyArrays::A(a) => assert(a.get_val() == 8),
        AllVariantsEmptyArrays::B(_) => assert(false),
        AllVariantsEmptyArrays::C(_) => assert(false),
        AllVariantsEmptyArrays::D(_) => assert(false),
    }
    match AVEA_B {
        AllVariantsEmptyArrays::A(_) => assert(false),
        AllVariantsEmptyArrays::B(a) => assert(a.get_val() == 64),
        AllVariantsEmptyArrays::C(_) => assert(false),
        AllVariantsEmptyArrays::D(_) => assert(false),
    }
    match AVEA_C {
        AllVariantsEmptyArrays::A(_) => assert(false),
        AllVariantsEmptyArrays::B(_) => assert(false),
        AllVariantsEmptyArrays::C(a) => assert(a.get_val() == 256),
        AllVariantsEmptyArrays::D(_) => assert(false),
    }
    match AVEA_D {
        AllVariantsEmptyArrays::A(_) => assert(false),
        AllVariantsEmptyArrays::B(_) => assert(false),
        AllVariantsEmptyArrays::C(_) => assert(false),
        AllVariantsEmptyArrays::D(a) => assert(a.get_val() == 9),
    }

    match AVDT_A {
        AllVariantsDifferentTypes::A => assert(().get_val() == 42),
        AllVariantsDifferentTypes::B(_) => assert(false),
        AllVariantsDifferentTypes::C(_) => assert(false),
        AllVariantsDifferentTypes::D(_) => assert(false),
    }
    match AVDT_B {
        AllVariantsDifferentTypes::A => assert(false),
        AllVariantsDifferentTypes::B(s) => assert(s.get_val() == 1),
        AllVariantsDifferentTypes::C(_) => assert(false),
        AllVariantsDifferentTypes::D(_) => assert(false),
    }
    match AVDT_C {
        AllVariantsDifferentTypes::A => assert(false),
        AllVariantsDifferentTypes::B(_) => assert(false),
        AllVariantsDifferentTypes::C(a) => assert(a.get_val() == 8),
        AllVariantsDifferentTypes::D(_) => assert(false),
    }
    match AVDT_D {
        AllVariantsDifferentTypes::A => assert(false),
        AllVariantsDifferentTypes::B(_) => assert(false),
        AllVariantsDifferentTypes::C(_) => assert(false),
        AllVariantsDifferentTypes::D(s) => assert(s.get_val() == 512),
    }

    assert(AVES_C_PAYLOAD.get_val() == 9);
    assert(AVEA_D_PAYLOAD.get_val() == 9);
    assert(AVDT_D_PAYLOAD.get_val() == 512);

    match GAVE_A {
        GenericAllVariantsEmpty::A(v) => assert(v.get_val() == 1),
        GenericAllVariantsEmpty::B(_) => assert(false),
        GenericAllVariantsEmpty::C(_) => assert(false),
    }
    match GAVE_B {
        GenericAllVariantsEmpty::A(_) => assert(false),
        GenericAllVariantsEmpty::B(v) => assert(v.get_val() == 1),
        GenericAllVariantsEmpty::C(_) => assert(false),
    }
    match GAVE_C {
        GenericAllVariantsEmpty::A(_) => assert(false),
        GenericAllVariantsEmpty::B(_) => assert(false),
        GenericAllVariantsEmpty::C(v) => assert(v.get_val() == 9),
    }

    42
}

const NOTA = !0u8;
const NOTB = !0u16;
const NOTC = !0u32;
const NOTD = !0u64;
const NOTE = !false;

fn test_not() {
    assert(NOTA == 0xFFu8);
    assert(NOTB == 0xFFFFu16);
    assert(NOTC == 0xFFFFFFFFu32);
    assert(NOTD == 0xFFFFFFFFFFFFFFFFu64);
    assert(NOTE == true);
}
