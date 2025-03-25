library;

#[storage(invalid)]
#[inline(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[fallback(invalid)]
enum E {
    #[storage(invalid)]
    #[inline(invalid)]
    #[test(invalid)]
    //! Invalid inner comment.
    #[payable(invalid)]
    #[fallback(invalid)]
    A: (),
}
