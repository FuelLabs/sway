script;

mod r#trait;
mod utils;

use r#trait::Trait;

struct Foo {

}

impl Trait for Foo {

}

fn main() {
    utils::uses_trait(Foo{});
}
