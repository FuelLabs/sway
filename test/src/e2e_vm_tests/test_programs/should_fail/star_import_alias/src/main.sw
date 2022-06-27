script;

dep bar;

// This should not compile but `use ::bar::*;` should
use ::bar::{* as all};

fn main() -> bool {
    false
}
