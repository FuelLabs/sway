use derivative::Derivative;
use sway_error::error::CompileError;
use sway_types::{state::StateIndex, Ident, Span, Spanned};

use crate::{error::*, language::ty::*, transform, type_system::*};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyStorageDeclaration {
    pub fields: Vec<TyStorageField>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl Spanned for TyStorageDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl TyStorageDeclaration {
    pub fn new(
        fields: Vec<TyStorageField>,
        span: Span,
        attributes: transform::AttributesMap,
    ) -> Self {
        TyStorageDeclaration {
            fields,
            span,
            attributes,
        }
    }

    /// Given a field, find its type information in the declaration and return it. If the field has not
    /// been declared as a part of storage, return an error.
    pub fn apply_storage_load(
        &self,
        type_engine: &TypeEngine,
        fields: Vec<Ident>,
        storage_fields: &[TyStorageField],
    ) -> CompileResult<(TyStorageAccess, TypeId)> {
        let mut errors = vec![];
        let warnings = vec![];

        let mut type_checked_buf = vec![];
        let mut fields: Vec<_> = fields.into_iter().rev().collect();

        let first_field = fields.pop().expect("guaranteed by grammar");
        let (ix, initial_field_type) = match storage_fields
            .iter()
            .enumerate()
            .find(|(_, TyStorageField { name, .. })| name == &first_field)
        {
            Some((
                ix,
                TyStorageField {
                    type_id: r#type, ..
                },
            )) => (StateIndex::new(ix), r#type),
            None => {
                errors.push(CompileError::StorageFieldDoesNotExist {
                    name: first_field.clone(),
                });
                return err(warnings, errors);
            }
        };

        type_checked_buf.push(TyStorageAccessDescriptor {
            name: first_field.clone(),
            type_id: *initial_field_type,
            span: first_field.span(),
        });

        let update_available_struct_fields = |id: TypeId| match type_engine.look_up_type_id(id) {
            TypeInfo::Struct { fields, .. } => fields,
            _ => vec![],
        };

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
                    type_checked_buf.push(TyStorageAccessDescriptor {
                        name: field.clone(),
                        type_id: struct_field.type_id,
                        span: field.span().clone(),
                    });
                    available_struct_fields = update_available_struct_fields(struct_field.type_id);
                }
                None => {
                    let available_fields = available_struct_fields
                        .iter()
                        .map(|x| x.name.as_str())
                        .collect::<Vec<_>>();
                    errors.push(CompileError::FieldNotFound {
                        field_name: field.clone(),
                        available_fields: available_fields.join(", "),
                        struct_name: type_checked_buf.last().unwrap().name.clone(),
                    });
                    return err(warnings, errors);
                }
            }
        }

        let return_type = type_checked_buf[type_checked_buf.len() - 1].type_id;

        ok(
            (
                TyStorageAccess {
                    fields: type_checked_buf,
                    ix,
                },
                return_type,
            ),
            warnings,
            errors,
        )
    }

    pub(crate) fn fields_as_typed_struct_fields(&self) -> Vec<TyStructField> {
        self.fields
            .iter()
            .map(
                |TyStorageField {
                     ref name,
                     type_id: ref r#type,
                     ref span,
                     ref initializer,
                     ref attributes,
                     ..
                 }| TyStructField {
                    name: name.clone(),
                    type_id: *r#type,
                    initial_type_id: *r#type,
                    span: span.clone(),
                    type_span: initializer.span.clone(),
                    attributes: attributes.clone(),
                },
            )
            .collect()
    }
}

#[derive(Clone, Debug, Eq)]
pub struct TyStorageField {
    pub name: Ident,
    pub type_id: TypeId,
    pub type_span: Span,
    pub initializer: TyExpression,
    pub(crate) span: Span,
    pub attributes: transform::AttributesMap,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyStorageField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.initializer == other.initializer
    }
}
