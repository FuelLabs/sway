use crate::{
    ast_elements::type_argument::GenericTypeArgument,
    decl_engine::MaterializeConstGenerics,
    engine_threading::*,
    language::{parsed::EnumDeclaration, ty::TyDeclParsedType, CallPath, Visibility},
    transform,
    type_system::*,
};
use ast_elements::type_parameter::ConstGenericExpr;
use monomorphization::MonomorphizeHelper;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyEnumDecl {
    pub call_path: CallPath,
    pub generic_parameters: Vec<TypeParameter>,
    pub attributes: transform::Attributes,
    pub variants: Vec<TyEnumVariant>,
    pub span: Span,
    pub visibility: Visibility,
}

impl TyDeclParsedType for TyEnumDecl {
    type ParsedType = EnumDeclaration;
}

impl Named for TyEnumDecl {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl EqWithEngines for TyEnumDecl {}
impl PartialEqWithEngines for TyEnumDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.call_path == other.call_path
            && self.generic_parameters.eq(&other.generic_parameters, ctx)
            && self.variants.eq(&other.variants, ctx)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyEnumDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyEnumDecl {
            call_path,
            generic_parameters: type_parameters,
            variants,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        call_path.hash(state);
        variants.hash(state, engines);
        type_parameters.hash(state, engines);
        visibility.hash(state);
    }
}

impl SubstTypes for TyEnumDecl {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        todo!()
        // has_changes! {
        //     self.variants.subst(ctx);
        //     self.generic_parameters.subst(ctx);
        // }
    }
}

impl Spanned for TyEnumDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl IsConcrete for TyEnumDecl {
    fn is_concrete(&self, engines: &Engines) -> bool {
        self.generic_parameters
            .iter()
            .all(|tp| tp.is_concrete(engines))
    }
}

impl MonomorphizeHelper for TyEnumDecl {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.generic_parameters
    }

    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }

    fn has_self_type_param(&self) -> bool {
        false
    }
}

impl MaterializeConstGenerics for TyEnumDecl {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &crate::language::ty::TyExpression,
    ) -> Result<(), ErrorEmitted> {
        for p in self.generic_parameters.iter_mut() {
            match p {
                TypeParameter::Const(p) if p.name.as_str() == name => {
                    p.expr = Some(ConstGenericExpr::from_ty_expression(handler, value)?);
                }
                TypeParameter::Type(p) => {
                    p.type_id
                        .materialize_const_generics(engines, handler, name, value)?;
                }
                _ => {}
            }
        }

        for variant in self.variants.iter_mut() {
            variant
                .type_argument
                .type_id
                .materialize_const_generics(engines, handler, name, value)?;
        }

        Ok(())
    }
}

impl TyEnumDecl {
    pub(crate) fn expect_variant_from_name(
        &self,
        handler: &Handler,
        variant_name: &Ident,
    ) -> Result<&TyEnumVariant, ErrorEmitted> {
        match self
            .variants
            .iter()
            .find(|x| x.name.as_str() == variant_name.as_str())
        {
            Some(variant) => Ok(variant),
            None => Err(handler.emit_err(CompileError::UnknownEnumVariant {
                enum_name: self.call_path.suffix.clone(),
                variant_name: variant_name.clone(),
                span: variant_name.span(),
            })),
        }
    }
}

impl Spanned for TyEnumVariant {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyEnumVariant {
    pub name: Ident,
    pub type_argument: GenericTypeArgument,
    pub(crate) tag: usize,
    pub span: Span,
    pub attributes: transform::Attributes,
}

impl HashWithEngines for TyEnumVariant {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        self.name.hash(state);
        self.type_argument.hash(state, engines);
        self.tag.hash(state);
    }
}

impl EqWithEngines for TyEnumVariant {}
impl PartialEqWithEngines for TyEnumVariant {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.type_argument.eq(&other.type_argument, ctx)
            && self.tag == other.tag
    }
}

impl OrdWithEngines for TyEnumVariant {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let TyEnumVariant {
            name: ln,
            type_argument: lta,
            tag: lt,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        let TyEnumVariant {
            name: rn,
            type_argument: rta,
            tag: rt,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = other;
        ln.cmp(rn)
            .then_with(|| lta.cmp(rta, ctx))
            .then_with(|| lt.cmp(rt))
    }
}

// impl SubstTypes for TyEnumVariant {
//     fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
//         self.type_argument.subst_inner(ctx)
//     }
// }
