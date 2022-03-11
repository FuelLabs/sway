use crate::semantic_analysis::{OwnedTypedStructField, TypeCheckedStorageAccess};
use crate::{error::*, type_engine::TypeId, Ident};
use sway_types::{state::StateIndex, Span};

#[derive(Clone, Debug)]
pub struct TypedStorageDeclaration {
    pub(crate) fields: Vec<TypedStorageField>,
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
        field: Ident,
        span: &Span,
    ) -> CompileResult<(TypeCheckedStorageAccess, TypeId)> {
        if let Some((ix, TypedStorageField { r#type, name, .. })) = self.find_field(&field) {
            ok(
                (
                    TypeCheckedStorageAccess::new_load(ix, name.clone(), span),
                    *r#type,
                ),
                vec![],
                vec![],
            )
        } else {
            todo!("storage field not found err")
        }
    }

    fn find_field(&self, field: &Ident) -> Option<(StateIndex, &TypedStorageField)> {
        self.fields
            .iter()
            .enumerate()
            .find(|(_ix, TypedStorageField { name, .. })| name == field)
            .map(|(ix, field)| (StateIndex::new(ix), field))
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub(crate) fn fields_as_owned_typed_struct_fields(&self) -> Vec<OwnedTypedStructField> {
        self.fields
            .iter()
            .map(
                |TypedStorageField {
                     ref name,
                     ref r#type,
                     ..
                 }| OwnedTypedStructField {
                    name: name.as_str().to_string(),
                    r#type: *r#type,
                },
            )
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct TypedStorageField {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    // TODO send initializers in the TX
    //    pub(crate) initializer: TypedExpression,
}

impl TypedStorageField {
    pub fn new(name: Ident, r#type: TypeId) -> Self {
        TypedStorageField {
            name,
            r#type,
            //            initializer,
        }
    }
}
