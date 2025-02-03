library;

trait T {
} {
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    fn trait_provided_method(self) {}
}
