use crate::{
    ast_elements::type_argument::GenericTypeArgument,
    decl_engine::DeclEngine,
    language::ty,
    metadata::MetadataManager,
    type_system::{TypeId, TypeInfo},
    Engines, GenericArgument, TypeEngine,
};

use super::convert::convert_resolved_typeid_no_span;

use sway_error::error::CompileError;
use sway_ir::{Context, Module, Type};
use sway_types::span::Spanned;

pub(super) fn create_tagged_union_type(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    variants: &[ty::TyEnumVariant],
) -> Result<Type, CompileError> {
    // Create the enum aggregate first.  NOTE: single variant enums don't need an aggregate but are
    // getting one here anyway.  They don't need to be a tagged union either.
    let field_types: Vec<_> = variants
        .iter()
        .map(|variant| {
            convert_resolved_typeid_no_span(
                engines,
                context,
                md_mgr,
                module,
                None,
                variant.type_argument.type_id,
            )
        })
        .collect::<Result<Vec<_>, CompileError>>()?;

    // Enums where all the variants are unit types don't really need the union. Only a tag is
    // needed. For consistency, and to keep enums as reference types, we keep the tag in an
    // Aggregate.
    Ok(if field_types.iter().all(|f| f.is_unit(context)) {
        Type::new_struct(context, vec![Type::get_uint64(context)])
    } else {
        let u64_ty = Type::get_uint64(context);
        let union_ty = Type::new_union(context, field_types);
        Type::new_struct(context, vec![u64_ty, union_ty])
    })
}

pub(super) fn create_tuple_aggregate(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    fields: &[TypeId],
) -> Result<Type, CompileError> {
    let field_types = fields
        .iter()
        .map(|ty_id| {
            convert_resolved_typeid_no_span(engines, context, md_mgr, module, None, *ty_id)
        })
        .collect::<Result<Vec<_>, CompileError>>()?;

    Ok(Type::new_struct(context, field_types))
}

pub(super) fn create_array_aggregate(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    element_type_id: TypeId,
    count: u64,
) -> Result<Type, CompileError> {
    let element_type =
        convert_resolved_typeid_no_span(engines, context, md_mgr, module, None, element_type_id)?;
    Ok(Type::new_array(context, element_type, count))
}

pub(super) fn get_struct_for_types(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    type_ids: &[TypeId],
) -> Result<Type, CompileError> {
    let types = type_ids
        .iter()
        .map(|ty_id| {
            convert_resolved_typeid_no_span(engines, context, md_mgr, module, None, *ty_id)
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    Ok(Type::new_struct(context, types))
}

/// For the [TypeInfo::Struct] given by `struct_type_id` and the
/// [ty::ProjectionKind::StructField] given by `field_kind`
/// returns the name of the struct, and the field index within
/// the struct together with the field [TypeId] if the field exists
/// on the struct.
///
/// Returns `None` if the `struct_type_id` is not a [TypeInfo::Struct]
/// or an alias to a [TypeInfo::Struct] or if the `field_kind`
/// is not a [ty::ProjectionKind::StructField].
pub(super) fn get_struct_name_field_index_and_type(
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    struct_type_id: TypeId,
    field_kind: ty::ProjectionKind,
) -> Option<(String, Option<(u64, TypeId)>)> {
    let struct_ty_info = type_engine
        .to_typeinfo(struct_type_id, &field_kind.span())
        .ok()?;
    match (struct_ty_info, &field_kind) {
        (
            TypeInfo::Struct(decl_ref),
            ty::ProjectionKind::StructField {
                name: field_name,
                field_to_access: _,
            },
        ) => {
            let decl = decl_engine.get_struct(&decl_ref);
            Some((
                decl.call_path.suffix.as_str().to_owned(),
                decl.fields
                    .iter()
                    .enumerate()
                    .find(|(_, field)| field.name == *field_name)
                    .map(|(idx, field)| (idx as u64, field.type_argument.type_id)),
            ))
        }
        (
            TypeInfo::Alias {
                ty: GenericTypeArgument { type_id, .. },
                ..
            },
            _,
        ) => get_struct_name_field_index_and_type(type_engine, decl_engine, type_id, field_kind),
        _ => None,
    }
}

// To gather the indices into nested structs for the struct oriented IR instructions we need to
// inspect the names and types of a vector of fields in a path.  There are several different
// representations of this in the AST but we can wrap fetching the struct type and field name in a
// trait.  And we can even wrap the implementation in a macro.

pub(super) trait TypedNamedField {
    fn get_field_kind(&self) -> ty::ProjectionKind;
}

macro_rules! impl_typed_named_field_for {
    ($field_type_name: ident) => {
        impl TypedNamedField for $field_type_name {
            fn get_field_kind(&self) -> ty::ProjectionKind {
                ty::ProjectionKind::StructField {
                    name: self.name.clone(),
                    field_to_access: None,
                }
            }
        }
    };
}

impl TypedNamedField for ty::ProjectionKind {
    fn get_field_kind(&self) -> ty::ProjectionKind {
        self.clone()
    }
}

use ty::TyStorageAccessDescriptor;
impl_typed_named_field_for!(TyStorageAccessDescriptor);

pub(super) fn get_indices_for_struct_access(
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    base_type: TypeId,
    fields: &[impl TypedNamedField],
) -> Result<Vec<u64>, CompileError> {
    fields
        .iter()
        .try_fold(
            (Vec::new(), base_type),
            |(mut fld_idcs, prev_type_id), field| {
                let field_kind = field.get_field_kind();
                let ty_info = match type_engine.to_typeinfo(prev_type_id, &field_kind.span()) {
                    Ok(ty_info) => ty_info,
                    Err(error) => {
                        return Err(CompileError::InternalOwned(
                            format!("type error resolving type for reassignment: {error}"),
                            field_kind.span(),
                        ));
                    }
                };
                // Make sure we have an aggregate to index into.
                // Get the field index and also its type for the next iteration.
                match (ty_info, &field_kind) {
                    (
                        TypeInfo::Struct(decl_ref),
                        ty::ProjectionKind::StructField {
                            name: field_name,
                            field_to_access: _,
                        },
                    ) => {
                        let decl = decl_engine.get_struct(&decl_ref);
                        let field_idx_and_type_opt = decl
                            .fields
                            .iter()
                            .enumerate()
                            .find(|(_, field)| field.name == *field_name);
                        let (field_idx, field_type) = match field_idx_and_type_opt {
                            Some((idx, field)) => (idx as u64, field.type_argument.type_id),
                            None => {
                                return Err(CompileError::InternalOwned(
                                    format!(
                                        "Unknown field '{}' for struct {} in reassignment.",
                                        field_kind.pretty_print(),
                                        decl.call_path,
                                    ),
                                    field_kind.span(),
                                ));
                            }
                        };
                        // Save the field index.
                        fld_idcs.push(field_idx);
                        Ok((fld_idcs, field_type))
                    }
                    (TypeInfo::Tuple(fields), ty::ProjectionKind::TupleField { index, .. }) => {
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
