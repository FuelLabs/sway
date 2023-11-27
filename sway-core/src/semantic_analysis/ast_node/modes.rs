use crate::{decl_engine::DeclId, language::ty::TyAbiDecl};

#[derive(Clone, PartialEq, Eq, Default)]
pub enum AbiMode {
    ImplAbiFn(sway_types::Ident, Option<DeclId<TyAbiDecl>>),
    #[default]
    NonAbi,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ConstShadowingMode {
    Sequential,
    #[default]
    ItemStyle,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum GenericShadowingMode {
    Disallow,
    #[default]
    Allow,
}
