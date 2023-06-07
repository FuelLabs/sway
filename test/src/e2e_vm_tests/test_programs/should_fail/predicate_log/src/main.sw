predicate;

use std::{
    inputs::input_owner,
    logging::log,
};

fn main() -> bool {
    log::<Address>(input_owner(0).unwrap());
        
    true
}