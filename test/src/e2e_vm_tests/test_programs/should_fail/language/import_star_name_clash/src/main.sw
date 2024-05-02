script;

mod a;
mod b;
mod c;
mod d;

// modules a and b both define the following names
// MyStruct
// MyEnum
// MyEnum::A
// C (as a::MyOtherEnum::C and b::C)
// E (as a::MyEnumVariants::E and b::MyOtherEnumVariants::E)
//
// Star importing the modules causes a name clash, but is not an error.
// Using the clashing names as simple names (without a callpath) is an error.
// Using the clashing names with a callpath is not an error.
use a::*; 
use b::*; // Not an error despite name clashes
use b::MyOtherEnum::*; // Import variant C

// modules c and d both define the enum variant E but for differently named enums
use c::MyEnumVariants;
use d::MyOtherEnumVariants;
use c::MyEnumVariants::*;
use d::MyOtherEnumVariants::*; // Not an error despite variant name clash

fn bad_variants_project_my_enum_b(e : b::MyEnum) -> u64 {
    match e {
	// Error - A and B are not in scope
	A(val)
	| B(val) => val,
    }
}

// Error: MyEnum is ambiguous
fn bad_enum_project_my_enum_b(e : MyEnum) -> u64 {
    match e {
	// Error - MyEnum::A and MyEnum::B are ambiguous
	MyEnum::A(val)
	| MyEnum::B(val) => val, // BUG: This resolves to b::MyEnum.
    }
}

fn bad_variants_project_my_enum_variants(e : MyEnumVariants) -> u64 {
    match e {
	// Error - E is ambiguous
	D(val)
	| E(val) => val, // BUG: This resolves to d::MyOtherEnumVariants::E
    }
}

fn main() {
    let my_struct_a = MyStruct { a : 0 }; // Error - MyStruct is ambiguous // BUG: MyStruct resolves to b::MyStruct
    let my_struct_b = MyStruct { b : 3 }; // Error - MyStruct is ambiguous // BUG: MyStruct resolves to b::MyStruct
    let my_struct_b_wrong_field = b::MyStruct { a : 6 }; // Error - b::MyStruct does not contain field a

    let my_enum_a_variant = A(100); // Error - A is not in scope
    let my_enum_a_enum_variant = MyEnum::A(101); // Error - MyEnum is ambiguous // BUG: Resolves to b::MyEnum::A
    let my_enum_b_variant = B(104); // Error - B is not in scope
    let my_enum_b_enum_variant = MyEnum::B(105); // Error - MyEnum is ambiguous // BUG: Resolves to b::MyEnum::B
    let my_enum_a_wrong_variant = b::MyEnum::B(108); // Error - b::MyEnum does not contain variant B

    let my_enum_function_wrong_type = project_my_enum_b(my_enum_a_enum_variant_relative); // Error - wrong MyEnum // BUG: project_my_enum_b not imported
    let my_enum_local_function_1 = bad_variants_project_my_enum_b(my_enum_b_enum_variant_relative); // Legal
    let my_enum_local_function_2 = bad_enum_project_my_enum_b(my_enum_b_enum_variant_relative); // Legal

    let c_struct = C { b: 200 }; // Error - C is ambiguous // BUG: Resolves to b::MyOtherEnum::C
    let c_variant = C(203); // Error - C is ambiguous // BUG: Resolves to b::MyOtherEnum::C

    let variants_e = E (304); // Error - E is ambiguous // BUG: Resolves to d::MyOtherEnumVariants::E
    let variants_function_1 = bad_variants_project_my_enum_variants(variants_d); // Legal
    let variants_function_1 = bad_variants_project_my_enum_variants(variants_e) // Error - wrong argument type
}
