script;

mod traits;

use traits::Foo;

struct Bar {}

fn main() -> u64 {
    let bar = Bar {};
    bar.foo()
    
}

