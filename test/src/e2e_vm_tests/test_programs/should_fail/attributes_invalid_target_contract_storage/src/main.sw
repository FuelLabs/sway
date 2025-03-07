contract;

#[storage(invalid)]
#[inline(invalid)]
#[test(invalid)]
//! Invalid inner comment.
#[payable(invalid)]
#[deprecated(invalid)]
#[fallback(invalid)]
storage {
    #[storage(invalid)]
    #[inline(invalid)]
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    x: u8 = 0,
    // The below examples would require a separate test.
    // Let's skip it for now, all storage items are
    // treated the same.
    #[storage(invalid)]
    #[inline(invalid)]
    #[test(invalid)]
    #[payable(invalid)]
    //! Invalid inner comment.
    #[deprecated(invalid)]
    #[fallback(invalid)]
    ns_1 {
        #[storage(invalid)]
        #[inline(invalid)]
        #[test(invalid)]
        //! Invalid inner comment.
        #[payable(invalid)]
        #[deprecated(invalid)]
        #[fallback(invalid)]
        x: u8 = 0,
        #[storage(invalid)]
        #[inline(invalid)]
        #[test(invalid)]
        //! Invalid inner comment.
        #[payable(invalid)]
        #[deprecated(invalid)]
        #[fallback(invalid)]
        ns_2 {
            #[storage(invalid)]
            #[inline(invalid)]
            #[test(invalid)]
            //! Invalid inner comment.
            #[payable(invalid)]
            #[deprecated(invalid)]
            #[fallback(invalid)]
            x: u8 = 0,
        }
    }
}
