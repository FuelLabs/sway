library;

#[abi_name(name = "name")]
pub struct Ok1 { }

#[abi_name(name = "name", name = "other name")]
pub struct NotOk1 { }
