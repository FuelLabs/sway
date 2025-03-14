library;

trait T {
    #[inline(invalid)]
    #[test(invalid)]
    //! Invalid inner comment.
    #[payable(invalid)]
    #[deprecated(invalid)]
    #[fallback(invalid)]
    fn trait_method(self);
} 
