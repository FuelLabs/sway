contract;

#[deprecated(note = "This struct is deprecated and should not be used.")]
struct DeprecatedStruct { }

impl Contract {
    fn impl_method() -> u64 { 42 }

    // This proves that https://github.com/FuelLabs/sway/issues/7574 is fixed.
    #[cfg(experimental_new_encoding = true)]
    fn some_contract_method() -> u64 { 4422 }

    #[cfg(experimental_new_encoding = false)]
    fn some_contract_method() -> u64 { 2244 }

    const AUTO_IMPL_CONST: u64 = 1234;

    fn get_auto_impl_const() -> u64 {
        Self::AUTO_IMPL_CONST
    }

    #[storage(read)]
    fn storage_attribute_preserved() -> u64 {
        __state_load_word(b256::zero())
    }

    #[inline(always)]
    #[allow(deprecated)]
    fn deprecated_attribute_preserved_in_abi_as_well() -> DeprecatedStruct {
        DeprecatedStruct { }
    }
}

#[test]
fn test_impl_method() {
    let caller = abi(ContractAbiAutoImplAbi, CONTRACT_ID);
    assert(caller.impl_method() == 42)
}

#[test]
#[cfg(experimental_new_encoding = true)]
fn test_some_contract_method_new_encoding() {
    let caller = abi(ContractAbiAutoImplAbi, CONTRACT_ID);
    assert(caller.some_contract_method() == 4422)
}

#[test]
#[cfg(experimental_new_encoding = false)]
fn test_some_contract_method_old_encoding() {
    let caller = abi(ContractAbiAutoImplAbi, CONTRACT_ID);
    assert(caller.some_contract_method() == 2244)
}

#[test]
fn test_get_auto_impl_const() {
    let caller = abi(ContractAbiAutoImplAbi, CONTRACT_ID);
    assert(caller.get_auto_impl_const() == 1234)
}

#[test]
fn test_storage_attribute_preserved() {
    let caller = abi(ContractAbiAutoImplAbi, CONTRACT_ID);
    assert(caller.storage_attribute_preserved() == 0);
}

#[test]
fn test_deprecated_attribute_preserved_in_abi_as_well() {
    let caller = abi(ContractAbiAutoImplAbi, CONTRACT_ID);
    let _ = caller.deprecated_attribute_preserved_in_abi_as_well();
}
