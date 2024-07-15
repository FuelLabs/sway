library;

// Reexported items from ext_1_items. All reexports are item imports.
use ext_1_lib::*;
use ext_1_items::Items1_Variants;
// Reexported items from ext_2_items. All reexports are star imports.
use ext_2_lib::*;
// Reexported items from ext_3_items. All reexports are item imports.
use ext_3_lib::Items3_Struct;
use ext_3_lib::Items3_Enum;
// ext_3_lib elevates U and V to the same namespace as the type names, so Items3_Variants cannot be found in ext_3_lib.
//use ext_3_lib::Items3_Variants::U;
//use ext_3_lib::Items3_Variants::V;
use ext_3_lib::U;
use ext_3_lib::V;
use ext_3_lib::ITEMS_3_FUNCTION_RES;
use ext_3_lib::items_3_function;
use ext_3_lib::Items3Trait;
use ext_3_items::Items3_Variants;
// Reexported items from ext_4_items. All reexports are star imports.
use ext_4_lib::Items4_Struct;
use ext_4_lib::Items4_Enum;
// ext_4_lib elevates S and T to the same namespace as the type names, so Items4_Variants cannot be found in ext_4_lib.
//use ext_4_lib::Items4_Variants::S;
//use ext_4_lib::Items4_Variants::T;
use ext_4_lib::S;
use ext_4_lib::T;
use ext_4_lib::ITEMS_4_FUNCTION_RES;
use ext_4_lib::items_4_function;
use ext_4_lib::Items4Trait;
use ext_4_items::Items4_Variants;
// Reexported items from ext_5_items through two libraries.
use ext_5_1_lib::*;
use ext_5_2_lib::*;


// Helper types

struct TestStruct1 {
    Z: u64,
}

struct TestStruct2 {
    W: bool,
}


// ext_1_lib tests

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


// ext_2_lib tests

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
	self.Z == 64 && x.W
    }
}


// ext_3_lib tests

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
	self.Z == 64 && x.W
    }
}


// ext_4_lib tests

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
	self.Z == 64 && x.W
    }
}


// ext_5_lib tests

fn project_items_5_struct(input: Items5_Struct) -> u64 {
    input.e
}

fn project_items_5_enum(input: Items5_Enum) -> u64 {
    match input {
	Items5_Enum::I(val) => val,
	Items5_Enum::J(val) => val + 1000,
    }
}

fn project_items_5_variants(input: Items5_Variants) -> u64 {
    match input {
	Q(val) => val,
	R(val) => val + 1000,
    }
}

fn call_items_5_function() -> u64 {
    items_5_function()
}

impl Items5Trait<TestStruct2> for TestStruct1 {
    fn items_5_trait_function(self, x: TestStruct2) -> bool {
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


    let items_2_struct = Items2_Struct { b: 223 };
    let items_2_struct_res = project_items_2_struct(items_2_struct);
    assert(items_2_struct_res == 223);

    let items_2_enum = Items2_Enum::C(432);
    let items_2_enum_res = project_items_2_enum(items_2_enum);
    assert(items_2_enum_res == 432);

    let items_2_variants = Z(680);
    let items_2_variants_res = project_items_2_variants(items_2_variants);
    assert(items_2_variants_res == 680);

    let items_2_function_res = call_items_2_function();
    assert(items_2_function_res == ITEMS_2_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_2_trait_teststruct_1_res = teststruct_1.items_2_trait_function(teststruct_2);
    assert(items_2_trait_teststruct_1_res);


    let items_3_struct = Items3_Struct { c: 323 };
    let items_3_struct_res = project_items_3_struct(items_3_struct);
    assert(items_3_struct_res == 323);

    let items_3_enum = Items3_Enum::E(432);
    let items_3_enum_res = project_items_3_enum(items_3_enum);
    assert(items_3_enum_res == 432);

    let items_3_variants = U(680);
    let items_3_variants_res = project_items_3_variants(items_3_variants);
    assert(items_3_variants_res == 680);

    let items_3_function_res = call_items_3_function();
    assert(items_3_function_res == ITEMS_3_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_3_trait_teststruct_1_res = teststruct_1.items_3_trait_function(teststruct_2);
    assert(items_3_trait_teststruct_1_res);


    let items_4_struct = Items4_Struct { d: 424 };
    let items_4_struct_res = project_items_4_struct(items_4_struct);
    assert(items_4_struct_res == 424);

    let items_4_enum = Items4_Enum::G(442);
    let items_4_enum_res = project_items_4_enum(items_4_enum);
    assert(items_4_enum_res == 442);

    let items_4_variants = S(680);
    let items_4_variants_res = project_items_4_variants(items_4_variants);
    assert(items_4_variants_res == 680);

    let items_4_function_res = call_items_4_function();
    assert(items_4_function_res == ITEMS_4_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_4_trait_teststruct_1_res = teststruct_1.items_4_trait_function(teststruct_2);
    assert(items_4_trait_teststruct_1_res);


    let items_5_struct = Items5_Struct { e: 525 };
    let items_5_struct_res = project_items_5_struct(items_5_struct);
    assert(items_5_struct_res == 525);

    let items_5_enum = Items5_Enum::I(552);
    let items_5_enum_res = project_items_5_enum(items_5_enum);
    assert(items_5_enum_res == 552);

    let items_5_variants = Q(680);
    let items_5_variants_res = project_items_5_variants(items_5_variants);
    assert(items_5_variants_res == 680);

    let items_5_function_res = call_items_5_function();
    assert(items_5_function_res == ITEMS_5_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_5_trait_teststruct_1_res = teststruct_1.items_5_trait_function(teststruct_2);
    assert(items_5_trait_teststruct_1_res);


    42
}
