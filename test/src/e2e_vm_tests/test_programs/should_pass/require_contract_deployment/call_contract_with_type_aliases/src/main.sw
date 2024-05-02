script;

use contract_with_type_aliases_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x9d76ecbf446c30ef659efd1157d67d156de02b1e6c2ac2f9c744125571efa229;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xbd2ab800338002ff56708e5ca457e678a714b25b93031bcd0e7a352212c0affc;

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
