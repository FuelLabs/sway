contract;

abi Abi {
    fn abi_function();
}

#[storage(invalid)]
#[inline(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
#[error_type(invalid)]
#[error(invalid)]
impl Abi for Contract {
    #[test(invalid)]
    //! Invalid inner comment.
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    fn abi_function() {
        let _ = 0;
    }
}