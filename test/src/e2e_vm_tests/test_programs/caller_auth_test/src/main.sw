script;
use std::chain::auth::caller_is_external;

// should be false in the case of a script
fn main() -> bool {
   caller_is_external()
}
