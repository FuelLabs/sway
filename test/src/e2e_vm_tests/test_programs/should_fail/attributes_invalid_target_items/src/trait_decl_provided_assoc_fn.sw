library;

trait T {
} {
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    fn trait_provided_fn() {}
}
