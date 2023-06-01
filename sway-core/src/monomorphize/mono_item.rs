use crate::{decl_engine::DeclId, language::ty::*, type_system::*};


pub(crate) enum MonoItem {
    MonoFn(DeclId<TyFunctionDecl>, SubstList),
    MonoStruct(DeclId<TyStructDecl>, SubstList),
    MonoEnum(DeclId<TyEnumDecl>, SubstList),
    MonoTrait(DeclId<TyTraitFn>, SubstList),
    MonoImplTrait(DeclId<TyImplTrait>, SubstList),
    MonoStorage(DeclId<TyStorageDecl>),
    MonoAbi(DeclId<TyAbiDecl>),
    MonoConstant(DeclId<TyConstantDecl>),
    MonoTypeAlias(DeclId<TyTypeAliasDecl>),
}
