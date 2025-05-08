library;

enum E {
    A: (),
}

impl E {
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    fn assoc_fn() {}
}
