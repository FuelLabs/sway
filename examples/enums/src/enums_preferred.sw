library enums_preferred;

dep enum_of_enums;
use enum_of_enums::{StateError, UserError};

fn preferred() {
    let error1 = StateError::Void;
    let error2 = UserError::Unauthorized;
}
