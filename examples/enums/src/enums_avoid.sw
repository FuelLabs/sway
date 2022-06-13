library enums_avoid;

dep enum_of_enums;
use enum_of_enums::{Error, StateError, UserError};

fn avoid() {
    let error1 = Error::StateError(StateError::Void);
    let error2 = Error::UserError(UserError::Unauthorized);
}
