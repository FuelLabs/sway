script;

// this module will be accessible outside the library
pub mod alpha;
// this module will not be accessible outside the library
mod beta;

use ::alpha::foo as foo1;

// Error: ::alpha::bar is private
use ::alpha::bar as bar1;

use ::beta::foo as foo2;

// Error: ::beta::bar is private
use ::beta::bar as bar2;

// Error: ::beta::gamma is private
use ::beta::gamma::foo as foo3;

// Error: ::beta::gamma is private
use ::beta::gamma::bar as bar3;

// Error: ::beta::gamma is private
use ::beta::gamma::*;


fn main() {
    ::alpha::foo();

    // Error: ::alpha::bar is private
    // ::alpha::bar();

    ::beta::foo();

    // Error: ::beta::bar is private
    ::beta::bar();

    // Error: ::beta::gamma is private
    ::beta::gamma::foo();

    // Error: ::beta::gamma is private
    ::beta::gamma::bar();

    // ::beta::gamma_foo();
    
}
