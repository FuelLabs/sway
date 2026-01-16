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

fn good_project_my_enum_a(e : a::MyEnum) -> u64 {
    match e {
    // Legal - a::MyEnum::A and a::MyEnum::B are unambiguous
    a::MyEnum::A(val)
    | a::MyEnum::B(val) => val,
    }
}

fn good_project_my_enum_b(e : b::MyEnum) -> u64 {
    match e {
    // Legal - b::MyEnum::A is unambiguous
    b::MyEnum::A(val) => val,
    }
}

fn good_project_my_other_enum_b(e : b::MyOtherEnum) -> u64 {
    match e {
    // Legal - b::MyOtherEnum::C is unambiguous
    b::MyOtherEnum::C(val) => val,
    }
}

fn good_project_my_enum_variants(e : MyEnumVariants) -> u64 {
    match e {
    // Legal - MyEnumVariants::D and MyEnumVariants::E are unambiguous
    MyEnumVariants::D(val)
    | MyEnumVariants::E(val) => val,
    }
}

fn good_project_my_other_enum_variants(e : MyOtherEnumVariants) -> u64 {
    match e {
    // Legal - MyOtherEnumVariants::E and MyOtherEnumVariants::F are unambiguous
    MyOtherEnumVariants::E(val)
    | MyOtherEnumVariants::F(val) => val,
    }
}

fn main() -> u64 {
    let my_struct_a_relative = a::MyStruct { a : 1 }; // Legal - a::MyStruct is unambiguous
    let my_struct_a_absolute = ::a::MyStruct { a : 2 }; // Legal - ::a::MyStruct is unambiguous
    let my_struct_b_relative = b::MyStruct { b : 4 }; // Legal - b::MyStruct is unambiguous
    let my_struct_b_absolute = ::b::MyStruct { b : 5 }; // Legal - ::b::MyStruct is unambiguous

    let my_enum_a_enum_variant_relative = a::MyEnum::A(102); // Legal - a::MyEnum is unambiguous
    let my_enum_a_enum_variant_absolute = ::a::MyEnum::A(103); // Legal - ::a::MyEnum is unambiguous
    let my_enum_b_enum_variant_relative = b::MyEnum::A(106); // Legal - b::MyEnum is unambiguous
    let my_enum_b_enum_variant_absolute = ::b::MyEnum::A(107); // Legal - ::b::MyEnum is unambiguous
    let my_enum_function_type = project_my_enum_b(my_enum_b_enum_variant_relative); // Legal // BUG: project_my_enum_b not imported
    let my_enum_local_function_3 = good_project_my_enum_a(my_enum_a_enum_variant_relative); // Legal

    let c_struct_relative = a::C { b: 201 }; // Legal - a::C is unambiguous
    let c_struct_absolute = ::a::C { b: 202 }; // Legal - ::a::C is unambiguous
    let c_variant_enum = MyOtherEnum::C(204); // Legal - MyOtherEnum::C is unambiguous
    let c_variant_enum_relative = b::MyOtherEnum::C(205); // Legal - b::MyOtherEnum::C is unambiguous
    let c_variant_enum_absolute = ::b::MyOtherEnum::C(206); // Legal - ::b::MyOtherEnum::C is unambiguous

    let variants_d = D (300); // Legal - D is unambiguous
    let variants_d_enum_variant = MyEnumVariants::D (301); // Legal - MyEnumVariants::D is unambiguous
    let variants_d_relative = c::MyEnumVariants::D (302); // Legal - c::MyEnumVariants::D is unambiguous
    let variants_d_absolute = ::c::MyEnumVariants::D (303); // Legal - ::c::MyEnumVariants::D is unambiguous
    let variants_e_enum_variant = MyEnumVariants::E (305); // Legal - MyEnumVariants::E is unambiguous
    let variants_e_relative = c::MyEnumVariants::E (306); // Legal - c::MyEnumVariants::E is unambiguous
    let variants_e_absolute = ::c::MyEnumVariants::E (307); // Legal - ::c::MyEnumVariants::E is unambiguous

    let variants_other_e_enum_variant = MyOtherEnumVariants::E (309); // Legal - MyOtherEnumVariants::E is unambiguous
    let variants_other_e_relative = d::MyOtherEnumVariants::E (310); // Legal - c::MyOtherEnumVariants::E is unambiguous
    let variants_other_e_absolute = ::d::MyOtherEnumVariants::E (311); // Legal - ::c::MyOtherEnumVariants::E is unambiguous
    let variants_f = F (312); // Legal - F is unambiguous
    let variants_f_enum_variant = MyOtherEnumVariants::F (313); // Legal - MyOtherEnumVariants::F is unambiguous
    let variants_f_relative = d::MyOtherEnumVariants::F (314); // Legal - c::MyOtherEnumVariants::F is unambiguous
    let variants_f_absolute = ::d::MyOtherEnumVariants::F (315); // Legal - ::c::MyOtherEnumVariants::F is unambiguous
    let variants_function_2 = good_project_my_enum_variants(variants_d); // Legal

    my_struct_a_relative.a
    + my_struct_a_absolute.a
    + my_struct_b_relative.b
    + my_struct_b_absolute.b
    + good_project_my_enum_a(my_enum_a_enum_variant_relative)
    + good_project_my_enum_a(my_enum_a_enum_variant_absolute)
    + good_project_my_enum_b(my_enum_b_enum_variant_relative)
    + good_project_my_enum_b(my_enum_b_enum_variant_absolute)
    + my_enum_function_type
    + my_enum_local_function_3
    + c_struct_relative.b
    + c_struct_absolute.b
    + good_project_my_other_enum_b(c_variant_enum)
    + good_project_my_other_enum_b(c_variant_enum_relative)
    + good_project_my_other_enum_b(c_variant_enum_absolute)
    + good_project_my_enum_variants(variants_d)
    + good_project_my_enum_variants(variants_d_enum_variant)
    + good_project_my_enum_variants(variants_d_relative)
    + good_project_my_enum_variants(variants_d_absolute)
    + good_project_my_enum_variants(variants_e_enum_variant)
    + good_project_my_enum_variants(variants_e_relative)
    + good_project_my_enum_variants(variants_e_absolute)
    + good_project_my_other_enum_variants(variants_other_e_enum_variant)
    + good_project_my_other_enum_variants(variants_other_e_relative)
    + good_project_my_other_enum_variants(variants_other_e_absolute)
    + good_project_my_other_enum_variants(variants_f)
    + good_project_my_other_enum_variants(variants_f_enum_variant)
    + good_project_my_other_enum_variants(variants_f_relative)
    + good_project_my_other_enum_variants(variants_f_absolute)
    + variants_function_2
}
