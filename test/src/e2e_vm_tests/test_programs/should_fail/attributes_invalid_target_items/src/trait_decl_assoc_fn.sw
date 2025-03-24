library;

trait T {
    #[inline(invalid)]
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    fn trait_assoc_fn();
} 

