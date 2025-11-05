library;

trait T {
    type Type;
} 

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
#[event]
#[indexed]
impl T for S {
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
    #[event]
    #[indexed]
    type Type = u8;
}
