library;

mod pkgb;

pub const TEST_CONST: u64 = 20;

pub fn foo() {
    assert(same_const_name_lib::TEST_CONST == 60);
    assert(same_const_name_lib::pkga::TEST_CONST == 50);
    assert(same_const_name_lib::pkga::pkgb::TEST_CONST == 40);

    assert(pkgb::TEST_CONST == 10);
    assert(TEST_CONST == 20);
}
