library;

use ::lib_1::*; // Items from items_1.sw not reexported. All imports are item imports
use ::lib_2::*; // Items from items_2.sw not reexported. All imports are glob imports

use ::items_1::Items1_Variants;


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
    input.n
}

fn project_items_2_enum(input: Items2_Enum) -> u64 {
    match input {
        Items2_Enum::N(val) => val,
        Items2_Enum::M(val) => val + 1000,
    }
}

fn project_items_2_variants(input: Items2_Variants) -> u64 {
    match input {
        O(val) => val,
        P(val) => val + 1000,
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
    poke(items_1_struct_res == 123);

    let items_1_enum = Items1_Enum::A(432);
    let items_1_enum_res = project_items_1_enum(items_1_enum);
    poke(items_1_enum_res == 432);

    let items_1_variants = X(680);
    let items_1_variants_res = project_items_1_variants(items_1_variants);
    poke(items_1_variants_res == 680);

    let items_1_function_res = call_items_1_function();
    poke(items_1_function_res == ITEMS_1_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_1_trait_teststruct_1_res = teststruct_1.items_1_trait_function(teststruct_2);
    poke(items_1_trait_teststruct_1_res);


    let items_2_struct = Items2_Struct { n: 789 };
    let items_2_struct_res = project_items_2_struct(items_2_struct);
    poke(items_2_struct_res == 789);

    let items_2_enum = Items2_Enum::N(246);
    let items_2_enum_res = project_items_2_enum(items_2_enum);
    poke(items_2_enum_res == 246);

    let items_2_variants = O(468);
    let items_2_variants_res = project_items_2_variants(items_2_variants);
    poke(items_2_variants_res == 468);

    let items_2_function_res = call_items_2_function();
    poke(items_2_function_res == ITEMS_2_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 128 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_2_trait_teststruct_1_res = teststruct_1.items_2_trait_function(teststruct_2);
    poke(items_2_trait_teststruct_1_res);

    42
}

fn poke(b: bool) { }