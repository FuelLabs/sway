library;

#[storage(invalid)]
#[inline(invalid)]
//! Invalid inner comment.
#[test(invalid)]
#[payable(invalid)]
#[fallback(invalid)]
struct S {
    #[storage(invalid)]
    #[inline(invalid)]
    //! Invalid inner comment.
    #[test(invalid)]
    #[payable(invalid)]
    #[fallback(invalid)]
    field: u8,
}
