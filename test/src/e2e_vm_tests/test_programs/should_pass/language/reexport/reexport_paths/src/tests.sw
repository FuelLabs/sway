library;

// Items from items_1.sw reexported via lib_1_1 and lib_1_2.
use ::lib_1_2::*;
// Reexport of std::hash::*, which is not part of the std prelude.
use ::lib_2::*;
// Reexports of std::ecr::EcRecoverError, which is not part of the std prelude.
use ::lib_3_1::*;
use ::lib_3_2::*;
// Reexport of std::registers::*, which is not part of the std prelude.
use ::lib_4::global_gas;
// Reexport of core::codec::*, which is part of the std prelude.
use ::lib_5::Buffer;
// Reexports of std::address::Address, one via std::prelude and one directly from std::address
// Importing from std::prelude causes an error, so that part of the test is disabled for now.
//use ::lib_6_1::*;
use ::lib_6_2::*;


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

fn mk_hasher() -> Hasher {
    Hasher::new()
}

impl Hash for TestStruct1 {
    fn hash(self, ref mut _state: Hasher) {
    }
}


// lib_3 tests

fn mk_ec_recover_error() -> EcRecoverError {
    EcRecoverError::UnrecoverablePublicKey
}


// lib_4 tests

fn get_global_gas() -> u64 {
    global_gas()
}


// lib_5 tests

fn mk_buffer() -> Buffer {
    Buffer::new()
}


// lib_6 tests

fn mk_address() -> Address {
    Address::zero()
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


    let hasher = mk_hasher();
    teststruct_1.hash(hasher);


    let _ = mk_ec_recover_error();


    let _ = get_global_gas();


    let _ = mk_buffer();


    let _ = mk_address();

    42
}
