script;

use contract_with_type_aliases_abi::*;

fn main() {
    let caller = abi(MyContract, 0xede5db326713355baf234583f3341f363efbfb0565ff8f6593dffb3a387a34f2);

    let x = AssetId::from(0x0101010101010101010101010101010101010101010101010101010101010101);

    let y = [
        contract_with_type_aliases_abi::IdentityAlias::ContractId(x),
        contract_with_type_aliases_abi::IdentityAlias::Address(Address::from(0x0202020202020202020202020202020202020202020202020202020202020202)),
    ];

    let z = contract_with_type_aliases_abi::IdentityAliasWrapperAlias { i: y[0] };

    let w = Generic { f: z };

    let u = (x, x);

    let s = "fuelfuel0";

    let (x_result, y_result, z_result, w_result, u_result, s_result) = caller.foo(x, y, z, w, u, s);

    assert(x == x_result);
    assert(y == y_result);
    assert(z == z_result);
    assert(w.f == w_result.f);
    assert(u == u_result);
    assert(s == s_result);
}
