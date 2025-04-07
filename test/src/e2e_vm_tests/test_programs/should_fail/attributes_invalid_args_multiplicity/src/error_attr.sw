library;

#[error_type]
enum Ok {
    #[error(m = "ok")]
    A: (),
}

#[error_type]
enum NotOk1 {
    #[error]
    A: (),
}

#[error_type]
enum NotOk2 {
    #[error()]
    A: (),
}

#[error_type]
enum NotOk3 {
    #[error(m = "ok", m = "not ok")]
    A: (),
}