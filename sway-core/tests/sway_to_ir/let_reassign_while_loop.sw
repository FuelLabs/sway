script;

fn main() -> bool {
    let mut a = true;
    while a {
        a = a && false;
    }
    a
}
