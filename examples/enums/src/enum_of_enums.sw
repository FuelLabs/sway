library enum_of_enums;

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
