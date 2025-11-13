use super::{TyDeclParsedType, TyTraitInterfaceItem, TyTraitItem};
use crate::{
    decl_engine::DeclEngineGet as _,
    engine_threading::*,
    language::parsed::{self, AbiDeclaration},
    transform,
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Named, Span, Spanned};

/// A [TyAbiDecl] contains the type-checked version of the parse tree's
/// [AbiDeclaration].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyAbiDecl {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<TyTraitInterfaceItem>,
    pub supertraits: Vec<parsed::Supertrait>,
    pub items: Vec<TyTraitItem>,
    pub span: Span,
    pub attributes: transform::Attributes,
}

impl TyAbiDecl {
    pub(crate) fn forbid_const_generics(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<(), ErrorEmitted> {
        for item in self.interface_surface.iter() {
            match item {
                TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    let decl = engines.de().get(decl_ref.id());

                    if decl.return_type.type_id.has_const_generics(engines) {
                        let err = handler.emit_err(CompileError::ConstGenericNotSupportedHere {
                            span: decl.return_type.span(),
                        });
                        return Err(err);
                    }

                    for arg in decl.parameters.iter() {
                        if arg.type_argument.type_id.has_const_generics(engines) {
                            let err =
                                handler.emit_err(CompileError::ConstGenericNotSupportedHere {
                                    span: arg.type_argument.span.clone(),
                                });
                            return Err(err);
                        }
                    }
                }
                TyTraitInterfaceItem::Constant(_) => {}
                TyTraitInterfaceItem::Type(_) => {}
            }
        }

        for item in self.items.iter() {
            match item {
                TyTraitItem::Fn(decl_ref) => {
                    let decl = engines.de().get(decl_ref.id());
                    if decl.return_type.type_id.has_const_generics(engines) {
                        let err = handler.emit_err(CompileError::ConstGenericNotSupportedHere {
                            span: decl.return_type.span(),
                        });
                        return Err(err);
                    }

                    for arg in decl.parameters.iter() {
                        if arg.type_argument.type_id.has_const_generics(engines) {
                            let err =
                                handler.emit_err(CompileError::ConstGenericNotSupportedHere {
                                    span: arg.type_argument.span.clone(),
                                });
                            return Err(err);
                        }
                    }
                }
                TyTraitItem::Constant(_) => {}
                TyTraitItem::Type(_) => {}
            }
        }

        Ok(())
    }
}

impl TyDeclParsedType for TyAbiDecl {
    type ParsedType = AbiDeclaration;
}

impl EqWithEngines for TyAbiDecl {}
impl PartialEqWithEngines for TyAbiDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let TyAbiDecl {
            name: ln,
            interface_surface: lis,
            supertraits: ls,
            items: li,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        let TyAbiDecl {
            name: rn,
            interface_surface: ris,
            supertraits: rs,
            items: ri,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = other;
        ln == rn && lis.eq(ris, ctx) && li.eq(ri, ctx) && ls.eq(rs, ctx)
    }
}

impl HashWithEngines for TyAbiDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyAbiDecl {
            name,
            interface_surface,
            items,
            supertraits,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        name.hash(state);
        interface_surface.hash(state, engines);
        items.hash(state, engines);
        supertraits.hash(state, engines);
    }
}

impl CreateTypeId for TyAbiDecl {
    fn create_type_id(&self, engines: &Engines) -> TypeId {
        engines
            .te()
            .new_contract_caller(engines, AbiName::Known(self.name.clone().into()), None)
    }
}

impl Spanned for TyAbiDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Named for TyAbiDecl {
    fn name(&self) -> &Ident {
        &self.name
    }
}
