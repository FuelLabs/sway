script;

// The types are the same ones used in `match_expressions_enums_zero_sized_variants`
// and `const_eval`. This test snapshots the final IR so we can ensure the enums
// with zero-sized variants are lowered to `{ u64 }` only.

// Empty structs that are themselves made only of zero-sized fields. This ensures we also
// cover zero-sized types that have structure, not just the trivial `{}` case.
struct EmptyStruct01 {}
#[allow(dead_code)] // To suppress warning for non-used struct fields.
struct EmptyStruct02 { es01: EmptyStruct01 }
#[allow(dead_code)] // To suppress warning for non-used struct fields.
struct EmptyStruct03 { es02: EmptyStruct02, empty_arr: [u8; 0] }

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
}

enum GenericAllVariantsEmpty<T1, T2, T3> {
    A: T1,
    B: T2,
    C: T3,
}

// Sample zero-sized values reused when instantiating the enum variants below.
const ES02: EmptyStruct02 = EmptyStruct02 { es01: EmptyStruct01 {} };
const ES03: EmptyStruct03 = EmptyStruct03 { es02: EmptyStruct02 { es01: EmptyStruct01 {} }, empty_arr: [] };

const GLOBAL_UNIT = AllVariantsUnit::A;
const GLOBAL_EMPTY_STRUCTS = AllVariantsEmptyStructs::B(ES02);
const GLOBAL_EMPTY_ARRAYS = AllVariantsEmptyArrays::C([]);
const GLOBAL_EMPTY_ARRAYS_OF_STRUCTS = AllVariantsEmptyArrays::D([ES03; 100]);
const GLOBAL_DIFFERENT_TYPES = AllVariantsDifferentTypes::B(EmptyStruct01 {});
const GLOBAL_GENERIC = GenericAllVariantsEmpty::<EmptyStruct01, EmptyStruct02, EmptyStruct03>::A(EmptyStruct01 {});

#[inline(never)]
fn poke<T>(_x: T) {}

fn main() {
    let local_unit = AllVariantsUnit::B;
    let local_empty_structs = AllVariantsEmptyStructs::A(EmptyStruct01 {});
    let local_empty_arrays = AllVariantsEmptyArrays::A([]);
    let local_empty_arrays_of_structs = AllVariantsEmptyArrays::D([ES03; 100]);
    let local_different_types = AllVariantsDifferentTypes::C([]);
    let local_generic =
        GenericAllVariantsEmpty::<EmptyStruct01, EmptyStruct02, EmptyStruct03>::C(ES03);

    poke(local_unit);
    poke(local_empty_structs);
    poke(local_empty_arrays);
    poke(local_empty_arrays_of_structs);
    poke(local_different_types);
    poke(local_generic);

    poke(GLOBAL_UNIT);
    poke(GLOBAL_EMPTY_STRUCTS);
    poke(GLOBAL_EMPTY_ARRAYS);
    poke(GLOBAL_EMPTY_ARRAYS_OF_STRUCTS);
    poke(GLOBAL_DIFFERENT_TYPES);
    poke(GLOBAL_GENERIC);
}
