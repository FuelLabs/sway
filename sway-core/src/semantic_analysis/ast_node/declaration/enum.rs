use crate::{
    error::*,
    namespace::*,
    parse_tree::*,
    semantic_analysis::*,
    type_engine::{
        insert_type, look_up_type_id, unify, CopyTypes, CreateTypeId, ReplaceSelfType,
        ResolveTypes, TypeId, TypeMapping,
    },
    types::{JsonAbiString, ToJsonAbi},
    TypeInfo,
};
use fuels_types::Property;
use std::hash::{Hash, Hasher};
use sway_types::{Ident, Span, Spanned};

#[derive(Clone, Debug, Eq)]
pub struct TypedEnumDeclaration {
    pub name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub variants: Vec<TypedEnumVariant>,
    pub(crate) span: Span,
    pub(crate) visibility: Visibility,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedEnumDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.type_parameters == other.type_parameters
            && self.variants == other.variants
            && self.visibility == other.visibility
    }
}

impl CopyTypes for TypedEnumDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
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
    type Output = TypedEnumDeclaration;

    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }

    fn monomorphize_inner(self, type_mapping: &TypeMapping, namespace: &mut Items) -> Self::Output {
        monomorphize_inner(self, type_mapping, namespace)
    }
}

impl ResolveTypes for TypedEnumDeclaration {
    fn resolve_types(
        &mut self,
        type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // create a new namespace for type resolution
        let mut namespace = namespace.clone();

        // insert the type parameters into the namespace
        let module = check!(
            namespace.check_submodule_mut(module_path),
            return err(warnings, errors),
            warnings,
            errors
        );
        for type_parameter in self.type_parameters.iter_mut() {
            type_parameter.type_id = insert_type(TypeInfo::UnknownGeneric {
                name: type_parameter.name_ident.clone(),
            });
            module.insert_symbol(type_parameter.name_ident.clone(), type_parameter.into());
        }

        // resolve the types of the variants
        for variant in self.variants.iter_mut() {
            check!(
                variant.resolve_types(vec!(), enforce_type_arguments, &mut namespace, module_path),
                continue,
                warnings,
                errors
            );
        }

        // unify the type parameters and the type arguments
        for (type_parameter, type_argument) in
            self.type_parameters.iter().zip(type_arguments.iter())
        {
            let (mut new_warnings, new_errors) = unify(
                type_parameter.type_id,
                type_argument.type_id,
                &type_argument.span,
                "Type argument is not assignable to generic type parameter.",
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }

        ok((), warnings, errors)
    }
}

impl TypedEnumDeclaration {
    pub fn type_check(
        decl: EnumDeclaration,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypedEnumDeclaration> {
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
        let mut namespace = namespace.clone();

        // type check the type parameters
        // insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            new_type_parameters.push(check!(
                TypeParameter::type_check(type_parameter, &mut namespace),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // type check the variants
        let mut variants_buf = vec![];
        for variant in variants {
            variants_buf.push(check!(
                TypedEnumVariant::type_check(
                    variant.clone(),
                    &mut namespace,
                    self_type,
                    variant.span,
                ),
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

#[derive(Debug, Clone, Eq)]
pub struct TypedEnumVariant {
    pub name: Ident,
    pub type_id: TypeId,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypedEnumVariant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        look_up_type_id(self.type_id).hash(state);
        self.tag.hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedEnumVariant {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.tag == other.tag
    }
}

impl CopyTypes for TypedEnumVariant {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id.update_type(type_mapping, &self.span);
    }
}

impl ToJsonAbi for TypedEnumVariant {
    type Output = Property;

    fn generate_json_abi(&self) -> Self::Output {
        Property {
            name: self.name.to_string(),
            type_field: self.type_id.json_abi_str(),
            components: self.type_id.generate_json_abi(),
        }
    }
}

impl ReplaceSelfType for TypedEnumVariant {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl ResolveTypes for TypedEnumVariant {
    fn resolve_types(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        self.type_id = check!(
            namespace.resolve_type(
                self.type_id,
                &self.span,
                enforce_type_arguments,
                module_path,
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );
        ok((), warnings, errors)
    }
}

impl TypedEnumVariant {
    pub(crate) fn type_check(
        variant: EnumVariant,
        namespace: &mut Namespace,
        self_type: TypeId,
        span: Span,
    ) -> CompileResult<TypedEnumVariant> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let enum_variant_type = check!(
            namespace.resolve_type_with_self(
                insert_type(variant.type_info),
                self_type,
                &span,
                EnforceTypeArguments::Yes
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        ok(
            TypedEnumVariant {
                name: variant.name.clone(),
                type_id: enum_variant_type,
                tag: variant.tag,
                span: variant.span,
            },
            vec![],
            errors,
        )
    }
}
