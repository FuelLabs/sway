script;

dep bar;

// This is okay
use ::bar::{Bar1, Bar1 as Foo};

// This is not okay because symbol `Foo` is already reserved
use ::bar::Bar2 as Foo;

// This is not okay
use ::bar::{Bar2, Bar2};

// This not okay now because Bar1 and Bar2 have already been imported
use ::bar::*;

fn main() -> bool {
    false
}
