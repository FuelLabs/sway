library;

// Disabled
// This test is supposed to test that `pub use` makes items available when accessed using a path,
// e.g., `::lib_1::X`, but the path resolution is broken, so the test doesn't test what it's
// supposed to test.

use ::items_1::Items1_Variants;

// Helper types

struct TestStruct1 {
    Z: u64,
}

struct TestStruct2 {
    W: bool,
}


// lib_1 tests

impl ::lib_1::Items1Trait<TestStruct2> for TestStruct1 {
    fn items_1_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 64 && x.W
    }
}


// lib_2 tests

impl ::lib_2::Items2Trait<TestStruct2> for TestStruct1 {
    fn items_2_trait_function(self, x: TestStruct2) -> bool {
	self.Z == 128 && x.W
    }
}


pub fn run_all_tests() -> u64 {
    // lib_1 tests
    let items_1_struct = ::lib_1::Items1_Struct { a: 123 };
    assert(items_1_struct.a == 123);

    let items_1_enum = ::lib_1::Items1_Enum::A(432);
    let items_1_enum_res = match items_1_enum {
	::items_1::Items1_Enum::A(val) => val,
	::items_1::Items1_Enum::B(val) => val + 1000,
    };
    assert(items_1_enum_res == 432);

    // TODO: Should this be allowed? ::lib_1::X refers to ::items_1::Items1_Variants::X.
    let items_1_variants = ::lib_1::X(680);
    let items_1_variants_res = match items_1_variants {
	::items_1::Items1_Variants::X(val) => val,
	::items_1::Items1_Variants::Y(val) => val + 1000,
    };
    assert(items_1_variants_res == 680);

    let items_1_function_res = ::lib_1::items_1_function();
    let items_1_function_oracle = ::lib_1::ITEMS_1_FUNCTION_RES;
    assert(items_1_function_res == items_1_function_oracle);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_1_trait_teststruct_1_res = teststruct_1.items_1_trait_function(teststruct_2);
    assert(items_1_trait_teststruct_1_res);

    // lib_2 tests

    let items_2_struct = ::lib_2::Items2_Struct { n: 789 };
    assert(items_2_struct.n == 789);

    let items_2_enum = ::lib_2::Items2_Enum::N(246);
    let items_2_enum_res = match items_2_enum {
	::items_2::Items2_Enum::N(val) => val,
	::items_2::Items2_Enum::M(val) => val + 1000,
    };
    assert(items_2_enum_res == 246);

    let items_2_variants = ::lib_2::O(468);
    let items_2_variants_res = match items_2_variants {
	::items_2::Items2_Variants::O(val) => val,
	::items_2::Items2_Variants::P(val) => val + 1000,
    };
    assert(items_2_variants_res == 468);

    let items_2_function_res = ::lib_2::call_items_2_function();
    let items_2_function_oracle = ::lib_2::ITEMS_2_FUNCTION_RES;
    assert(items_2_function_res == items_2_function_oracle);

    let teststruct_1 = TestStruct1 { Z : 128 };
    let teststruct_2 = TestStruct2 { W : false };
    let items_2_trait_teststruct_1_res = teststruct_1.items_2_trait_function(teststruct_2);
    assert(items_2_trait_teststruct_1_res);

    42
}
