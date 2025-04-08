library;

#[error_type]
enum Ok {
    #[error(m = "ok")]
    A: (),
}

#[error_type(invalid)] // Should be no invalid arg error or warning here.
enum NotOk {
    #[error(m = "ok")]
    A: (),
}
