use crate::{decl_engine::DeclId, language::ty::TyAbiDecl};

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub enum AbiMode {
    ImplAbiFn(sway_types::Ident, Option<DeclId<TyAbiDecl>>),
    #[default]
    NonAbi,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ConstShadowingMode {
    Allow,
    Sequential,
    #[default]
    ItemStyle,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum GenericShadowingMode {
    Disallow,
    #[default]
    Allow,
}
