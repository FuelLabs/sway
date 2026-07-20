script;

trait GetVal {
    fn get_val(self) -> u64;
}

impl GetVal for () {
    fn get_val(self) -> u64 { 42 }
}

impl GetVal for [u8;0] {
    fn get_val(self) -> u64 { 8 }
}

impl GetVal for [u64;0] {
    fn get_val(self) -> u64 { 64 }
}

impl GetVal for [u256;0] {
    fn get_val(self) -> u64 { 256 }
}

// Empty structs that are themselves made only of zero-sized fields. This ensures we also
// cover zero-sized types that have structure, not just the trivial `{}` case.
struct EmptyStruct01 {}
struct EmptyStruct02 { es01: EmptyStruct01 }
struct EmptyStruct03 { es02: EmptyStruct02, empty_arr: [u8;0] }

impl GetVal for EmptyStruct01 {
    fn get_val(self) -> u64 { 1 }
}

// Empty structs with fields return the sum of the `get_val`s of their fields.
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
    A: [u8;0],
    B: [u64;0],
    C: [u256;0],
    D: [EmptyStruct03; 100],
}

enum AllVariantsDifferentTypes {
    A: (),
    B: EmptyStruct01,
    C: [u8;0],
}

enum GenericAllVariantsEmpty<T1, T2, T3> {
    A: T1,
    B: T2,
    C: T3,
}

fn match_all_variants_unit(val: AllVariantsUnit) -> u64 {
    match val {
        AllVariantsUnit::A => 11,
        AllVariantsUnit::B => 22,
        AllVariantsUnit::C => 33,
    }
}

fn match_all_variants_empty_structs(val: AllVariantsEmptyStructs) -> u64 {
    match val {
        AllVariantsEmptyStructs::A(s) => s.get_val(),
        AllVariantsEmptyStructs::B(s) => s.get_val(),
        AllVariantsEmptyStructs::C(s) => s.get_val(),
    }
}

fn match_all_variants_empty_arrays(val: AllVariantsEmptyArrays) -> u64 {
    match val {
        AllVariantsEmptyArrays::A(a) => a.get_val(),
        AllVariantsEmptyArrays::B(a) => a.get_val(),
        AllVariantsEmptyArrays::C(a) => a.get_val(),
        AllVariantsEmptyArrays::D(a) => a.get_val(),
    }
}

fn match_all_variants_different_types(val: AllVariantsDifferentTypes) -> u64 {
    match val {
        AllVariantsDifferentTypes::A => ().get_val(),
        AllVariantsDifferentTypes::B(s) => s.get_val(),
        AllVariantsDifferentTypes::C(a) => a.get_val(),
    }
}

fn match_generic_all_variants_empty<T1, T2, T3>(val: GenericAllVariantsEmpty<T1, T2, T3>) -> u64
where
    T1: GetVal,
    T2: GetVal,
    T3: GetVal,
{
    match val {
        GenericAllVariantsEmpty::A(v) => v.get_val(),
        GenericAllVariantsEmpty::B(v) => v.get_val(),
        GenericAllVariantsEmpty::C(v) => v.get_val(),
    }
}

// Sample zero-sized values reused when instantiating the enum variants below.
const ES02: EmptyStruct02 = EmptyStruct02 { es01: EmptyStruct01 {} };
const ES03: EmptyStruct03 = EmptyStruct03 { es02: EmptyStruct02 { es01: EmptyStruct01 {} }, empty_arr: [] };

fn main() -> u64 {
    let x = match_all_variants_unit(AllVariantsUnit::A);
    assert_eq(x, 11);

    let x = match_all_variants_unit(AllVariantsUnit::B);
    assert_eq(x, 22);

    let x = match_all_variants_unit(AllVariantsUnit::C);
    assert_eq(x, 33);

    let x = match_all_variants_empty_structs(AllVariantsEmptyStructs::A(EmptyStruct01 {}));
    assert_eq(x, 1);

    let x = match_all_variants_empty_structs(AllVariantsEmptyStructs::B(ES02));
    assert_eq(x, 1);

    let x = match_all_variants_empty_structs(AllVariantsEmptyStructs::C(ES03));
    assert_eq(x, 9);

    let x = match_all_variants_empty_arrays(AllVariantsEmptyArrays::A([]));
    assert_eq(x, 8);

    let x = match_all_variants_empty_arrays(AllVariantsEmptyArrays::B([]));
    assert_eq(x, 64);

    let x = match_all_variants_empty_arrays(AllVariantsEmptyArrays::C([]));
    assert_eq(x, 256);

    let x = match_all_variants_empty_arrays(AllVariantsEmptyArrays::D([ES03; 100]));
    assert_eq(x, 9);

    let x = match_all_variants_different_types(AllVariantsDifferentTypes::A);
    assert_eq(x, 42);

    let x = match_all_variants_different_types(AllVariantsDifferentTypes::B(EmptyStruct01 {}));
    assert_eq(x, 1);

    let x = match_all_variants_different_types(AllVariantsDifferentTypes::C([]));
    assert_eq(x, 8);

    let x = match_generic_all_variants_empty(GenericAllVariantsEmpty::<EmptyStruct01, EmptyStruct02, EmptyStruct03>::A(EmptyStruct01 {}));
    assert_eq(x, 1);

    let x = match_generic_all_variants_empty(GenericAllVariantsEmpty::<EmptyStruct01, EmptyStruct02, EmptyStruct03>::B(ES02));
    assert_eq(x, 1);

    let x = match_generic_all_variants_empty(GenericAllVariantsEmpty::<EmptyStruct01, EmptyStruct02, EmptyStruct03>::C(ES03));
    assert_eq(x, 9);

    let x = match_generic_all_variants_empty(GenericAllVariantsEmpty::<(), EmptyStruct01, [u256; 0]>::A(()));
    assert_eq(x, 42);

    let x = match_generic_all_variants_empty(GenericAllVariantsEmpty::<(), EmptyStruct01, [u256; 0]>::B(EmptyStruct01 {}));
    assert_eq(x, 1);

    let x = match_generic_all_variants_empty(GenericAllVariantsEmpty::<(), EmptyStruct01, [u256; 0]>::C([]));
    assert_eq(x, 256);

    42
}
