library;

use std::hash::*;

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

fn sha256_str9(value: str[9]) -> b256 {
    let mut hasher = Hasher::new();
    hasher.write_str(value);
    hasher.sha256()
}

pub type StringTy = str[9];
impl core::ops::Eq for StringTy {
    fn eq(self, other: Self) -> bool {
        sha256_str9(self) == sha256_str9(other)
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
