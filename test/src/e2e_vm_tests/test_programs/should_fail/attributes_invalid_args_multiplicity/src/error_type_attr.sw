library;

#[error_type]
enum Ok1 {
    #[error(m = "ok")]
    A: (),
}

#[error_type()]
enum Ok2 {
    #[error(m = "ok")]
    A: (),
}

#[error_type(invalid)]
enum NotOk {
    #[error(m = "ok")]
    A: (),
}