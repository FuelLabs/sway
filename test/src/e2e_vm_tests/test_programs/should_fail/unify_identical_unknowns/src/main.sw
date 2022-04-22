script;

pub trait Foo {
    fn bar(self, other: Self) -> bool;
}

impl Foo for NonExistant {
    fn bar(self, other: Self) -> bool {
        false
    }
}

fn main() { }
