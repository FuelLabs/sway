library;

#[storage(invalid)]
#[inline(invalid)]
#[trace(invalid)]
//! Invalid inner comment.
#[test(invalid)]
#[payable(invalid)]
#[fallback(invalid)]
#[error_type(invalid)]
#[error(invalid)]
struct S {
    #[storage(invalid)]
    #[inline(invalid)]
    #[trace(invalid)]
    //! Invalid inner comment.
    #[test(invalid)]
    #[payable(invalid)]
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    field: u8,
}
