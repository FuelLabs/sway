script;

mod pkga;

const TEST_CONST: u64 = 30;

fn main() {
    same_const_name_lib::foo();

    pkga::pkgb::bar();
    same_const_name_lib::pkga::pkgb::bar();

    pkga::foo();
    same_const_name_lib::pkga::foo();

    assert(pkga::pkgb::TEST_CONST == 10);
    assert(same_const_name_lib::pkga::pkgb::TEST_CONST == 40);

    assert(pkga::TEST_CONST == 20);
    assert(same_const_name_lib::pkga::TEST_CONST == 50);

    assert(TEST_CONST == 30);
    assert(same_const_name_lib::TEST_CONST == 60);
}
