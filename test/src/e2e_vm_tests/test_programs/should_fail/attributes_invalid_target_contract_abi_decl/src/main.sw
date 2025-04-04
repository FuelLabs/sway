library;

#[storage(invalid)]
#[inline(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
#[error_type(invalid)]
#[error(invalid)]
abi Abi {
    #[test(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    #[error(invalid)]
    fn abi_function();
}