library;

trait T {
    type Type;
} 

enum E {
    A: (),
}

#[storage(invalid)]
#[inline(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
impl T for E {
    #[storage(invalid)]
    #[inline(invalid)]
    #[test(invalid)]
    //! Invalid inner comment.
    #[payable(invalid)]
    #[fallback(invalid)]
    type Type = u8;
}
