library;

#[allow(dead_code)]
pub fn ok_1() {}

#[allow(dead_code, deprecated)]
pub fn ok_2() {}

#[allow]
#[allow()]
pub fn not_ok() {}