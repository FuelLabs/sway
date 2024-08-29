library;

 // Reexported items from items_1.sw. All reexports aliased by lib_1
use ::lib_1::Alias1_Struct;
use ::lib_1::Alias1_Enum;
use ::lib_1::Alias1_X;
use ::lib_1::Alias1_Y;
use ::lib_1::ALIAS_1_FUNCTION_RES;
use ::lib_1::alias_1_function;
use ::lib_1::Alias1Trait;

use ::items_1::Items1_Variants;

// Reexported items from items_2.sw. All reexports aliased by lib_2
use ::lib_2::*;

use ::items_2::Items2_Variants;

// Reexported items from items_3.sw. All reexports aliased by lib_3_1 and then by lib_3_2
use ::lib_3_2::*;

use ::items_3::Items3_Variants;

// Reexported items from items_4.sw. All items are reexported and aliased by lib_4_1 and by lib_4_2 using different aliases.
use ::lib_4_1::*;
use ::lib_4_2::*;

use ::items_4::Items4_Variants;


// Helper types

struct TestStruct1 {
    Z: u64,
}

struct TestStruct2 {
    W: bool,
}


// lib_1 tests

fn project_items_1_struct(input: Alias1_Struct) -> u64 {
    input.a
}

fn project_items_1_enum(input: Alias1_Enum) -> u64 {
    match input {
	Alias1_Enum::A(val) => val,
	Alias1_Enum::B(val) => val + 1000,
    }
}

// Aliased enum variants are not recognized as belonging to the enum they're coming from,
// so this test is disabled
//fn project_items_1_variants(input: Items1_Variants) -> u64 {
//    match input {
//	Alias1_X(val) => val,
//	Alias1_Y(val) => val + 1000,
//    }
//}

fn call_items_1_function() -> u64 {
    alias_1_function()
}

impl Alias1Trait<TestStruct2> for TestStruct1 {
    fn items_1_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 64 && x.W
    }
}


// lib_2 tests

fn project_items_2_struct(input: Alias2_Struct) -> u64 {
    input.b
}

fn project_items_2_enum(input: Alias2_Enum) -> u64 {
    match input {
	Alias2_Enum::C(val) => val,
	Alias2_Enum::D(val) => val + 1000,
    }
}

// Aliased enum variants are not recognized as belonging to the enum they're coming from,
// so this test is disabled
//fn project_items_2_variants(input: Items2_Variants) -> u64 {
//    match input {
//	Alias2_Z(val) => val,
//	Alias2_W(val) => val + 1000,
//    }
//}

fn call_items_2_function() -> u64 {
    alias_2_function()
}

impl Alias2Trait<TestStruct2> for TestStruct1 {
    fn items_2_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 64 && x.W
    }
}


// lib_3 tests

fn project_items_3_struct(input: AltAlias3_Struct) -> u64 {
    input.c
}

fn project_items_3_enum(input: AltAlias3_Enum) -> u64 {
    match input {
	AltAlias3_Enum::E(val) => val,
	AltAlias3_Enum::F(val) => val + 1000,
    }
}

// Aliased enum variants are not recognized as belonging to the enum they're coming from,
// so this test is disabled
//fn project_items_3_variants(input: Items3_Variants) -> u64 {
//    match input {
//	AltAlias3_U(val) => val,
//	AltAlias3_V(val) => val + 1000,
//    }
//}

fn call_items_3_function() -> u64 {
    altalias_3_function()
}

impl AltAlias3Trait<TestStruct2> for TestStruct1 {
    fn items_3_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 64 && x.W
    }
}


// lib_4 tests

fn project_items_4_struct(input: AltAlias4_Struct) -> u64 {
    input.d
}

fn project_items_4_enum(input: Alias4_Enum) -> u64 {
    match input {
	Alias4_Enum::G(val) => val,
	AltAlias4_Enum::H(val) => val + 1000,
    }
}

// Aliased enum variants are not recognized as belonging to the enum they're coming from,
// so this test is disabled
//fn project_items_4_variants(input: Items4_Variants) -> u64 {
//    match input {
//	Alias4_S(val) => val,
//	AltAlias4_T(val) => val + 1000,
//    }
//}

fn call_items_4_function() -> u64 {
    alias_4_function() + alt_alias_4_function()
}


pub fn run_all_tests() -> u64 {
    let items_1_struct = Alias1_Struct { a: 123 };
    let items_1_struct_res = project_items_1_struct(items_1_struct);
    assert(items_1_struct_res == 123);

    let items_1_enum = Alias1_Enum::A(432);
    let items_1_enum_res = project_items_1_enum(items_1_enum);
    assert(items_1_enum_res == 432);

// Alias1_X is recognized as an alias, but it's still impossible to construct a value using Alias1_X
//    let items_1_variants = Alias1_X(680);
//    let items_1_variants_res = project_items_1_variants(items_1_variants);
//    assert(items_1_variants_res == 680);

    let items_1_function_res = call_items_1_function();
    assert(items_1_function_res == ALIAS_1_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_1_trait_teststruct_1_res = teststruct_1.items_1_trait_function(teststruct_2);
    assert(items_1_trait_teststruct_1_res);


    let items_2_struct = Alias2_Struct { b: 123 };
    let items_2_struct_res = project_items_2_struct(items_2_struct);
    assert(items_2_struct_res == 123);

    let items_2_enum = Alias2_Enum::C(432);
    let items_2_enum_res = project_items_2_enum(items_2_enum);
    assert(items_2_enum_res == 432);

    // Alias2_Z is recognized as an alias, but it's still impossible to construct a value using Alias2_Z
//    let items_2_variants = Alias2_Z(680);
//    let items_2_variants_res = project_items_2_variants(items_2_variants);
//    assert(items_2_variants_res == 680);

    let items_2_function_res = call_items_2_function();
    assert(items_2_function_res == ALIAS_2_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_2_trait_teststruct_1_res = teststruct_1.items_2_trait_function(teststruct_2);
    assert(items_2_trait_teststruct_1_res);


    let items_3_struct = AltAlias3_Struct { c: 123 };
    let items_3_struct_res = project_items_3_struct(items_3_struct);
    assert(items_3_struct_res == 123);

    let items_3_enum = AltAlias3_Enum::E(432);
    let items_3_enum_res = project_items_3_enum(items_3_enum);
    assert(items_3_enum_res == 432);

    // AltAlias3_U is recognized as an alias, but it's still impossible to construct a value using AltAlias3_U
//    let items_3_variants = AltAlias3_U(680);
//    let items_3_variants_res = project_items_3_variants(items_3_variants);
//    assert(items_3_variants_res == 680);

    let items_3_function_res = call_items_3_function();
    assert(items_3_function_res == ALTALIAS_3_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_3_trait_teststruct_1_res = teststruct_1.items_3_trait_function(teststruct_2);
    assert(items_3_trait_teststruct_1_res);


    let items_4_struct = AltAlias4_Struct { d: 123 };
    let items_4_struct_res = project_items_4_struct(items_4_struct);
    assert(items_4_struct_res == 123);

    let items_4_enum = AltAlias4_Enum::G(432);
    let items_4_enum_res = project_items_4_enum(items_4_enum);
    assert(items_4_enum_res == 432);

    // AltAlias4_S is recognized as an alias, but it's still impossible to construct a value using AltAlias4_S
//    let items_4_variants = AltAlias4_S(680);
//    let items_4_variants_res = project_items_4_variants(items_4_variants);
//    assert(items_4_variants_res == 680);

    let items_4_function_res = call_items_4_function();
    assert(items_4_function_res == ALTALIAS_4_FUNCTION_RES * 2);


    42
}
