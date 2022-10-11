use derivative::Derivative;
use sway_types::Ident;

use crate::{
    language::{parsed, Visibility},
    semantic_analysis::TyTraitFn,
    type_system::*,
};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyTraitDeclaration {
    pub name: Ident,
    pub interface_surface: Vec<TyTraitFn>,
    // NOTE: deriving partialeq and hash on this element may be important in the
    // future, but I am not sure. For now, adding this would 2x the amount of
    // work, so I am just going to exclude it
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) methods: Vec<parsed::FunctionDeclaration>,
    pub(crate) supertraits: Vec<parsed::Supertrait>,
    pub visibility: Visibility,
}

impl CopyTypes for TyTraitDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.interface_surface
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}
