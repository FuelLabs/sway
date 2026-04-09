library;

// ANCHOR: style_enums
pub enum Error {
    StateError: StateError,
    UserError: UserError,
}

pub enum StateError {
    Void: (),
    Pending: (),
    Completed: (),
}

pub enum UserError {
    InsufficientPermissions: (),
    Unauthorized: (),
}
// ANCHOR_END: style_enums

fn preferred() {
    // ANCHOR: use
    let error1 = StateError::Void;
    let error2 = UserError::Unauthorized;
    // ANCHOR_END: use
}

fn avoid() {
    // ANCHOR: avoid
    let error1 = Error::StateError(StateError::Void);
    let error2 = Error::UserError(UserError::Unauthorized);
    // ANCHOR_END: avoid
}
