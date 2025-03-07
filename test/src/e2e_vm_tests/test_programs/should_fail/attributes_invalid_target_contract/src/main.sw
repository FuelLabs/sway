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
impl Abi for Contract {
    #[test(invalid)]
    //! Invalid inner comment.
    #[fallback(invalid)]
    fn abi_function() {
        let _ = 0;
    }
}