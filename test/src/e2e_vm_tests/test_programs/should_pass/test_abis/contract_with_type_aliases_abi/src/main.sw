library;

use std::hash::*;

pub type IdentityAlias = Identity;

pub struct IdentityAliasWrapper {
    pub i: IdentityAlias,
}

pub type Array = [IdentityAlias; 2];
impl core::ops::Eq for Array {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}

pub type Tuple = (SubId, SubId);
impl core::ops::Eq for Tuple {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

pub type StringTy = str[9];
impl core::ops::Eq for StringTy {
    fn eq(self, other: Self) -> bool {
        sha256_str_array(self) == sha256_str_array(other)
    }
}

pub type IdentityAliasWrapperAlias = IdentityAliasWrapper;
impl core::ops::Eq for IdentityAliasWrapperAlias {
    fn eq(self, other: Self) -> bool {
        self.i == other.i
    }
}

pub struct Generic<T> {
    pub f: T,
}

abi MyContract {
    fn foo(x: SubId, y: [IdentityAlias; 2], z: IdentityAliasWrapperAlias, w: Generic<IdentityAliasWrapperAlias>, u: (SubId, SubId), s: StringTy) -> (SubId, [IdentityAlias; 2], IdentityAliasWrapperAlias, Generic<IdentityAliasWrapperAlias>, (SubId, SubId), StringTy);
}
