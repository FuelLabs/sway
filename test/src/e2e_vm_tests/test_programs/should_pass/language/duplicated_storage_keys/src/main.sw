contract;

storage {
    f1:u64 = 1,
    #[cfg(experimental_storage_domains = true)]
    f2 in 0xcecf0a910789de762c699a85a66835df1662df633238cbb25804b7f78640747b: u64 = 2,
    #[cfg(experimental_storage_domains = false)]
    f2 in 0x36389d1013642dcb070193fc48b0316e9dfdfef1860096dc5957e3eb44430b83: u64 = 2,
    ns1 {
        f3 in 0x5f4c20ce4bd128e5393a4c2b82007dac795fa0006d01acf8db4c42632bc680ca: u64 = 2,
    },
    ns2 {
        f4 in 0x5f4c20ce4bd128e5393a4c2b82007dac795fa0006d01acf8db4c42632bc680ca: u64 = 2,
    },
    ns3 {
        #[cfg(experimental_storage_domains = true)]
        f5 in 0x41e70e0fdfa49becc40cbfd5c057ab0540e8844f3d737fa3b1ab21a564b48069: u64 = 3,
        #[cfg(experimental_storage_domains = false)]
        f5 in 0xa49ebab6739a90f7658bbbdc2ed139942bd0b7be2e89aa8d90a953c45bf7a211: u64 = 3,
    },
    ns4 {
        f6: u64 = 4,
    },
}

