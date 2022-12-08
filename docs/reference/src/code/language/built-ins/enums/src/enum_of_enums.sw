library enum_of_enums;

// ANCHOR: content
enum UserError {
    InsufficientPermissions: (),
    Unauthorized: (),
}

enum Error {
    UserError: UserError,
}

fn main() {
    let my_enum = Error::UserError(UserError::Unauthorized);
}
// ANCHOR_END: content
