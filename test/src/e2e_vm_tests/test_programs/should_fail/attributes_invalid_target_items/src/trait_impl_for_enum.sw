library;

trait T {
    type Type;
} 

enum E {
    A: (),
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
impl T for E {
    #[storage(invalid)]
    #[inline(invalid)]
    #[trace(invalid)]
    #[test(invalid)]
    //! Invalid inner comment.
    #[payable(invalid)]
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    #[event]
    #[indexed]
    type Type = u8;
}
