library;

use std::hash::Hash;
use ::lib::{A, FirstTrait, SecondTrait, GenericTrait};

impl FirstTrait for A {}

impl GenericTrait<u8> for A {}

impl<T> SecondTrait<T> for A {
    fn trait_method(self, t: T) where T: FirstTrait { }
    fn trait_associated_function(t: T) where T: FirstTrait { }
}

use ::lib::FirstTrait as FirstTraitAlias;

pub fn function_with_trait_alias<T>(t: T) where T: FirstTraitAlias { }

use ::other_lib::DuplicatedTrait;

impl DuplicatedTrait for A {}

pub fn function_with_duplicated_trait<T>(t: T) where T: DuplicatedTrait { }