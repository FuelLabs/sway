library;

trait T {
    #[inline(invalid)]
    #[trace(invalid)]
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    fn trait_assoc_fn();
} 

