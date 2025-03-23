script;

mod bar;

// This should not compile but `use ::bar::*;` should
use ::bar::{* as all}; <-- warning: unused import

fn main() -> bool {
    false
}
