use crate::{
    error::CompileError,
    semantic_analysis::{
        ProjectionKind, TypeCheckedStorageAccessDescriptor, TypeCheckedStorageReassignDescriptor,
        TypedEnumVariant,
    },
    type_system::{to_typeinfo, TypeId, TypeInfo},
};

use super::convert::convert_resolved_typeid_no_span;

use sway_ir::{Aggregate, Context, Type};
use sway_types::span::Spanned;

pub(super) fn create_enum_aggregate(
    context: &mut Context,
    variants: Vec<TypedEnumVariant>,
) -> Result<Aggregate, CompileError> {
    // Create the enum aggregate first.  NOTE: single variant enums don't need an aggregate but are
    // getting one here anyway.  They don't need to be a tagged union either.
    let field_types: Vec<_> = variants
        .into_iter()
        .map(|tev| convert_resolved_typeid_no_span(context, &tev.type_id))
        .collect::<Result<Vec<_>, CompileError>>()?;

    // Enums where all the variants are unit types don't really need the union. Only a tag is
    // needed. For consistency, and to keep enums as reference types, we keep the tag in an
    // Aggregate.
    Ok(if field_types.iter().all(|f| matches!(f, Type::Unit)) {
        Aggregate::new_struct(context, vec![Type::Uint(64)])
    } else {
        let enum_aggregate = Aggregate::new_struct(context, field_types);
        Aggregate::new_struct(context, vec![Type::Uint(64), Type::Union(enum_aggregate)])
    })
}

pub(super) fn create_tuple_aggregate(
    context: &mut Context,
    fields: Vec<TypeId>,
) -> Result<Aggregate, CompileError> {
    let field_types = fields
        .into_iter()
        .map(|ty_id| convert_resolved_typeid_no_span(context, &ty_id))
        .collect::<Result<Vec<_>, CompileError>>()?;

    Ok(Aggregate::new_struct(context, field_types))
}

pub(super) fn create_array_aggregate(
    context: &mut Context,
    element_type_id: TypeId,
    count: u64,
) -> Result<Aggregate, CompileError> {
    let element_type = convert_resolved_typeid_no_span(context, &element_type_id)?;
    Ok(Aggregate::new_array(context, element_type, count))
}

pub(super) fn get_aggregate_for_types(
    context: &mut Context,
    type_ids: &[TypeId],
) -> Result<Aggregate, CompileError> {
    let types = type_ids
        .iter()
        .map(|ty_id| convert_resolved_typeid_no_span(context, ty_id))
        .collect::<Result<Vec<_>, CompileError>>()?;
    Ok(Aggregate::new_struct(context, types))
}

pub(super) fn get_struct_name_field_index_and_type(
    field_type: TypeId,
    field_kind: ProjectionKind,
) -> Option<(String, Option<(u64, TypeId)>)> {
    let ty_info = to_typeinfo(field_type, &field_kind.span()).ok()?;
    match (ty_info, field_kind) {
        (
            TypeInfo::Struct { name, fields, .. },
            ProjectionKind::StructField { name: field_name },
        ) => Some((
            name.as_str().to_owned(),
            fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == field_name)
                .map(|(idx, field)| (idx as u64, field.type_id)),
        )),
        _otherwise => None,
    }
}

// To gather the indices into nested structs for the struct oriented IR instructions we need to
// inspect the names and types of a vector of fields in a path.  There are several different
// representations of this in the AST but we can wrap fetching the struct type and field name in a
// trait.  And we can even wrap the implementation in a macro.

pub(super) trait TypedNamedField {
    fn get_field_kind(&self) -> ProjectionKind;
}

macro_rules! impl_typed_named_field_for {
    ($field_type_name: ident) => {
        impl TypedNamedField for $field_type_name {
            fn get_field_kind(&self) -> ProjectionKind {
                ProjectionKind::StructField {
                    name: self.name.clone(),
                }
            }
        }
    };
}

impl TypedNamedField for ProjectionKind {
    fn get_field_kind(&self) -> ProjectionKind {
        self.clone()
    }
}

impl_typed_named_field_for!(TypeCheckedStorageAccessDescriptor);
impl_typed_named_field_for!(TypeCheckedStorageReassignDescriptor);

pub(super) fn get_indices_for_struct_access<F: TypedNamedField>(
    base_type: TypeId,
    fields: &[F],
) -> Result<Vec<u64>, CompileError> {
    fields
        .iter()
        .try_fold(
            (Vec::new(), base_type),
            |(mut fld_idcs, prev_type_id), field| {
                let field_kind = field.get_field_kind();
                let ty_info = match to_typeinfo(prev_type_id, &field_kind.span()) {
                    Ok(ty_info) => ty_info,
                    Err(error) => {
                        return Err(CompileError::InternalOwned(
                            format!("type error resolving type for reassignment: {}", error),
                            field_kind.span(),
                        ));
                    }
                };
                // Make sure we have an aggregate to index into.
                // Get the field index and also its type for the next iteration.
                match (ty_info, &field_kind) {
                    (
                        TypeInfo::Struct { name, fields, .. },
                        ProjectionKind::StructField { name: field_name },
                    ) => {
                        let field_idx_and_type_opt = fields
                            .iter()
                            .enumerate()
                            .find(|(_, field)| field.name == *field_name);
                        let (field_idx, field_type) = match field_idx_and_type_opt {
                            Some((idx, field)) => (idx as u64, field.type_id),
                            None => {
                                return Err(CompileError::InternalOwned(
                                    format!(
                                        "Unknown field '{}' for struct {} in reassignment.",
                                        field_kind.pretty_print(),
                                        name,
                                    ),
                                    field_kind.span(),
                                ));
                            }
                        };
                        // Save the field index.
                        fld_idcs.push(field_idx);
                        Ok((fld_idcs, field_type))
                    }
                    (TypeInfo::Tuple(fields), ProjectionKind::TupleField { index, .. }) => {
                        let field_type = match fields.get(*index) {
                            Some(field_type_argument) => field_type_argument.type_id,
                            None => {
                                return Err(CompileError::InternalOwned(
                                    format!(
                                        "index {} is out of bounds for tuple of length {}",
                                        index,
                                        fields.len(),
                                    ),
                                    field_kind.span(),
                                ));
                            }
                        };
                        fld_idcs.push(*index as u64);
                        Ok((fld_idcs, field_type))
                    }
                    _ => Err(CompileError::Internal(
                        "Unknown aggregate in reassignment.",
                        field_kind.span(),
                    )),
                }
            },
        )
        .map(|(fld_idcs, _)| fld_idcs)
}
