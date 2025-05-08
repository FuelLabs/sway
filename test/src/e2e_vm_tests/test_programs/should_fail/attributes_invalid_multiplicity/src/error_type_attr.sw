library;

#[error_type]
enum Ok {
    #[error(m = "ok")]
    A: (),
}

#[error_type]
#[error_type]
#[error_type]
enum NotOk {
    #[error(m = "ok")]
    A: (),
}