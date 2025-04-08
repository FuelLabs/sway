library;

struct S {
    field: u8,
}

impl S {
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    fn method(self) {}
}
