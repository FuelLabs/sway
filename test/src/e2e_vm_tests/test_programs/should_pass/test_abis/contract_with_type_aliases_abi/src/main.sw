library;

pub type IdentityAlias = Identity;

pub struct IdentityAliasWrapper {
    i: IdentityAlias,
}

pub type IdentityAliasWrapperAlias = IdentityAliasWrapper;

impl core::ops::Eq for IdentityAliasWrapperAlias {
    fn eq(self, other: Self) -> bool {
        self.i == other.i
    }
}

pub struct Generic<T> {
    f: T,
}

abi MyContract {
    fn foo(x: AssetId, y: [IdentityAlias; 2], z: IdentityAliasWrapperAlias, w: Generic<IdentityAliasWrapperAlias>, u: (AssetId, AssetId)) -> (AssetId, [IdentityAlias; 2], IdentityAliasWrapperAlias, Generic<IdentityAliasWrapperAlias>, (AssetId, AssetId));
}
