contract;

type IdentityAlias = Identity;

struct IdentityAliasWrapper {
    i: IdentityAlias,
}

type Array = [IdentityAlias; 2];
type Tuple = (AssetId, AssetId);
type StringTy = str;
type IdentityAliasWrapperAlias = IdentityAliasWrapper;
struct Generic<T> {
    f: T,
}

abi MyContract {
    fn foo(x: AssetId, y: [IdentityAlias; 2], z: IdentityAliasWrapperAlias, w: Generic<IdentityAliasWrapperAlias>, u: (AssetId, AssetId)) -> (AssetId, [IdentityAlias; 2], IdentityAliasWrapperAlias, Generic<IdentityAliasWrapperAlias>, (AssetId, AssetId));
    fn bar(s: StringTy) -> StringTy;
}

impl MyContract for Contract {
    fn foo(
        x: AssetId,
        y: [IdentityAlias; 2],
        z: IdentityAliasWrapperAlias,
        w: Generic<IdentityAliasWrapperAlias>,
        u: (AssetId, AssetId),
    ) -> (AssetId, [IdentityAlias; 2], IdentityAliasWrapperAlias, Generic<IdentityAliasWrapperAlias>, (AssetId, AssetId)) {
        (x, y, z, w, u)
    }
    fn bar(s: StringTy) -> StringTy {
        s
    }
}
