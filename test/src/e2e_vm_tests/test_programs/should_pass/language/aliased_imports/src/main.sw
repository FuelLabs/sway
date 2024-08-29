script;
// This tests importing other files.

mod foo;
mod wiz;

use foo::Foo as MyFoo;

use wiz::Wiz as WizWiz;

// This is fine - the imported Wiz name is aliased, so no name clash
struct Wiz { 
    local_wiz: bool
}

fn main() -> u64 {
    let foo = MyFoo {
        foo: 42,
    };
    let wiz = WizWiz {
	wiz: 128
    };
    let local_wiz = Wiz { // This should resolve to the locally defined Wiz
	local_wiz: true
    };
    if local_wiz.local_wiz {
	foo.foo + wiz.wiz
    }
    else {
	0
    }
}
