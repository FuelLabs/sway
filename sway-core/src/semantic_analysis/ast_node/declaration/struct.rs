use crate::{
    declaration_engine::declaration_engine::DeclarationEngine, error::*, parse_tree::*,
    semantic_analysis::*, type_system::*, types::*,
};
use std::hash::{Hash, Hasher};
use sway_types::{Ident, Property, Span, Spanned};

#[derive(Clone, Debug)]
pub struct TypedStructDeclaration {
    pub name: Ident,
    pub fields: Vec<TypedStructField>,
    pub type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub(crate) span: Span,
}

impl PartialEq for CompileWrapper<'_, TypedStructDeclaration> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name
            && me.fields.iter().map(|x| x.wrap_ref(de)).collect::<Vec<_>>()
                == them
                    .fields
                    .iter()
                    .map(|x| x.wrap_ref(de))
                    .collect::<Vec<_>>()
            && me
                .type_parameters
                .iter()
                .map(|x| x.wrap_ref(de))
                .collect::<Vec<_>>()
                == them
                    .type_parameters
                    .iter()
                    .map(|x| x.wrap_ref(de))
                    .collect::<Vec<_>>()
            && me.visibility == them.visibility
    }
}

impl CopyTypes for TypedStructDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, de));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, de));
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

impl TypedStructDeclaration {
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
                TypedStructField::type_check(ctx.by_ref(), field),
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

#[derive(Debug, Clone)]
pub struct TypedStructField {
    pub name: Ident,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub(crate) span: Span,
    pub type_span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for CompileWrapper<'_, TypedStructField> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name
            && look_up_type_id(me.type_id).wrap_ref(de)
                == look_up_type_id(them.type_id).wrap_ref(de)
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for CompileWrapper<'_, TypedStructField> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        me.name.hash(state);
        look_up_type_id(me.type_id).wrap_ref(de).hash(state);
    }
}

impl CopyTypes for TypedStructField {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
        self.type_id.update_type(type_mapping, de, &self.span);
    }
}

impl ToJsonAbi for TypedStructField {
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

impl ReplaceSelfType for TypedStructField {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl TypedStructField {
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
        let field = TypedStructField {
            name: field.name,
            type_id: r#type,
            initial_type_id,
            span: field.span,
            type_span: field.type_span,
        };
        ok(field, warnings, errors)
    }
}
