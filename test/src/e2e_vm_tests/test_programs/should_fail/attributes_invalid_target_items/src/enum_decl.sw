library;

#[storage(invalid)]
#[inline(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
enum E {
    #[storage(invalid)]
    #[inline(invalid)]
    #[test(invalid)]
    //! Invalid inner comment.
    #[payable(invalid)]
    #[deprecated(invalid)]
    #[fallback(invalid)]
    A: (),
}
