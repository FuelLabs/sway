library;

pub type IdentityAlias = Identity;

pub struct IdentityAliasWrapper {
    i: IdentityAlias,
}

pub type Array = [IdentityAlias; 2];
impl core::ops::Eq for Array {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}

pub type Tuple = (AssetId, AssetId);
impl core::ops::Eq for Tuple {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

pub type StringTy = str[9];
impl core::ops::Eq for StringTy {
    fn eq(self, other: Self) -> bool {
        std::hash::sha256(self) == std::hash::sha256(other)
    }
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
    fn foo(x: AssetId, y: [IdentityAlias; 2], z: IdentityAliasWrapperAlias, w: Generic<IdentityAliasWrapperAlias>, u: (AssetId, AssetId), s: StringTy) -> (AssetId, [IdentityAlias; 2], IdentityAliasWrapperAlias, Generic<IdentityAliasWrapperAlias>, (AssetId, AssetId), StringTy);
}
