use crate::{
    error::*, namespace::*, parse_tree::*, semantic_analysis::*, type_engine::*, types::*,
};
use fuels_types::Property;
use std::hash::{Hash, Hasher};
use sway_types::{Ident, Span, Spanned};

#[derive(Clone, Debug, Eq)]
pub struct TypedStructDeclaration {
    pub name: Ident,
    pub fields: Vec<TypedStructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) visibility: Visibility,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedStructDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.fields == other.fields
            && self.type_parameters == other.type_parameters
            && self.visibility == other.visibility
    }
}

impl CopyTypes for TypedStructDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl CreateTypeId for TypedStructDeclaration {
    fn create_type_id(&self) -> TypeId {
        insert_type(TypeInfo::Struct {
            name: self.name.clone(),
            fields: self.fields.clone(),
            type_parameters: self.type_parameters.clone(),
        })
    }
}

impl Spanned for TypedStructDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TypedStructDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl ResolveTypes for TypedStructDeclaration {
    fn resolve_type_with_self(
        &mut self,
        type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        self_type: TypeId,
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

        // resolve the types of the fields
        for field in self.fields.iter_mut() {
            check!(
                field.resolve_type_with_self(
                    vec!(),
                    enforce_type_arguments,
                    self_type,
                    &mut namespace,
                    module_path
                ),
                continue,
                warnings,
                errors
            );
        }

        // unify the type parameters and the type arguments
        for (type_parameter, type_argument) in
            self.type_parameters.iter().zip(type_arguments.iter())
        {
            let (mut new_warnings, new_errors) = unify_with_self(
                type_parameter.type_id,
                type_argument.type_id,
                self_type,
                &type_argument.span,
                "Type argument is not assignable to generic type parameter.",
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }

        ok((), warnings, errors)
    }

    fn resolve_type_without_self(
        &mut self,
        type_arguments: Vec<TypeArgument>,
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

        // resolve the types of the fields
        for field in self.fields.iter_mut() {
            check!(
                field.resolve_type_without_self(vec!(), &mut namespace, module_path),
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

impl TypedStructDeclaration {
    pub(crate) fn type_check(
        decl: StructDeclaration,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypedStructDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let StructDeclaration {
            name,
            fields,
            type_parameters,
            visibility,
            span,
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

        // type check the fields
        let mut new_fields = vec![];
        for field in fields.into_iter() {
            new_fields.push(check!(
                TypedStructField::type_check(field, &mut namespace, self_type),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // create the struct decl
        let decl = TypedStructDeclaration {
            name,
            type_parameters: new_type_parameters,
            fields: new_fields,
            visibility,
            span,
        };

        ok(decl, warnings, errors)
    }

    pub(crate) fn expect_field(&self, field_to_access: &Ident) -> CompileResult<&TypedStructField> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .fields
            .iter()
            .find(|TypedStructField { name, .. }| name.as_str() == field_to_access.as_str())
        {
            Some(field) => ok(field, warnings, errors),
            None => {
                errors.push(CompileError::FieldNotFound {
                    available_fields: self
                        .fields
                        .iter()
                        .map(|TypedStructField { name, .. }| name.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    field_name: field_to_access.clone(),
                    struct_name: self.name.clone(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct TypedStructField {
    pub name: Ident,
    pub type_id: TypeId,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypedStructField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        look_up_type_id(self.type_id).hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedStructField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
    }
}

impl CopyTypes for TypedStructField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id.update_type(type_mapping, &self.span);
    }
}

impl ToJsonAbi for TypedStructField {
    type Output = Property;

    fn generate_json_abi(&self) -> Self::Output {
        Property {
            name: self.name.to_string(),
            type_field: self.type_id.json_abi_str(),
            components: self.type_id.generate_json_abi(),
        }
    }
}

impl ReplaceSelfType for TypedStructField {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl ResolveTypes for TypedStructField {
    fn resolve_type_with_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        self_type: TypeId,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        self.type_id = check!(
            namespace.resolve_type_with_self(
                self.type_id,
                self_type,
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

    fn resolve_type_without_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        self.type_id = check!(
            namespace.resolve_type_without_self(self.type_id, module_path,),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );
        ok((), warnings, errors)
    }
}

impl TypedStructField {
    pub(crate) fn type_check(
        field: StructField,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypedStructField> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let r#type = check!(
            namespace.resolve_type_with_self(
                insert_type(field.type_info),
                self_type,
                &field.type_span,
                EnforceTypeArguments::Yes
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let field = TypedStructField {
            name: field.name,
            type_id: r#type,
            span: field.span,
        };
        ok(field, warnings, errors)
    }
}
