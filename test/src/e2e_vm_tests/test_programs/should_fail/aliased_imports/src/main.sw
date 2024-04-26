script;

mod foo;
mod bar;

use foo::Bar; // Should not work. foo re-exports bar::Bar, but aliased to Baz

fn main() {
    let bar = Bar {
	bar : 42
    };
    bar.bar
}
