script;

mod foo;

use foo::*;

fn main() -> u64 {
    bar::func()
}
