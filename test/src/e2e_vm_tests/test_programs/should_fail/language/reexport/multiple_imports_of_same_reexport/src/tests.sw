library;

// Both lib_1_1.sw and lib_1_2.sw reexport items from items_1.sw. All reexports are star imports.
// Importing individual items from both lib_1_1 and lib_1_2 causes a name clash.
// (The fact that they refer to the same item is irrelevant, since it is also an error to import the
// same item twice from the same source)
use ::lib_1_1::Items1_Struct; 
use ::lib_1_1::Items1_Enum;
use ::lib_1_1::Items1_Variants::X; 
use ::lib_1_1::Items1_Variants::Y; 
use ::lib_1_1::ITEMS_1_FUNCTION_RES; 
use ::lib_1_1::items_1_function; 
use ::lib_1_1::Items1Trait;

use ::lib_1_2::Items1_Struct;
use ::lib_1_2::Items1_Enum;
use ::lib_1_2::Items1_Variants::X;
use ::lib_1_2::Items1_Variants::Y;
use ::lib_1_2::ITEMS_1_FUNCTION_RES;
use ::lib_1_2::items_1_function;
use ::lib_1_2::Items1Trait;

use ::items_1::Items1_Variants;

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

    42
}
