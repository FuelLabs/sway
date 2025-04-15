library;

#[error_type]
enum OkE {
    #[error(m = "ok")]
    A: (),
}

#[error_type]
enum NotOkE {
    #[error(msg = "not ok")]
    A: (),
}
