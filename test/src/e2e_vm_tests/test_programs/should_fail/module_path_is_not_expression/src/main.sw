script;

mod lib;

use lib::*;

fn main() { 
    let _ = lib;
    let _ = lib::;
    let _ = lib::submodule;
    let _ = lib::
                submodule;

    f(lib);
    f(lib::);
    f(lib::submodule);
    f(lib::
        submodule);

    lib = 0;
    lib:: = 0;
    lib::submodule = 0;
    lib::
        submodule = 0;
}

fn f(_x: u8) { }

