script;

dep bar;

// This is okay
use ::bar::{Bar1, Bar1 as Foo};

// This is not okay because symbol `Foo` is already reserved
use ::bar::Bar2 as Foo;

// This is not okay
use ::bar::{Bar2, Bar2};

// This okay. Although Bar1 + Bar2 have already been imported, glob imports don't cause shadow errors.
use ::bar::*;

fn main() -> bool {
    false
}
