script;

use contract_with_type_aliases_abi::*;

type Array = [contract_with_type_aliases_abi::IdentityAlias; 2];
type Tuple = (AssetId, AssetId);

impl core::ops::Eq for Array {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}

impl core::ops::Eq for (AssetId, AssetId) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

fn main() {
    let caller = abi(MyContract, 0x8c3e0b0bf4c3d29494d9f58431fd849638367873da301ae4f290d023a8c79bea);

    let x = AssetId::from(0x0101010101010101010101010101010101010101010101010101010101010101);

    let y = [
        contract_with_type_aliases_abi::IdentityAlias::ContractId(x),
        contract_with_type_aliases_abi::IdentityAlias::Address(Address::from(0x0202020202020202020202020202020202020202020202020202020202020202)),
    ];

    let z = contract_with_type_aliases_abi::IdentityAliasWrapperAlias { i: y[0] };

    let w = Generic { f: z };

    let u = (x, x);

    let (x_result, y_result, z_result, w_result) = caller.foo(x, y, z, w, u);

    assert(x == x_result);
    assert(y == y_result);
    assert(z == z_result);
    assert(w.f == w_result.f);
    assert(u == u);
}
