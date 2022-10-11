use crate::{
    error::*,
    language::{parsed::*, Visibility},
    semantic_analysis::*,
    type_system::{
        insert_type, look_up_type_id, CopyTypes, CreateTypeId, EnforceTypeArguments,
        MonomorphizeHelper, ReplaceSelfType, TypeId, TypeMapping, TypeParameter,
    },
    AttributesMap, TypeInfo,
};
use std::hash::{Hash, Hasher};
use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

#[derive(Clone, Debug, Eq)]
pub struct TyEnumDeclaration {
    pub name: Ident,
    pub type_parameters: Vec<TypeParameter>,
    pub attributes: AttributesMap,
    pub variants: Vec<TyEnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyEnumDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.type_parameters == other.type_parameters
            && self.variants == other.variants
            && self.visibility == other.visibility
    }
}

impl CopyTypes for TyEnumDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl CreateTypeId for TyEnumDeclaration {
    fn create_type_id(&self) -> TypeId {
        insert_type(TypeInfo::Enum {
            name: self.name.clone(),
            variant_types: self.variants.clone(),
            type_parameters: self.type_parameters.clone(),
        })
    }
}

impl Spanned for TyEnumDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyEnumDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl TyEnumDeclaration {
    pub fn type_check(ctx: TypeCheckContext, decl: EnumDeclaration) -> CompileResult<Self> {
        let mut errors = vec![];
        let mut warnings = vec![];

        let EnumDeclaration {
            name,
            type_parameters,
            variants,
            span,
            attributes,
            visibility,
            ..
        } = decl;

        // create a namespace for the decl, used to create a scope for generics
        let mut decl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut decl_namespace);

        // type check the type parameters
        // insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            new_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // type check the variants
        let mut variants_buf = vec![];
        for variant in variants {
            variants_buf.push(check!(
                TyEnumVariant::type_check(ctx.by_ref(), variant.clone()),
                continue,
                warnings,
                errors
            ));
        }

        // create the enum decl
        let decl = TyEnumDeclaration {
            name,
            type_parameters: new_type_parameters,
            variants: variants_buf,
            span,
            attributes,
            visibility,
        };
        ok(decl, warnings, errors)
    }

    pub(crate) fn expect_variant_from_name(
        &self,
        variant_name: &Ident,
    ) -> CompileResult<&TyEnumVariant> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .variants
            .iter()
            .find(|x| x.name.as_str() == variant_name.as_str())
        {
            Some(variant) => ok(variant, warnings, errors),
            None => {
                errors.push(CompileError::UnknownEnumVariant {
                    enum_name: self.name.clone(),
                    variant_name: variant_name.clone(),
                    span: self.span.clone(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct TyEnumVariant {
    pub name: Ident,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub type_span: Span,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TyEnumVariant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        look_up_type_id(self.type_id).hash(state);
        self.tag.hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyEnumVariant {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.tag == other.tag
    }
}

impl CopyTypes for TyEnumVariant {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id.copy_types(type_mapping);
    }
}

impl ReplaceSelfType for TyEnumVariant {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl TyEnumVariant {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        variant: EnumVariant,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let initial_type_id = insert_type(variant.type_info);
        let enum_variant_type = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &variant.span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        ok(
            TyEnumVariant {
                name: variant.name.clone(),
                type_id: enum_variant_type,
                initial_type_id,
                type_span: variant.type_span.clone(),
                tag: variant.tag,
                span: variant.span,
            },
            vec![],
            errors,
        )
    }
}
