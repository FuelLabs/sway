contract;

type IdentityAlias = Identity;

struct IdentityAliasWrapper {
    i: IdentityAlias,
}

type IdentityAliasWrapperAlias = IdentityAliasWrapper;

struct Generic<T> {
    f: T,
}

abi MyContract {
    fn foo(x: AssetId, y: [IdentityAlias; 2], z: IdentityAliasWrapperAlias, w: Generic<IdentityAliasWrapperAlias>) -> AssetId;
}

impl MyContract for Contract {
    fn foo(
        x: AssetId,
        y: [IdentityAlias; 2],
        z: IdentityAliasWrapperAlias,
        w: Generic<IdentityAliasWrapperAlias>,
    ) -> AssetId {
        x
    }
}
