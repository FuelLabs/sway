use crate::{
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
    AttributesMap,
};
use std::hash::{Hash, Hasher};
use sway_error::error::CompileError;
use sway_types::{Ident, Span};

impl ty::TyStructDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        decl: StructDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let StructDeclaration {
            name,
            fields,
            type_parameters,
            visibility,
            span,
            attributes,
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

        // type check the fields
        let mut new_fields = vec![];
        for field in fields.into_iter() {
            new_fields.push(check!(
                TyStructField::type_check(ctx.by_ref(), field),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // create the struct decl
        let decl = ty::TyStructDeclaration {
            name,
            type_parameters: new_type_parameters,
            fields: new_fields,
            visibility,
            span,
            attributes,
        };

        ok(decl, warnings, errors)
    }

    pub(crate) fn expect_field(&self, field_to_access: &Ident) -> CompileResult<&TyStructField> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .fields
            .iter()
            .find(|TyStructField { name, .. }| name.as_str() == field_to_access.as_str())
        {
            Some(field) => ok(field, warnings, errors),
            None => {
                errors.push(CompileError::FieldNotFound {
                    available_fields: self
                        .fields
                        .iter()
                        .map(|TyStructField { name, .. }| name.to_string())
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
pub struct TyStructField {
    pub name: Ident,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub(crate) span: Span,
    pub type_span: Span,
    pub attributes: AttributesMap,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TyStructField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        look_up_type_id(self.type_id).hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyStructField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
    }
}

impl CopyTypes for TyStructField {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id.copy_types(type_mapping);
    }
}

impl ReplaceSelfType for TyStructField {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl TyStructField {
    pub(crate) fn type_check(mut ctx: TypeCheckContext, field: StructField) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let initial_type_id = insert_type(field.type_info);
        let r#type = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &field.type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let field = TyStructField {
            name: field.name,
            type_id: r#type,
            initial_type_id,
            span: field.span,
            type_span: field.type_span,
            attributes: field.attributes,
        };
        ok(field, warnings, errors)
    }
}
