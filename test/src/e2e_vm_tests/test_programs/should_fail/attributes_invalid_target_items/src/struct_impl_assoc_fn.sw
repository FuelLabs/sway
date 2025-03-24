library;

struct S {
    field: u8,
}

impl S {
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[fallback(invalid)]
    fn assoc_fn() {}
}
