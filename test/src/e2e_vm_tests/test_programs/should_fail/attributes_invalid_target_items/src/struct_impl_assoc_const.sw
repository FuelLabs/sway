library;

struct S {
    field: u8,
}

#[storage(invalid)]
#[inline(invalid)]
#[trace(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
#[error_type(invalid)]
#[error(invalid)]
impl S {
    #[storage(invalid)]
    #[inline(invalid)]
    #[trace(invalid)]
    #[test(invalid)]
    //! Invalid inner comment.
    #[payable(invalid)]
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    const ASSOC_CONST: u8 = 0;
}
