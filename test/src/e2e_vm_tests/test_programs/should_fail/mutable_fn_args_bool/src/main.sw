script;

fn mut_arg(mut b: bool) {
    b = true;
}

fn main() -> bool {
    let mut b = false;
    mut_arg(b);
    b
}
