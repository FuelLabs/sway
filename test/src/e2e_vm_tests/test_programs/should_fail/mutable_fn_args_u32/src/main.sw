script;

fn mut_arg(mut b: u32) {
    b = 20;
}

fn main() -> u32 {
    let mut b = 0u32;
    mut_arg(b);
    b
}
