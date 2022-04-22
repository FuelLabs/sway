use crate::semantic_analysis::{
    TypeCheckedStorageAccess, TypeCheckedStorageAccessDescriptor, TypedStructField,
};
use crate::type_engine::look_up_type_id;
use crate::{
    error::*,
    type_engine::{TypeId, TypeInfo},
    Ident,
};
use sway_types::{state::StateIndex, Span};

use derivative::Derivative;

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedStorageDeclaration {
    pub(crate) fields: Vec<TypedStorageField>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    span: Span,
}

impl TypedStorageDeclaration {
    pub fn new(fields: Vec<TypedStorageField>, span: Span) -> Self {
        TypedStorageDeclaration { fields, span }
    }
    /// Given a field, find its type information in the declaration and return it. If the field has not
    /// been declared as a part of storage, return an error.
    pub fn apply_storage_load(
        &self,
        fields: Vec<Ident>,
        storage_fields: &[TypedStorageField],
    ) -> CompileResult<(TypeCheckedStorageAccess, TypeId)> {
        let mut errors = vec![];
        let warnings = vec![];

        let mut type_checked_buf = vec![];
        let mut fields: Vec<_> = fields.into_iter().rev().collect();

        let first_field = fields.pop().expect("guaranteed by grammar");
        let (ix, initial_field_type) = match storage_fields
            .iter()
            .enumerate()
            .find(|(_, TypedStorageField { name, .. })| name == &first_field)
        {
            Some((ix, TypedStorageField { r#type, .. })) => (StateIndex::new(ix), r#type),
            None => {
                errors.push(CompileError::StorageFieldDoesNotExist {
                    name: first_field.as_str().to_string(),
                    span: first_field.span().clone(),
                });
                return err(warnings, errors);
            }
        };

        type_checked_buf.push(TypeCheckedStorageAccessDescriptor {
            name: first_field.clone(),
            r#type: *initial_field_type,
            span: first_field.span().clone(),
        });

        fn update_available_struct_fields(id: TypeId) -> Vec<TypedStructField> {
            match crate::type_engine::look_up_type_id(id) {
                TypeInfo::Struct { fields, .. } => fields,
                _ => vec![],
            }
        }

        // if the previously iterated type was a struct, put its fields here so we know that,
        // in the case of a subfield, we can type check the that the subfield exists and its type.
        let mut available_struct_fields = update_available_struct_fields(*initial_field_type);

        // get the initial field's type
        // make sure the next field exists in that type
        for field in fields.into_iter().rev() {
            match available_struct_fields
                .iter()
                .find(|x| x.name.as_str() == field.as_str())
            {
                Some(struct_field) => {
                    type_checked_buf.push(TypeCheckedStorageAccessDescriptor {
                        name: field.clone(),
                        r#type: struct_field.r#type,
                        span: field.span().clone(),
                    });
                    available_struct_fields = update_available_struct_fields(struct_field.r#type);
                }
                None => {
                    let available_fields = available_struct_fields
                        .iter()
                        .map(|x| x.name.as_str())
                        .collect::<Vec<_>>();
                    errors.push(CompileError::FieldNotFound {
                        field_name: field.clone(),
                        available_fields: available_fields.join(", "),
                        struct_name: type_checked_buf.last().unwrap().name.as_str().to_string(),
                        span: field.span().clone(),
                    });
                    return err(warnings, errors);
                }
            }
        }

        let return_type = type_checked_buf[type_checked_buf.len() - 1].r#type;

        ok(
            (
                TypeCheckedStorageAccess {
                    fields: type_checked_buf,
                    ix,
                },
                return_type,
            ),
            warnings,
            errors,
        )
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub(crate) fn fields_as_typed_struct_fields(&self) -> Vec<TypedStructField> {
        self.fields
            .iter()
            .map(
                |TypedStorageField {
                     ref name,
                     ref r#type,
                     ref span,
                 }| TypedStructField {
                    name: name.clone(),
                    r#type: *r#type,
                    span: span.clone(),
                },
            )
            .collect()
    }
}

#[derive(Clone, Debug, Eq)]
pub struct TypedStorageField {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedStorageField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.r#type) == look_up_type_id(other.r#type)
    }
}

impl TypedStorageField {
    pub fn new(name: Ident, r#type: TypeId, span: Span) -> Self {
        TypedStorageField { name, r#type, span }
    }
}
