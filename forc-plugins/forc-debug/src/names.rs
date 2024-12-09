/// A list of predefined register names mapped to their corresponding indices.
pub const REGISTERS: [&str; 16] = [
    "zero", "one", "of", "pc", "ssp", "sp", "fp", "hp", "err", "ggas", "cgas", "bal", "is", "ret",
    "retl", "flag",
];

/// Returns the name of a register given its index.
///
/// If the index corresponds to a predefined register, the corresponding name
/// from `REGISTERS` is returned. Otherwise, it returns a formatted name
/// like `"reg{index}"`.
///
/// # Examples
///
/// ```
/// use forc_debug::names::register_name;
/// assert_eq!(register_name(0), "zero".to_string());
/// assert_eq!(register_name(15), "flag".to_string());
/// ```
pub fn register_name(index: usize) -> String {
    if index < REGISTERS.len() {
        REGISTERS[index].to_owned()
    } else {
        format!("reg{index}")
    }
}

/// Returns the index of a register given its name.
///
/// If the name matches a predefined register in `REGISTERS`, the corresponding
/// index is returned. Otherwise, returns `None`.
///
/// # Examples
///
/// ```
/// use forc_debug::names::register_index;
/// assert_eq!(register_index("zero"), Some(0));
/// assert_eq!(register_index("flag"), Some(15));
/// assert_eq!(register_index("unknown"), None);
/// ```
pub fn register_index(name: &str) -> Option<usize> {
    REGISTERS.iter().position(|&n| n == name)
}
