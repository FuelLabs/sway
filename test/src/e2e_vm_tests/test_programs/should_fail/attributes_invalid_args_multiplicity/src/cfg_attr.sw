library;

#[cfg(target = "fuel")]
pub fn ok_1() {}

#[cfg]
#[cfg()]
#[cfg(target = "fuel", program_type = "library")]
pub fn not_ok() {}