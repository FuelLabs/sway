script;
// This tests importing other files.

mod foo;
mod bar;
mod wiz;

use foo::Foo as MyFoo;

use foo::Baz; // Refers to bar::Bar, but foo re-exports using the alias Baz

use wiz::Wiz as WizWiz;

// This is fine - the imported Wiz name is aliased, so no name clash
struct Wiz { 
    local_wiz: bool
}

fn main() -> u64 {
    let foo = MyFoo {
        foo: 42,
    };
    let bar = Baz {
	bar: 64,
    };
    let wiz = WizWiz {
	wiz: 128
    };
    let local_wiz = Wiz { // This should resolve to the locally defined Wiz
	local_wiz: true
    };
    if local_wiz.local_wiz {
	foo.foo + bar.bar + wiz.wiz
    }
    else {
	0
    }
}
