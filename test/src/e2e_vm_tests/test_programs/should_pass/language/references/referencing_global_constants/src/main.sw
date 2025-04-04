script;

mod lib;

#[inline(never)]
fn get_addr_x() -> u64 {
    let x_addr = &lib::TEST.x;
    lib::reference_to_int(x_addr)
}

#[inline(never)]
fn get_addr_y() -> u64 {
    let y_addr = &lib::TEST.y;
    lib::reference_to_int(y_addr)
}

#[inline(never)]
fn sum_x_y_addresses() -> u64 {
   get_addr_x() + get_addr_y()
}

fn main() -> u64 {
   0
}

#[test]
fn test_x_y_addr() {
   assert(get_addr_x() + get_addr_y() == sum_x_y_addresses());
}

#[test]
fn test_across_modules() {
   assert(lib::get_addr_x() == get_addr_x());
   assert(lib::get_addr_y() == get_addr_y());
   assert(lib::get_addr_x() + lib::get_addr_y() == sum_x_y_addresses());

   assert(lib::get_addr_u64() == lib::reference_to_int(&lib::TEST_U64));
   assert(lib::get_addr_bool() == lib::reference_to_int(&lib::TEST_BOOL));
   assert(lib::get_addr_array() == lib::reference_to_int(&lib::TEST_ARRAY));
   assert(lib::get_addr_tuple() == lib::reference_to_int(&lib::TEST_TUPLE));
}

#[test]
fn test_associated_const() {
   // TODO: Associated consts don't work right yet.
   // assert(lib::get_associated_const() == lib::reference_to_int(&lib::S1::ASSOCIATED_CONST));
}
