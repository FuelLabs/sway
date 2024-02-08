pub const REGISTERS: [&str; 16] = [
    "zero", "one", "of", "pc", "ssp", "sp", "fp", "hp", "err", "ggas", "cgas", "bal", "is", "ret",
    "retl", "flag",
];

pub fn register_name(index: usize) -> String {
    if index < REGISTERS.len() {
        REGISTERS[index].to_owned()
    } else {
        format!("reg{index}")
    }
}

pub fn register_index(name: &str) -> Option<usize> {
    REGISTERS.iter().position(|&n| n == name)
}
