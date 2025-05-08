library;

#[allow(dead_code)]
#[error_type]
pub enum ErrorEnumWithNonErrorVariant {
    #[error(m = "ok")]
    A: (),
    #[error(m = "also ok")]
    B: (),
    NotOk: (),
}

#[allow(dead_code)]
#[error_type]
pub enum ErrorEnumWithTwoNonErrorVariants {
    NotOk1: (),
    #[error(m = "ok")]
    A: (),
    NotOk2: (),
    #[error(m = "also ok")]
    B: (),
}

#[allow(dead_code)]
#[error_type]
pub enum ErrorEnumWithMoreNonErrorVariants {
    NotOk1: (),
    #[error(m = "ok")]
    A: (),
    NotOk2: (),
    NotOk3: (),
    #[error(m = "also ok")]
    B: (),
    NotOk4: (),
    NotOk5: (),
}

#[allow(dead_code)]
pub enum ErrorAttributeInNonErrorEnum {
    #[error(m = "this error is in non-error type enum")]
    NotOk: (),
}