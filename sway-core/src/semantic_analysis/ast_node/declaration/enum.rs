use crate::{
    declaration_engine::declaration_engine::DeclarationEngine,
    error::*,
    parse_tree::*,
    semantic_analysis::*,
    type_system::{
        insert_type, look_up_type_id, CopyTypes, CreateTypeId, EnforceTypeArguments,
        MonomorphizeHelper, ReplaceSelfType, TypeId, TypeMapping, TypeParameter,
    },
    types::{CompileWrapper, JsonAbiString, ToCompileWrapper, ToJsonAbi},
    TypeInfo,
};
use std::hash::{Hash, Hasher};
use sway_types::{Ident, Property, Span, Spanned};

#[derive(Clone, Debug)]
pub struct TypedEnumDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub variants: Vec<TypedEnumVariant>,
    pub(crate) span: Span,
    pub visibility: Visibility,
}

impl PartialEq for CompileWrapper<'_, TypedEnumDeclaration> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name
            && me.type_parameters.wrap(de) == them.type_parameters.wrap(de)
            && me.variants.wrap(de) == them.variants.wrap(de)
            && me.visibility == them.visibility
    }
}

impl CopyTypes for TypedEnumDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, de));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, de));
    }
}

impl CreateTypeId for TypedEnumDeclaration {
    fn create_type_id(&self) -> TypeId {
        insert_type(TypeInfo::Enum {
            name: self.name.clone(),
            variant_types: self.variants.clone(),
            type_parameters: self.type_parameters.clone(),
        })
    }
}

impl Spanned for TypedEnumDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TypedEnumDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl TypedEnumDeclaration {
    pub fn type_check(ctx: TypeCheckContext, decl: EnumDeclaration) -> CompileResult<Self> {
        let mut errors = vec![];
        let mut warnings = vec![];

        let EnumDeclaration {
            name,
            type_parameters,
            variants,
            span,
            visibility,
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
                TypedEnumVariant::type_check(ctx.by_ref(), variant.clone()),
                continue,
                warnings,
                errors
            ));
        }

        // create the enum decl
        let decl = TypedEnumDeclaration {
            name,
            type_parameters: new_type_parameters,
            variants: variants_buf,
            span,
            visibility,
        };
        ok(decl, warnings, errors)
    }

    pub(crate) fn expect_variant_from_name(
        &self,
        variant_name: &Ident,
    ) -> CompileResult<&TypedEnumVariant> {
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

#[derive(Debug, Clone)]
pub struct TypedEnumVariant {
    pub name: Ident,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

impl PartialEq for CompileWrapper<'_, TypedEnumVariant> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name
            && look_up_type_id(me.type_id).wrap(declaration_engine)
                == look_up_type_id(them.type_id).wrap(declaration_engine)
            && me.tag == them.tag
    }
}

impl PartialEq for CompileWrapper<'_, Vec<TypedEnumVariant>> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        if me.len() != them.len() {
            return false;
        }
        me.iter()
            .map(|elem| elem.wrap(de))
            .zip(other.inner.iter().map(|elem| elem.wrap(de)))
            .map(|(left, right)| left == right)
            .all(|elem| elem)
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for CompileWrapper<'_, TypedEnumVariant> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        me.name.hash(state);
        look_up_type_id(me.type_id).wrap(de).hash(state);
        me.tag.hash(state);
    }
}

impl CopyTypes for TypedEnumVariant {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
        self.type_id.update_type(type_mapping, de, &self.span);
    }
}

impl ToJsonAbi for TypedEnumVariant {
    type Output = Property;

    fn generate_json_abi(&self) -> Self::Output {
        Property {
            name: self.name.to_string(),
            type_field: self.type_id.json_abi_str(),
            components: self.type_id.generate_json_abi(),
            type_arguments: self
                .type_id
                .get_type_parameters()
                .map(|v| v.iter().map(TypeParameter::generate_json_abi).collect()),
        }
    }
}

impl ReplaceSelfType for TypedEnumVariant {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl TypedEnumVariant {
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
            TypedEnumVariant {
                name: variant.name.clone(),
                type_id: enum_variant_type,
                initial_type_id,
                tag: variant.tag,
                span: variant.span,
            },
            vec![],
            errors,
        )
    }
}
