library;

mod pkga;

pub const TEST_CONST: u64 = 60;

pub fn foo() {
    pkga::pkgb::bar();
    pkga::foo();
    assert(pkga::pkgb::TEST_CONST == 40);
    assert(pkga::TEST_CONST == 50);
    assert(TEST_CONST == 60);
}
