library;

#[allow(dead_code)]
pub fn ok_1() {}

#[allow(dead_code, deprecated)]
pub fn ok_2() {}

#[allow(ded_code)]
#[allow(unknown_arg_1, unknown_arg_2)]
pub fn not_ok() {}