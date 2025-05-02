script;

use contract_with_type_aliases_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID: b256 = 0x0cbeb6efe3104b460be769bdc4ea101ebf16ccc16f2d7b667ec3e1c7f5ce35b5;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID: b256 = 0xb0899ebf2030d48330436b8025b9ca15243ac985f54024ea00d64d961b67482a; // AUTO-CONTRACT-ID ../../test_contracts/contract_with_type_aliases --release

fn main() {
    let caller = abi(MyContract, CONTRACT_ID);

    let x: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101;

    let y = [
        contract_with_type_aliases_abi::IdentityAlias::ContractId(ContractId::from(x)),
        contract_with_type_aliases_abi::IdentityAlias::Address(Address::from(0x0202020202020202020202020202020202020202020202020202020202020202)),
    ];

    let z = contract_with_type_aliases_abi::IdentityAliasWrapperAlias { i: y[0] };

    let w = Generic { f: z };

    let u = (x, x);

    let s = __to_str_array("fuelfuel0");

    let (x_result, y_result, z_result, w_result, u_result, s_result) = caller.foo(x, y, z, w, u, s);

    assert(x == x_result);
    assert(y == y_result);
    assert(z == z_result);
    assert(w.f == w_result.f);
    assert(u == u_result);
    assert(s == s_result);
}
