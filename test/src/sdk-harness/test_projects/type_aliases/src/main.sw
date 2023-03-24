contract;

type IdentityAlias = Identity;

struct IdentityAliasWrapper {
    i: IdentityAlias,
}

type Array = [IdentityAlias; 2];
type Tuple = (AssetId, AssetId);
type StringTy = str[9];
type IdentityAliasWrapperAlias = IdentityAliasWrapper;
struct Generic<T> {
    f: T,
}

abi MyContract {
    fn foo(x: AssetId, y: [IdentityAlias; 2], z: IdentityAliasWrapperAlias, w: Generic<IdentityAliasWrapperAlias>, u: (AssetId, AssetId), s: StringTy) -> (AssetId, [IdentityAlias; 2], IdentityAliasWrapperAlias, Generic<IdentityAliasWrapperAlias>, (AssetId, AssetId), StringTy);
}

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
