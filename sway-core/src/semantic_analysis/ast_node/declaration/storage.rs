use crate::{
    error::*,
    ir_generation::{
        const_eval::compile_constant_expression_to_constant, storage::serialize_to_storage_slots,
    },
    language::ty,
    metadata::MetadataManager,
    type_system::{TypeId, TypeInfo},
    AttributesMap, Ident,
};
use fuel_tx::StorageSlot;
use sway_error::error::CompileError;
use sway_ir::{Context, Module};
use sway_types::{state::StateIndex, Span, Spanned};

impl ty::TyStorageDeclaration {
    pub fn new(fields: Vec<ty::TyStorageField>, span: Span, attributes: AttributesMap) -> Self {
        ty::TyStorageDeclaration {
            fields,
            span,
            attributes,
        }
    }
    /// Given a field, find its type information in the declaration and return it. If the field has not
    /// been declared as a part of storage, return an error.
    pub fn apply_storage_load(
        &self,
        fields: Vec<Ident>,
        storage_fields: &[ty::TyStorageField],
    ) -> CompileResult<(ty::TyStorageAccess, TypeId)> {
        let mut errors = vec![];
        let warnings = vec![];

        let mut type_checked_buf = vec![];
        let mut fields: Vec<_> = fields.into_iter().rev().collect();

        let first_field = fields.pop().expect("guaranteed by grammar");
        let (ix, initial_field_type) = match storage_fields
            .iter()
            .enumerate()
            .find(|(_, ty::TyStorageField { name, .. })| name == &first_field)
        {
            Some((
                ix,
                ty::TyStorageField {
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

        type_checked_buf.push(ty::TyStorageAccessDescriptor {
            name: first_field.clone(),
            type_id: *initial_field_type,
            span: first_field.span(),
        });

        fn update_available_struct_fields(id: TypeId) -> Vec<ty::TyStructField> {
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
                    type_checked_buf.push(ty::TyStorageAccessDescriptor {
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
                ty::TyStorageAccess {
                    fields: type_checked_buf,
                    ix,
                },
                return_type,
            ),
            warnings,
            errors,
        )
    }

    pub(crate) fn fields_as_typed_struct_fields(&self) -> Vec<ty::TyStructField> {
        self.fields
            .iter()
            .map(
                |ty::TyStorageField {
                     ref name,
                     type_id: ref r#type,
                     ref span,
                     ref initializer,
                     ref attributes,
                     ..
                 }| ty::TyStructField {
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

impl ty::TyStorageField {
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
