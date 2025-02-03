library;

enum E {
    A: (),
}

impl E {
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    fn assoc_fn() {}
}
