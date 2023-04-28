library;

pub mod pkgb;

pub const TEST_CONST: u64 = 50;

pub fn foo() {
    assert(pkgb::TEST_CONST == 40);
    assert(TEST_CONST == 50);
}
