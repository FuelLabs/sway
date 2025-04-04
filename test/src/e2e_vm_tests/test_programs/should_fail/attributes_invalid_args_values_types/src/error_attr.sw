library;

#[allow(dead_code)]
#[error_type]
pub enum NotOk {
    #[error(m = 42)]
    A: (),
}