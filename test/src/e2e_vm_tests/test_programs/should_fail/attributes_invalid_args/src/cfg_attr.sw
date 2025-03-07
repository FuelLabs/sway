library;

#[cfg(target = "fuel")]
pub fn ok_1() {}

#[cfg(program_type = "library")]
pub fn ok_2() {}

#[cfg(trget = "fuel")]
// If any `cfg` has an error, the whole item is ignored including the attributes.
// This means we don't expect invalid argument errors for the below two cases.
#[cfg(program_typ = "library")]
#[cfg(unknown_arg = "unknown")]
pub fn not_ok() {}