library enums_preferred;

dep enum_of_enums;
use enum_of_enums::{StateError, UserError};

// ANCHOR: content
fn preferred() {
    let error1 = StateError::Void;
    let error2 = UserError::Unauthorized;
}
// ANCHOR_END: content
