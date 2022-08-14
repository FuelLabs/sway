script;

fn mut_arg(ref mut b: bool) {
    b = true;
}

fn main() -> bool {
    let mut b = false;
    mut_arg(b);
    b
}
