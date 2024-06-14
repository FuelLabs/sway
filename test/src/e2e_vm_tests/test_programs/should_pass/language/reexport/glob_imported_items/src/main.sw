script;

mod lib_1;
mod items_1;

use lib_1::*;

fn test_items1_struct1(input: Items1Struct) -> Items1Struct {
    // Items1Struct and Lib1Struct are the same type - they are aliased in lib_1
    Lib1Struct { a: input.a + 1 }
}

fn flip_items1_enum1(input: Items1Enum1) -> Items1Enum1 {
    match input {
	Items1Enum1::A(val) => Items1Enum1::B(val),
	Items1Enum1::B(val) => Items1Enum1::A(val),
    }
}

fn test_items1_enum1(input: Items1Enum1) -> u64 {
    match flip_items1_enum1(input) {
	Items1Enum1::A(val) => val + 1000,
	Items1Enum1::B(val) => val,
    }
}

fn flip_items1_enum2(input: items_1::Items1Enum2) -> items_1::Items1Enum2 {
    match input {
	X(val) => Y(val),
	Y(val) => X(val + 1),
    }
}

fn test_items1_enum2(input: items_1::Items1Enum2) -> u64 {
    match flip_items1_enum2(input) {
	X(val) => val + 10000,
	Y(val) => val,
    }
}

fn main() -> u64 {
    let struct1 = Items1Struct { a : 42 };
    let res = test_items1_struct1(struct1);
    assert(res.a == 43);

    let enum1 = Items1Enum1::A(123);
    let res = test_items1_enum1(enum1);
    assert(res == 123);

    let enum2 = items_1::Items1Enum2::X(456);
    let res = test_items1_enum2(enum2);
    assert(res == 456);
 
    let fun1 = items_1_function_1();
    assert(fun1 == 1);
 
    // items_1_function2 aliased to lib_1_function_2 in lib_1
    let fun2 = lib_1_function_2();
    assert(fun2 == 2);

    42
}





// Reexport using `pub use a::X`
// Reexport using `pub use a::X as Y`


// Reexport using `pub use a::*`
// Reexport using `pub use a::{self, X}`
// Reexport of enum variants
// Reexport of impls and traits
// Reexport name clashes when the reexport uses aliases:
// - Item imports: Name clashes illegal, but allowed to clash with unaliased name
// - Star imports: Name clashes legal, but use of unqualified clashing name is illegal. Allow redefinition of unaliased name, since this does not clash with alias
// Name clashes with preludes:
// - All explicit imports should shadow implicit prelude imports
// - Check name clashes with explicit imports from preludes + std and core

// Structure:
// Declarations in source_decls.sw
// Tests in tests.sw
// Other modules are used for reexporting.

// mod source_decls_items; // Decls for item reexports
// mod items; // Item reexports
// 
// mod verification; // Verifies expected values
// 
// 
// 
// use items::Struct01; // Reexported from source_decls_items::MyStructItem
// use items::A; // Reexported from source_decls_items::MyEnumItem::A
// use items::C; // Reexported and aliased from source_decls_items::MyEnumItem::B
// use items::my_function; // Reexported and aliased source_decls_items::from my_function_item
// 
// 
// 
// 
// fn test_item_imports (x: MyStructItem,
// 		      y: MyEnumItem,
// 		      z: MyEnumItem,
// 		      w: u64) -> bool {
//     let y_val = match y {
// 	MyEnumItem::A(val) => val,
// 	MyEnumItem::B(val) => val+1000,
//     };
//     let z_val = match y {
// 	MyEnumItem::A(val) => 1000+val,
// 	MyEnumItem::B(val) => val,
//     };
// 
//     x.a == 123 &&
//     y_val == 321 &&
//     z_val == 456 &&
//     w == 42
// }
// 
// fn main() {
//     // Item reexports
//     let struct_01 = Struct01 { a : 123 };
//     let enum_02_a = A (321);
//     let enum_02_b = C (456); // Actually B because of aliasing in items
//     let function_03_res = my_function();
//     assert(tests::test_item_imports(struct_01, enum_02_a, enum_02_b, function_03_res));
// 
//     
// }
// 
// 
// 
// 
// 
// 
// // modules a and b both define the following names
// // MyStruct
// // MyEnum
// // MyEnum::A
// // C (as a::MyOtherEnum::C and b::C)
// // E (as a::MyEnumVariants::E and b::MyOtherEnumVariants::E)
// //
// // Star importing the modules causes a name clash, but is not an error.
// // Using the clashing names as simple names (without a callpath) is an error.
// // Using the clashing names with a callpath is not an error.
// use a::*; 
// use b::*; // Not an error despite name clashes
// use b::MyOtherEnum::*; // Import variant C
// 
// // modules c and d both define the enum variant E but for differently named enums
// use c::MyEnumVariants;
// use d::MyOtherEnumVariants;
// use c::MyEnumVariants::*;
// use d::MyOtherEnumVariants::*; // Not an error despite variant name clash
// 
// fn good_project_my_enum_a(e : a::MyEnum) -> u64 {
//     match e {
// 	// Legal - a::MyEnum::A and a::MyEnum::B are unambiguous
// 	a::MyEnum::A(val)
// 	| a::MyEnum::B(val) => val,
//     }
// }
// 
// fn good_project_my_enum_b(e : b::MyEnum) -> u64 {
//     match e {
// 	// Legal - b::MyEnum::A is unambiguous
// 	b::MyEnum::A(val) => val,
//     }
// }
// 
// fn good_project_my_other_enum_b(e : b::MyOtherEnum) -> u64 {
//     match e {
// 	// Legal - b::MyOtherEnum::C is unambiguous
// 	b::MyOtherEnum::C(val) => val,
//     }
// }
// 
// fn good_project_my_enum_variants(e : MyEnumVariants) -> u64 {
//     match e {
// 	// Legal - MyEnumVariants::D and MyEnumVariants::E are unambiguous
// 	MyEnumVariants::D(val)
// 	| MyEnumVariants::E(val) => val,
//     }
// }
// 
// fn good_project_my_other_enum_variants(e : MyOtherEnumVariants) -> u64 {
//     match e {
// 	// Legal - MyOtherEnumVariants::E and MyOtherEnumVariants::F are unambiguous
// 	MyOtherEnumVariants::E(val)
// 	| MyOtherEnumVariants::F(val) => val,
//     }
// }
// 
// 
// fn main() -> u64 {
//     let my_struct_a_relative = a::MyStruct { a : 1 }; // Legal - a::MyStruct is unambiguous
//     let my_struct_a_absolute = ::a::MyStruct { a : 2 }; // Legal - ::a::MyStruct is unambiguous
//     let my_struct_b_relative = b::MyStruct { b : 4 }; // Legal - b::MyStruct is unambiguous
//     let my_struct_b_absolute = ::b::MyStruct { b : 5 }; // Legal - ::b::MyStruct is unambiguous
// 
//     let my_enum_a_enum_variant_relative = a::MyEnum::A(102); // Legal - a::MyEnum is unambiguous
//     let my_enum_a_enum_variant_absolute = ::a::MyEnum::A(103); // Legal - ::a::MyEnum is unambiguous
//     let my_enum_b_enum_variant_relative = b::MyEnum::A(106); // Legal - b::MyEnum is unambiguous
//     let my_enum_b_enum_variant_absolute = ::b::MyEnum::A(107); // Legal - ::b::MyEnum is unambiguous
//     let my_enum_function_type = project_my_enum_b(my_enum_b_enum_variant_relative); // Legal // BUG: project_my_enum_b not imported
//     let my_enum_local_function_3 = good_project_my_enum_a(my_enum_a_enum_variant_relative); // Legal
// 
//     let c_struct_relative = a::C { b: 201 }; // Legal - a::C is unambiguous
//     let c_struct_absolute = ::a::C { b: 202 }; // Legal - ::a::C is unambiguous
//     let c_variant_enum = MyOtherEnum::C(204); // Legal - MyOtherEnum::C is unambiguous
//     let c_variant_enum_relative = b::MyOtherEnum::C(205); // Legal - b::MyOtherEnum::C is unambiguous
//     let c_variant_enum_absolute = ::b::MyOtherEnum::C(206); // Legal - ::b::MyOtherEnum::C is unambiguous
// 
//     let variants_d = D (300); // Legal - D is unambiguous
//     let variants_d_enum_variant = MyEnumVariants::D (301); // Legal - MyEnumVariants::D is unambiguous
//     let variants_d_relative = c::MyEnumVariants::D (302); // Legal - c::MyEnumVariants::D is unambiguous
//     let variants_d_absolute = ::c::MyEnumVariants::D (303); // Legal - ::c::MyEnumVariants::D is unambiguous
//     let variants_e_enum_variant = MyEnumVariants::E (305); // Legal - MyEnumVariants::E is unambiguous
//     let variants_e_relative = c::MyEnumVariants::E (306); // Legal - c::MyEnumVariants::E is unambiguous
//     let variants_e_absolute = ::c::MyEnumVariants::E (307); // Legal - ::c::MyEnumVariants::E is unambiguous
// 
//     let variants_other_e_enum_variant = MyOtherEnumVariants::E (309); // Legal - MyOtherEnumVariants::E is unambiguous
//     let variants_other_e_relative = d::MyOtherEnumVariants::E (310); // Legal - c::MyOtherEnumVariants::E is unambiguous
//     let variants_other_e_absolute = ::d::MyOtherEnumVariants::E (311); // Legal - ::c::MyOtherEnumVariants::E is unambiguous
//     let variants_f = F (312); // Legal - F is unambiguous
//     let variants_f_enum_variant = MyOtherEnumVariants::F (313); // Legal - MyOtherEnumVariants::F is unambiguous
//     let variants_f_relative = d::MyOtherEnumVariants::F (314); // Legal - c::MyOtherEnumVariants::F is unambiguous
//     let variants_f_absolute = ::d::MyOtherEnumVariants::F (315); // Legal - ::c::MyOtherEnumVariants::F is unambiguous
//     let variants_function_2 = good_project_my_enum_variants(variants_d); // Legal
// 
//     my_struct_a_relative.a
// 	+ my_struct_a_absolute.a
// 	+ my_struct_b_relative.b
// 	+ my_struct_b_absolute.b
// 	+ good_project_my_enum_a(my_enum_a_enum_variant_relative)
// 	+ good_project_my_enum_a(my_enum_a_enum_variant_absolute)
// 	+ good_project_my_enum_b(my_enum_b_enum_variant_relative)
// 	+ good_project_my_enum_b(my_enum_b_enum_variant_absolute)
// 	+ my_enum_function_type
// 	+ my_enum_local_function_3
// 	+ c_struct_relative.b
// 	+ c_struct_absolute.b
// 	+ good_project_my_other_enum_b(c_variant_enum)
// 	+ good_project_my_other_enum_b(c_variant_enum_relative)
// 	+ good_project_my_other_enum_b(c_variant_enum_absolute)
// 	+ good_project_my_enum_variants(variants_d)
// 	+ good_project_my_enum_variants(variants_d_enum_variant)
// 	+ good_project_my_enum_variants(variants_d_relative)
// 	+ good_project_my_enum_variants(variants_d_absolute)
// 	+ good_project_my_enum_variants(variants_e_enum_variant)
// 	+ good_project_my_enum_variants(variants_e_relative)
// 	+ good_project_my_enum_variants(variants_e_absolute)
// 	+ good_project_my_other_enum_variants(variants_other_e_enum_variant)
// 	+ good_project_my_other_enum_variants(variants_other_e_relative)
// 	+ good_project_my_other_enum_variants(variants_other_e_absolute)
// 	+ good_project_my_other_enum_variants(variants_f)
// 	+ good_project_my_other_enum_variants(variants_f_enum_variant)
// 	+ good_project_my_other_enum_variants(variants_f_relative)
// 	+ good_project_my_other_enum_variants(variants_f_absolute)
// 	+ variants_function_2
// }
// 
