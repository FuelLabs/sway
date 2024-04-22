predicate;

use std::{
    inputs::input_coin_owner,
    logging::log,
};

fn main() -> bool {
    log::<Address>(input_coin_owner(0).unwrap());
        
    true
}