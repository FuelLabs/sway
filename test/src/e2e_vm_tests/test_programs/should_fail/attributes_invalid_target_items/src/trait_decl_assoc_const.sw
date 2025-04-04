library;

#[storage(invalid)]
#[inline(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
#[error_type(invalid)]
#[error(invalid)]
trait T {
    #[storage(invalid)]
    #[inline(invalid)]
    #[test(invalid)]
    //! Invalid inner comment.
    #[payable(invalid)]
    #[deprecated(invalid)]
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    const TRAIT_CONST: u8 = 0;
} 
