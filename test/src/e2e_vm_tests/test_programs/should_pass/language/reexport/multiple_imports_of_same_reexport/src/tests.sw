library;

// Both lib_1_1.sw and lib_1_2.sw reexport items from items_1.sw. All reexports are item imports.
// Importing both lib_1_1 and lib_1_2 should not cause a name clash.
use ::lib_1_1::*; 
use ::lib_1_2::*;
// Both lib_2_1.sw and lib_2_2.sw reexport items from items_2.sw. All reexports are star imports.
// Importing both lib_2_1 and lib_2_2 should not cause a name clash.
use ::lib_2_1::*; 
use ::lib_2_2::*;
// Both lib_3_1.sw and lib_3_2.sw reexport items from items_3.sw. lib_3_1 star reexports, lib_3_2
// item reexports. Importing both lib_3_1 and lib_3_2 should not cause a name clash.
use ::lib_3_1::*;
use ::lib_3_2::*;
// Both lib_4_1.sw and lib_4_2.sw reexport items from items_4.sw. lib_4_1 item reexports, lib_4_2
// star reexports. Importing both lib_4_1 and lib_4_2 should not cause a name clash.
// This tests that ordering of imports do not matter.
use ::lib_4_1::*;
use ::lib_4_2::*;

use ::items_1::Items1_Variants;
use ::items_2::Items2_Variants;
use ::items_3::Items3_Variants;
use ::items_4::Items4_Variants;

// Helper types

struct TestStruct1 {
    Z: u64,
}

struct TestStruct2 {
    W: bool,
}

// items_1 tests

fn project_items_1_struct(input: Items1_Struct) -> u64 {
    input.a
}

fn project_items_1_enum(input: Items1_Enum) -> u64 {
    match input {
	Items1_Enum::A(val) => val,
	Items1_Enum::B(val) => val + 1000,
    }
}

fn project_items_1_variants(input: Items1_Variants) -> u64 {
    match input {
	X(val) => val,
	Y(val) => val + 1000,
    }
}

fn call_items_1_function() -> u64 {
    items_1_function()
}

impl Items1Trait<TestStruct2> for TestStruct1 {
    fn items_1_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 64 && x.W
    }
}

// items_2 tests

fn project_items_2_struct(input: Items2_Struct) -> u64 {
    input.b
}

fn project_items_2_enum(input: Items2_Enum) -> u64 {
    match input {
	Items2_Enum::C(val) => val,
	Items2_Enum::D(val) => val + 1000,
    }
}

fn project_items_2_variants(input: Items2_Variants) -> u64 {
    match input {
	Z(val) => val,
	W(val) => val + 1000,
    }
}

fn call_items_2_function() -> u64 {
    items_2_function()
}

impl Items2Trait<TestStruct2> for TestStruct1 {
   fn items_2_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 128 && x.W
   }
}

// items_3 tests

fn project_items_3_struct(input: Items3_Struct) -> u64 {
    input.c
}

fn project_items_3_enum(input: Items3_Enum) -> u64 {
    match input {
	Items3_Enum::E(val) => val,
	Items3_Enum::F(val) => val + 1000,
    }
}

fn project_items_3_variants(input: Items3_Variants) -> u64 {
    match input {
	U(val) => val,
	V(val) => val + 1000,
    }
}

fn call_items_3_function() -> u64 {
    items_3_function()
}

impl Items3Trait<TestStruct2> for TestStruct1 {
    fn items_3_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 122 && x.W
    }
}

// items_4 tests

fn project_items_4_struct(input: Items4_Struct) -> u64 {
    input.d
}

fn project_items_4_enum(input: Items4_Enum) -> u64 {
    match input {
	Items4_Enum::G(val) => val,
	Items4_Enum::H(val) => val + 1000,
    }
}

fn project_items_4_variants(input: Items4_Variants) -> u64 {
    match input {
	S(val) => val,
	T(val) => val + 1000,
    }
}

fn call_items_4_function() -> u64 {
    items_4_function()
}

impl Items4Trait<TestStruct2> for TestStruct1 {
    fn items_4_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 122 && x.W
    }
}



pub fn run_all_tests() -> u64 {
    let items_1_struct = Items1_Struct { a: 123 };
    let items_1_struct_res = project_items_1_struct(items_1_struct);
    assert(items_1_struct_res == 123);

    let items_1_enum = Items1_Enum::A(432);
    let items_1_enum_res = project_items_1_enum(items_1_enum);
    assert(items_1_enum_res == 432);

    let items_1_variants = X(680);
    let items_1_variants_res = project_items_1_variants(items_1_variants);
    assert(items_1_variants_res == 680);

    let items_1_function_res = call_items_1_function();
    assert(items_1_function_res == ITEMS_1_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_1_trait_teststruct_1_res = teststruct_1.items_1_trait_function(teststruct_2);
    assert(items_1_trait_teststruct_1_res);

 
    let items_2_struct = Items2_Struct { b: 789 };
    let items_2_struct_res = project_items_2_struct(items_2_struct);
    assert(items_2_struct_res == 789);
 
    let items_2_enum = Items2_Enum::C(246);
    let items_2_enum_res = project_items_2_enum(items_2_enum);
    assert(items_2_enum_res == 246);
 
    let items_2_variants = Z(468);
    let items_2_variants_res = project_items_2_variants(items_2_variants);
    assert(items_2_variants_res == 468);
 
    let items_2_function_res = call_items_2_function();
    assert(items_2_function_res == ITEMS_2_FUNCTION_RES);
 
    let teststruct_1 = TestStruct1 { Z : 128 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_2_trait_teststruct_1_res = teststruct_1.items_2_trait_function(teststruct_2);
    assert(items_2_trait_teststruct_1_res);


    let items_3_struct = Items3_Struct { c: 789 };
    let items_3_struct_res = project_items_3_struct(items_3_struct);
    assert(items_3_struct_res == 789);
 
    let items_3_enum = Items3_Enum::E(246);
    let items_3_enum_res = project_items_3_enum(items_3_enum);
    assert(items_3_enum_res == 246);
 
    let items_3_variants = U(468);
    let items_3_variants_res = project_items_3_variants(items_3_variants);
    assert(items_3_variants_res == 468);
 
    let items_3_function_res = call_items_3_function();
    assert(items_3_function_res == ITEMS_3_FUNCTION_RES);
 
    let teststruct_1 = TestStruct1 { Z : 122 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_3_trait_teststruct_1_res = teststruct_1.items_3_trait_function(teststruct_2);
    assert(items_3_trait_teststruct_1_res);


    let items_4_struct = Items4_Struct { d: 789 };
    let items_4_struct_res = project_items_4_struct(items_4_struct);
    assert(items_4_struct_res == 789);
 
    let items_4_enum = Items4_Enum::G(246);
    let items_4_enum_res = project_items_4_enum(items_4_enum);
    assert(items_4_enum_res == 246);
 
    let items_4_variants = S(468);
    let items_4_variants_res = project_items_4_variants(items_4_variants);
    assert(items_4_variants_res == 468);
 
    let items_4_function_res = call_items_4_function();
    assert(items_4_function_res == ITEMS_4_FUNCTION_RES);
 
    let teststruct_1 = TestStruct1 { Z : 122 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_4_trait_teststruct_1_res = teststruct_1.items_4_trait_function(teststruct_2);
    assert(items_4_trait_teststruct_1_res);

    42
}
