contract;

#[storage(read, write)]
#[doc(test)]
abi GoodAbi {
    #[storage(read, write)]
    #[doc(test)]
    fn good_func() -> bool;

    #[bad_attr(blah)]
    fn bad_func() -> bool;
}

impl GoodAbi for Contract {
    #[storage(read, write)]
    #[doc(Test)]
    fn good_func() -> bool {
        true
    }

    #[bad_attr(blah)]
    fn bad_func() -> bool {
        true
    }
}

#[bad_attr(blah)]
struct BadStruct {}
