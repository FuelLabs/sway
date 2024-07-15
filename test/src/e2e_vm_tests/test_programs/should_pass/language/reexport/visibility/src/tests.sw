library;

use ::lib_1_1::*; // Reexported items from items_1.sw. 
use ::lib_1_2::*; // Imported but not reexported items from items_1.sw. 
use ::lib_2_1::*; // Imported but not reexported items from items_2.sw.
use ::lib_2_2::*; // Reexported items from items_2.sw.

use ::items_1::Items1_Variants;
use ::items_2::Items2_Variants;


// Helper types

struct TestStruct1 {
    Z: u64,
}

struct TestStruct2 {
    W: bool,
}


// lib_1 tests

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


// lib_2 tests

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

    42
}
