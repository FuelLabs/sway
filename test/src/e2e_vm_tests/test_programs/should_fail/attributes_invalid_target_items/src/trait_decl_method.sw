library;

trait T {
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
    fn trait_method(self);
} 
