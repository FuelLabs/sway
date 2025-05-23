library;

#[error_type]
enum Ok {
    #[error(m = "ok")]
    A: (),
}

#[error_type]
enum NotOk {
    #[error(m = "not ok")]
    #[error(m = "not ok")]
    A: (),
}