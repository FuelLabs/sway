library;

use std::hash::*;

pub type IdentityAlias = Identity;

pub struct IdentityAliasWrapper {
    pub i: IdentityAlias,
}

pub type Array = [IdentityAlias; 2];
#[cfg(experimental_partial_eq = false)]
impl core::ops::Eq for Array {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::PartialEq for Array {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::Eq for Array {}

pub type Tuple = (SubId, SubId);
#[cfg(experimental_partial_eq = false)]
impl core::ops::Eq for Tuple {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::PartialEq for Tuple {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::Eq for Tuple {}

pub type StringTy = str[9];
#[cfg(experimental_partial_eq = false)]
impl core::ops::Eq for StringTy {
    fn eq(self, other: Self) -> bool {
        sha256_str_array(self) == sha256_str_array(other)
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::PartialEq for StringTy {
    fn eq(self, other: Self) -> bool {
        sha256_str_array(self) == sha256_str_array(other)
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::Eq for StringTy {}

pub type IdentityAliasWrapperAlias = IdentityAliasWrapper;
#[cfg(experimental_partial_eq = false)]
impl core::ops::Eq for IdentityAliasWrapperAlias {
    fn eq(self, other: Self) -> bool {
        self.i == other.i
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::PartialEq for IdentityAliasWrapperAlias {
    fn eq(self, other: Self) -> bool {
        self.i == other.i
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::Eq for IdentityAliasWrapperAlias {}

pub struct Generic<T> {
    pub f: T,
}

abi MyContract {
    fn foo(
        x: SubId,
        y: [IdentityAlias; 2],
        z: IdentityAliasWrapperAlias,
        w: Generic<IdentityAliasWrapperAlias>,
        u: (SubId, SubId),
        s: StringTy,
    ) -> (SubId, [IdentityAlias; 2], IdentityAliasWrapperAlias, Generic<IdentityAliasWrapperAlias>, (SubId, SubId), StringTy);
}
