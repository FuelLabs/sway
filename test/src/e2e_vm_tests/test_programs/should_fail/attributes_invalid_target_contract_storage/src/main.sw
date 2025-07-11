contract;

#[storage(invalid)]
#[inline(invalid)]
#[trace(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
#[error_type(invalid)]
#[error(invalid)]
storage {
    #[storage(invalid)]
    #[inline(invalid)]
    #[trace(invalid)]
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    x: u8 = 0,
    // The below examples would require a separate test.
    // Let's skip it for now, all storage items are
    // treated the same.
    #[storage(invalid)]
    #[inline(invalid)]
    #[trace(invalid)]
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    #[error_type(invalid)]
    #[error(invalid)]
    ns_1 {
        #[storage(invalid)]
        #[inline(invalid)]
        #[trace(invalid)]
        #[test(invalid)]
        //! Invalid inner comment.
        #[payable(invalid)]
        #[deprecated(invalid)]
        #[fallback(invalid)]
        #[error(invalid)]
        x: u8 = 0,
        #[storage(invalid)]
        #[inline(invalid)]
        #[trace(invalid)]
        #[test(invalid)]
        //! Invalid inner comment.
        #[payable(invalid)]
        #[deprecated(invalid)]
        #[fallback(invalid)]
        #[error_type(invalid)]
        #[error(invalid)]
        ns_2 {
            #[storage(invalid)]
            #[inline(invalid)]
            #[trace(invalid)]
            #[test(invalid)]
            //! Invalid inner comment.
            #[payable(invalid)]
            #[deprecated(invalid)]
            #[fallback(invalid)]
            #[error_type(invalid)]
            #[error(invalid)]
            x: u8 = 0,
        }
    }
}
