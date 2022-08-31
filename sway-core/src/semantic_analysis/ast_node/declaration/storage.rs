use crate::{
    error::*,
    ir_generation::{
        const_eval::compile_constant_expression_to_constant, storage::serialize_to_storage_slots,
    },
    metadata::MetadataManager,
    semantic_analysis::{
        TypeCheckedStorageAccess, TypeCheckedStorageAccessDescriptor, TypedExpression,
        TypedStructField,
    },
    type_system::{look_up_type_id, TypeId, TypeInfo},
    Ident,
};
use derivative::Derivative;
use fuel_tx::StorageSlot;
use sway_ir::{Context, Module};
use sway_types::{state::StateIndex, Span, Spanned};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedStorageDeclaration {
    pub fields: Vec<TypedStorageField>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub span: Span,
}

impl Spanned for TypedStorageDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
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
            Some((
                ix,
                TypedStorageField {
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

        type_checked_buf.push(TypeCheckedStorageAccessDescriptor {
            name: first_field.clone(),
            type_id: *initial_field_type,
            span: first_field.span(),
        });

        fn update_available_struct_fields(id: TypeId) -> Vec<TypedStructField> {
            match crate::type_system::look_up_type_id(id) {
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

    pub(crate) fn fields_as_typed_struct_fields(&self) -> Vec<TypedStructField> {
        self.fields
            .iter()
            .map(
                |TypedStorageField {
                     ref name,
                     type_id: ref r#type,
                     ref span,
                     ref initializer,
                     ..
                 }| TypedStructField {
                    name: name.clone(),
                    type_id: *r#type,
                    initial_type_id: *r#type,
                    span: span.clone(),
                    type_span: initializer.span.clone(),
                },
            )
            .collect()
    }

    pub(crate) fn get_initialized_storage_slots(
        &self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> CompileResult<Vec<StorageSlot>> {
        let mut errors = vec![];
        let storage_slots = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                f.get_initialized_storage_slots(context, md_mgr, module, &StateIndex::new(i))
            })
            .filter_map(|s| s.map_err(|e| errors.push(e)).ok())
            .flatten()
            .collect::<Vec<_>>();

        match errors.is_empty() {
            true => ok(storage_slots, vec![], vec![]),
            false => err(vec![], errors),
        }
    }
}

#[derive(Clone, Debug, Eq)]
pub struct TypedStorageField {
    pub name: Ident,
    pub type_id: TypeId,
    pub type_span: Span,
    pub initializer: TypedExpression,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedStorageField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.initializer == other.initializer
    }
}

impl TypedStorageField {
    pub fn new(
        name: Ident,
        r#type: TypeId,
        type_span: Span,
        initializer: TypedExpression,
        span: Span,
    ) -> Self {
        TypedStorageField {
            name,
            type_id: r#type,
            type_span,
            initializer,
            span,
        }
    }

    pub(crate) fn get_initialized_storage_slots(
        &self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
        ix: &StateIndex,
    ) -> Result<Vec<StorageSlot>, CompileError> {
        compile_constant_expression_to_constant(context, md_mgr, module, None, &self.initializer)
            .map(|constant| serialize_to_storage_slots(&constant, context, ix, &constant.ty, &[]))
    }
}
