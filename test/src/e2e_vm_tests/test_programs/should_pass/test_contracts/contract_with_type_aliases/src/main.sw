contract;

use contract_with_type_aliases_abi::*;

impl MyContract for Contract {
    fn foo(
        x: AssetId,
        y: [IdentityAlias; 2],
        z: IdentityAliasWrapperAlias,
        w: Generic<IdentityAliasWrapperAlias>,
        u: (AssetId, AssetId),
        s: StringTy,
    ) -> (AssetId, [IdentityAlias; 2], IdentityAliasWrapperAlias, Generic<IdentityAliasWrapperAlias>, (AssetId, AssetId), StringTy) {
        (x, y, z, w, u, s)
    }
}
