library;

#[allow(dead_code)]
#[error_type]
pub enum EmptyErrorTypeEnum { }

#[allow(dead_code)]
#[error_type]
pub enum EmptyErrorMessages {
    #[error(m = "ok")]
    A: (),
    #[error(m = "")]
    B: (),
    #[error(m = "also ok")]
    C: (),
    #[error(m = "")]
    D: (),
}

#[allow(dead_code)]
#[error_type]
pub enum DuplicatedErrorMessages {
    #[error(m = "k")]
    K: (),
    #[error(m = "duplicated trice")]
    L: (),
    #[error(m = "duplicated twice")]
    E: (),
    #[error(m = "a")]
    A: (),
    #[error(m = "duplicated once")]
    B: (),
    #[error(m = "c")]
    C: (),
    #[error(m = "duplicated once")]
    D: (),
    #[error(m = "f")]
    F: (),
    #[error(m = "h")]
    H: (),
    #[error(m = "duplicated twice")]
    G: (),
    #[error(m = "o")]
    O: (),
    #[error(m = "duplicated trice")]
    P: (),
    #[error(m = "duplicated twice")]
    I: (),
    #[error(m = "duplicated trice")]
    J: (),
    #[error(m = "m")]
    M: (),
    #[error(m = "duplicated trice")]
    N: (),
}