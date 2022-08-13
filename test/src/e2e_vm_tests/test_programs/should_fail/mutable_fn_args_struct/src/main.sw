script;

struct S{}

fn mut_arg(mut s: S) {}

fn main() -> u32 {
    let mut s = S{};
    mut_arg(s);
    0
}
