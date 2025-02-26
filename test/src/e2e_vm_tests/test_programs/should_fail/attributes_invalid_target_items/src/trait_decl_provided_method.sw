library;

trait T {
} {
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[fallback(invalid)]
    fn trait_provided_method(self) {}
}
