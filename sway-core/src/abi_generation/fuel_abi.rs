use fuel_abi_types::abi::program as program_abi;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{
    language::ty::{TyFunctionDecl, TyProgram, TyProgramKind},
    transform::AttributesMap,
    Engines, TypeId, TypeInfo, TypeParameter,
};

use super::abi_str::AbiStrContext;

pub struct AbiContext<'a> {
    pub program: &'a TyProgram,
    pub abi_with_callpaths: bool,
    pub type_ids_to_full_type_str: HashMap<String, String>,
}

impl<'a> AbiContext<'a> {
    fn to_str_context(
        &self,
        engines: &Engines,
        abi_with_fully_specified_types: bool,
    ) -> AbiStrContext {
        AbiStrContext {
            program_name: self
                .program
                .root
                .namespace
                .program_id(engines)
                .read(engines, |m| m.name().to_string()),
            abi_with_callpaths: self.abi_with_callpaths,
            abi_with_fully_specified_types,
        }
    }
}

impl TypeId {
    fn get_abi_type_id(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
    ) -> Result<String, ErrorEmitted> {
        let type_str =
            self.get_abi_type_str(&ctx.to_str_context(engines, true), engines, self.clone());
        let mut hasher = Sha256::new();
        hasher.update(type_str.clone());
        let result = hasher.finalize();
        let type_id = format!("{:x}", result);

        if let Some(old_type_str) = ctx
            .type_ids_to_full_type_str
            .insert(type_id.clone(), type_str.clone())
        {
            if old_type_str != type_str {
                return Err(
                    handler.emit_err(sway_error::error::CompileError::ABIHashCollision {
                        span: Span::dummy(),
                        hash: type_id,
                        first_type: old_type_str,
                        second_type: type_str,
                    }),
                );
            }
        }

        Ok(type_id)
    }
}

pub fn generate_program_abi(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    types: &mut Vec<program_abi::TypeDeclaration>,
    encoding: Option<program_abi::Version>,
    spec_version: program_abi::Version,
    abi_version: program_abi::Version,
) -> Result<program_abi::ProgramABI, ErrorEmitted> {
    let decl_engine = engines.de();
    let mut program_abi = match &ctx.program.kind {
        TyProgramKind::Contract { abi_entries, .. } => {
            let functions = abi_entries
                .iter()
                .map(|x| {
                    let fn_decl = decl_engine.get_function(x);
                    Ok(fn_decl.generate_abi_function(handler, ctx, engines, types)?)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let logged_types = generate_logged_types(handler, ctx, engines, types)?;
            let messages_types = generate_messages_types(handler, ctx, engines, types)?;
            let configurables = generate_configurables(handler, ctx, engines, types)?;
            program_abi::ProgramABI {
                program_type: "contract".to_string(),
                spec_version,
                abi_version,
                encoding,
                types: types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
            }
        }
        TyProgramKind::Script { main_function, .. } => {
            let main_function = decl_engine.get_function(main_function);
            let functions =
                vec![main_function.generate_abi_function(handler, ctx, engines, types)?];
            let logged_types = generate_logged_types(handler, ctx, engines, types)?;
            let messages_types = generate_messages_types(handler, ctx, engines, types)?;
            let configurables = generate_configurables(handler, ctx, engines, types)?;
            program_abi::ProgramABI {
                program_type: "script".to_string(),
                spec_version,
                abi_version,
                encoding,
                types: types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
            }
        }
        TyProgramKind::Predicate { main_function, .. } => {
            let main_function = decl_engine.get_function(main_function);
            let functions =
                vec![main_function.generate_abi_function(handler, ctx, engines, types)?];
            let logged_types = generate_logged_types(handler, ctx, engines, types)?;
            let messages_types = generate_messages_types(handler, ctx, engines, types)?;
            let configurables = generate_configurables(handler, ctx, engines, types)?;
            program_abi::ProgramABI {
                program_type: "predicate".to_string(),
                spec_version,
                abi_version,
                encoding,
                types: types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
            }
        }
        TyProgramKind::Library { .. } => program_abi::ProgramABI {
            program_type: "library".to_string(),
            spec_version,
            abi_version,
            encoding,
            types: vec![],
            functions: vec![],
            logged_types: None,
            messages_types: None,
            configurables: None,
        },
    };

    standardize_json_abi_types(&mut program_abi);

    Ok(program_abi)
}

/// Standardize the JSON ABI data structure by eliminating duplicate types.
fn standardize_json_abi_types(json_abi_program: &mut program_abi::ProgramABI) {
    // Two `program_abi::TypeDeclaration` are deemed the same if the have the same type_id
    let mut deduped_types: HashMap<String, program_abi::TypeDeclaration> =
        HashMap::<String, program_abi::TypeDeclaration>::new();

    // Insert values in `deduped_types` if they haven't been inserted before. Otherwise, check to see
    // the types are identical if not throw an error.
    for decl in &json_abi_program.types {
        if let Some(ty) = deduped_types.get(&decl.type_id) {
            if ty.type_field != decl.type_field
                || ty.components != decl.components
                || ty.type_parameters != decl.type_parameters
            {
                // We already throw an error on get_abi_type_id so this should not occur.
                panic!("There are conflicting type ids for different type declarations.")
            }
        } else {
            deduped_types.insert(decl.type_id.clone(), decl.clone());
        }
    }

    json_abi_program.types = deduped_types.values().cloned().collect::<Vec<_>>();

    // Sort the `program_abi::TypeDeclaration`s
    json_abi_program
        .types
        .sort_by(|t1, t2| t1.type_field.cmp(&t2.type_field));
}

fn generate_logged_types(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Result<Vec<program_abi::LoggedType>, ErrorEmitted> {
    // A list of all `program_abi::TypeDeclaration`s needed for the logged types
    let logged_types = ctx
        .program
        .logged_types
        .iter()
        .map(|(_, type_id)| {
            Ok(program_abi::TypeDeclaration {
                type_id: type_id.get_abi_type_id(handler, ctx, engines)?,
                type_field: type_id.get_abi_type_str(
                    &ctx.to_str_context(engines, false),
                    engines,
                    *type_id,
                ),
                components: type_id
                    .get_abi_type_components(handler, ctx, engines, types, *type_id)?,
                type_parameters: type_id
                    .get_abi_type_parameters(handler, ctx, engines, types, *type_id)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Add the new types to `types`
    types.extend(logged_types);

    // Generate the JSON data for the logged types
    let mut log_ids: HashSet<u64> = HashSet::default();
    Ok(ctx
        .program
        .logged_types
        .iter()
        .map(|(log_id, type_id)| {
            let log_id = log_id.hash_id;
            if log_ids.contains(&log_id) {
                Ok(None)
            } else {
                log_ids.insert(log_id);
                Ok(Some(program_abi::LoggedType {
                    log_id: log_id.to_string(),
                    application: program_abi::TypeApplication {
                        name: "".to_string(),
                        type_id: type_id.get_abi_type_id(handler, ctx, engines)?,
                        type_arguments: type_id
                            .get_abi_type_arguments(handler, ctx, engines, types, *type_id)?,
                    },
                }))
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter_map(|o| o)
        .collect())
}

fn generate_messages_types(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Result<Vec<program_abi::MessageType>, ErrorEmitted> {
    // A list of all `program_abi::TypeDeclaration`s needed for the messages types
    let messages_types = ctx
        .program
        .messages_types
        .iter()
        .map(|(_, type_id)| {
            Ok(program_abi::TypeDeclaration {
                type_id: type_id.get_abi_type_id(handler, ctx, engines)?,
                type_field: type_id.get_abi_type_str(
                    &ctx.to_str_context(engines, false),
                    engines,
                    *type_id,
                ),
                components: type_id
                    .get_abi_type_components(handler, ctx, engines, types, *type_id)?,
                type_parameters: type_id
                    .get_abi_type_parameters(handler, ctx, engines, types, *type_id)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Add the new types to `types`
    types.extend(messages_types);

    // Generate the JSON data for the messages types
    ctx.program
        .messages_types
        .iter()
        .map(|(message_id, type_id)| {
            Ok(program_abi::MessageType {
                message_id: **message_id as u64,
                application: program_abi::TypeApplication {
                    name: "".to_string(),
                    type_id: type_id.get_abi_type_id(handler, ctx, engines)?,
                    type_arguments: type_id
                        .get_abi_type_arguments(handler, ctx, engines, types, *type_id)?,
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn generate_configurables(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Result<Vec<program_abi::Configurable>, ErrorEmitted> {
    // A list of all `program_abi::TypeDeclaration`s needed for the configurables types
    let configurables_types = ctx
        .program
        .configurables
        .iter()
        .map(|decl| {
            Ok(program_abi::TypeDeclaration {
                type_id: decl
                    .type_ascription
                    .type_id
                    .get_abi_type_id(handler, ctx, engines)?,
                type_field: decl.type_ascription.type_id.get_abi_type_str(
                    &ctx.to_str_context(engines, false),
                    engines,
                    decl.type_ascription.type_id,
                ),
                components: decl.type_ascription.type_id.get_abi_type_components(
                    handler,
                    ctx,
                    engines,
                    types,
                    decl.type_ascription.type_id,
                )?,
                type_parameters: decl.type_ascription.type_id.get_abi_type_parameters(
                    handler,
                    ctx,
                    engines,
                    types,
                    decl.type_ascription.type_id,
                )?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Add the new types to `types`
    types.extend(configurables_types);

    // Generate the JSON data for the configurables types
    ctx.program
        .configurables
        .iter()
        .map(|decl| {
            Ok(program_abi::Configurable {
                name: decl.call_path.suffix.to_string(),
                application: program_abi::TypeApplication {
                    name: "".to_string(),
                    type_id: decl
                        .type_ascription
                        .type_id
                        .get_abi_type_id(handler, ctx, engines)?,
                    type_arguments: decl.type_ascription.type_id.get_abi_type_arguments(
                        handler,
                        ctx,
                        engines,
                        types,
                        decl.type_ascription.type_id,
                    )?,
                },
                offset: 0,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

impl TypeId {
    /// Return the type parameters of a given (potentially generic) type while considering what it
    /// actually resolves to. These parameters are essentially of type of `usize` which are
    /// basically the IDs of some set of `program_abi::TypeDeclaration`s. The method below also
    /// updates the provide list of `program_abi::TypeDeclaration`s  to add the newly discovered
    /// types.
    pub(self) fn get_abi_type_parameters(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Result<Option<Vec<String>>, ErrorEmitted> {
        match self.is_generic_parameter(engines, resolved_type_id) {
            true => Ok(None),
            false => resolved_type_id
                .get_type_parameters(engines)
                .map(|v| {
                    v.iter()
                        .map(|v| Ok(v.get_abi_type_parameter(handler, ctx, engines, types)?))
                        .collect::<Result<Vec<_>, _>>()
                })
                .map_or(Ok(None), |v| v.map(Some)),
        }
    }
    /// Return the components of a given (potentially generic) type while considering what it
    /// actually resolves to. These components are essentially of type of
    /// `program_abi::TypeApplication`.  The method below also updates the provided list of
    /// `program_abi::TypeDeclaration`s  to add the newly discovered types.
    pub(self) fn get_abi_type_components(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Result<Option<Vec<program_abi::TypeApplication>>, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        Ok(match &*type_engine.get(*self) {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                // A list of all `program_abi::TypeDeclaration`s needed for the enum variants
                let variants = decl
                    .variants
                    .iter()
                    .map(|x| {
                        Ok(program_abi::TypeDeclaration {
                            type_id: x
                                .type_argument
                                .initial_type_id
                                .get_abi_type_id(handler, ctx, engines)?,
                            type_field: x.type_argument.initial_type_id.get_abi_type_str(
                                &ctx.to_str_context(engines, false),
                                engines,
                                x.type_argument.type_id,
                            ),
                            components: x.type_argument.initial_type_id.get_abi_type_components(
                                handler,
                                ctx,
                                engines,
                                types,
                                x.type_argument.type_id,
                            )?,
                            type_parameters: x
                                .type_argument
                                .initial_type_id
                                .get_abi_type_parameters(
                                    handler,
                                    ctx,
                                    engines,
                                    types,
                                    x.type_argument.type_id,
                                )?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                types.extend(variants);

                // Generate the JSON data for the enum. This is basically a list of
                // `program_abi::TypeApplication`s
                Some(
                    decl.variants
                        .iter()
                        .map(|x| {
                            Ok(program_abi::TypeApplication {
                                name: x.name.to_string(),
                                type_id: x
                                    .type_argument
                                    .initial_type_id
                                    .get_abi_type_id(handler, ctx, engines)?,
                                type_arguments: x
                                    .type_argument
                                    .initial_type_id
                                    .get_abi_type_arguments(
                                        handler,
                                        ctx,
                                        engines,
                                        types,
                                        x.type_argument.type_id,
                                    )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);

                // A list of all `program_abi::TypeDeclaration`s needed for the struct fields
                let field_types = decl
                    .fields
                    .iter()
                    .map(|x| {
                        Ok(program_abi::TypeDeclaration {
                            type_id: x
                                .type_argument
                                .initial_type_id
                                .get_abi_type_id(handler, ctx, engines)?,
                            type_field: x.type_argument.initial_type_id.get_abi_type_str(
                                &ctx.to_str_context(engines, false),
                                engines,
                                x.type_argument.type_id,
                            ),
                            components: x.type_argument.initial_type_id.get_abi_type_components(
                                handler,
                                ctx,
                                engines,
                                types,
                                x.type_argument.type_id,
                            )?,
                            type_parameters: x
                                .type_argument
                                .initial_type_id
                                .get_abi_type_parameters(
                                    handler,
                                    ctx,
                                    engines,
                                    types,
                                    x.type_argument.type_id,
                                )?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                types.extend(field_types);

                // Generate the JSON data for the struct. This is basically a list of
                // `program_abi::TypeApplication`s
                Some(
                    decl.fields
                        .iter()
                        .map(|x| {
                            Ok(program_abi::TypeApplication {
                                name: x.name.to_string(),
                                type_id: x
                                    .type_argument
                                    .initial_type_id
                                    .get_abi_type_id(handler, ctx, engines)?,
                                type_arguments: x
                                    .type_argument
                                    .initial_type_id
                                    .get_abi_type_arguments(
                                        handler,
                                        ctx,
                                        engines,
                                        types,
                                        x.type_argument.type_id,
                                    )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            TypeInfo::Array(..) => {
                if let TypeInfo::Array(elem_ty, _) = &*type_engine.get(resolved_type_id) {
                    // The `program_abi::TypeDeclaration`s needed for the array element type
                    let elem_abi_ty = program_abi::TypeDeclaration {
                        type_id: elem_ty
                            .initial_type_id
                            .get_abi_type_id(handler, ctx, engines)?,
                        type_field: elem_ty.initial_type_id.get_abi_type_str(
                            &ctx.to_str_context(engines, false),
                            engines,
                            elem_ty.type_id,
                        ),
                        components: elem_ty.initial_type_id.get_abi_type_components(
                            handler,
                            ctx,
                            engines,
                            types,
                            elem_ty.type_id,
                        )?,
                        type_parameters: elem_ty.initial_type_id.get_abi_type_parameters(
                            handler,
                            ctx,
                            engines,
                            types,
                            elem_ty.type_id,
                        )?,
                    };
                    types.push(elem_abi_ty);

                    // Generate the JSON data for the array. This is basically a single
                    // `program_abi::TypeApplication` for the array element type
                    Some(vec![program_abi::TypeApplication {
                        name: "__array_element".to_string(),
                        type_id: elem_ty
                            .initial_type_id
                            .get_abi_type_id(handler, ctx, engines)?,
                        type_arguments: elem_ty.initial_type_id.get_abi_type_arguments(
                            handler,
                            ctx,
                            engines,
                            types,
                            elem_ty.type_id,
                        )?,
                    }])
                } else {
                    unreachable!();
                }
            }
            TypeInfo::Slice(..) => {
                if let TypeInfo::Slice(elem_ty) = &*type_engine.get(resolved_type_id) {
                    // The `program_abi::TypeDeclaration`s needed for the slice element type
                    let elem_abi_ty = program_abi::TypeDeclaration {
                        type_id: elem_ty.initial_type_id.index(),
                        type_field: elem_ty.initial_type_id.get_abi_type_str(
                            &ctx.to_str_context(engines, false),
                            engines,
                            elem_ty.type_id,
                        ),
                        components: elem_ty.initial_type_id.get_abi_type_components(
                            ctx,
                            engines,
                            types,
                            elem_ty.type_id,
                        ),
                        type_parameters: elem_ty.initial_type_id.get_abi_type_parameters(
                            ctx,
                            engines,
                            types,
                            elem_ty.type_id,
                        ),
                    };
                    types.push(elem_abi_ty);

                    // Generate the JSON data for the array. This is basically a single
                    // `program_abi::TypeApplication` for the array element type
                    Some(vec![program_abi::TypeApplication {
                        name: "__slice_element".to_string(),
                        type_id: elem_ty.initial_type_id.index(),
                        type_arguments: elem_ty.initial_type_id.get_abi_type_arguments(
                            ctx,
                            engines,
                            types,
                            elem_ty.type_id,
                        ),
                    }])
                } else {
                    unreachable!();
                }
            }
            TypeInfo::Tuple(_) => {
                if let TypeInfo::Tuple(fields) = &*type_engine.get(resolved_type_id) {
                    // A list of all `program_abi::TypeDeclaration`s needed for the tuple fields
                    let fields_types = fields
                        .iter()
                        .map(|x| {
                            Ok(program_abi::TypeDeclaration {
                                type_id: x
                                    .initial_type_id
                                    .get_abi_type_id(handler, ctx, engines)?,
                                type_field: x.initial_type_id.get_abi_type_str(
                                    &ctx.to_str_context(engines, false),
                                    engines,
                                    x.type_id,
                                ),
                                components: x.initial_type_id.get_abi_type_components(
                                    handler, ctx, engines, types, x.type_id,
                                )?,
                                type_parameters: x.initial_type_id.get_abi_type_parameters(
                                    handler, ctx, engines, types, x.type_id,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    types.extend(fields_types);

                    // Generate the JSON data for the tuple. This is basically a list of
                    // `program_abi::TypeApplication`s
                    Some(
                        fields
                            .iter()
                            .map(|x| {
                                Ok(program_abi::TypeApplication {
                                    name: "__tuple_element".to_string(),
                                    type_id: x
                                        .initial_type_id
                                        .get_abi_type_id(handler, ctx, engines)?,
                                    type_arguments: x.initial_type_id.get_abi_type_arguments(
                                        handler, ctx, engines, types, x.type_id,
                                    )?,
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                } else {
                    unreachable!()
                }
            }
            TypeInfo::Custom { type_arguments, .. } => {
                if !self.is_generic_parameter(engines, resolved_type_id) {
                    // A list of all `program_abi::TypeDeclaration`s needed for the type arguments
                    let type_args = type_arguments
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .zip(
                            resolved_type_id
                                .get_type_parameters(engines)
                                .unwrap_or_default()
                                .iter(),
                        )
                        .map(|(v, p)| {
                            Ok(program_abi::TypeDeclaration {
                                type_id: v
                                    .initial_type_id
                                    .get_abi_type_id(handler, ctx, engines)?,
                                type_field: v.initial_type_id.get_abi_type_str(
                                    &ctx.to_str_context(engines, false),
                                    engines,
                                    p.type_id,
                                ),
                                components: v.initial_type_id.get_abi_type_components(
                                    handler, ctx, engines, types, p.type_id,
                                )?,
                                type_parameters: v.initial_type_id.get_abi_type_parameters(
                                    handler, ctx, engines, types, p.type_id,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    types.extend(type_args);

                    resolved_type_id.get_abi_type_components(
                        handler,
                        ctx,
                        engines,
                        types,
                        resolved_type_id,
                    )?
                } else {
                    None
                }
            }
            TypeInfo::Alias { .. } => {
                if let TypeInfo::Alias { ty, .. } = &*type_engine.get(resolved_type_id) {
                    ty.initial_type_id
                        .get_abi_type_components(handler, ctx, engines, types, ty.type_id)?
                } else {
                    None
                }
            }
            TypeInfo::UnknownGeneric { .. } => {
                // avoid infinite recursion
                if *self == resolved_type_id {
                    None
                } else {
                    resolved_type_id.get_abi_type_components(
                        handler,
                        ctx,
                        engines,
                        types,
                        resolved_type_id,
                    )?
                }
            }
            _ => None,
        })
    }

    /// Return the type arguments of a given (potentially generic) type while considering what it
    /// actually resolves to. These arguments are essentially of type of
    /// `program_abi::TypeApplication`. The method below also updates the provided list of
    /// `program_abi::TypeDeclaration`s  to add the newly discovered types.
    pub(self) fn get_abi_type_arguments(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Result<Option<Vec<program_abi::TypeApplication>>, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let resolved_params = resolved_type_id.get_type_parameters(engines);
        Ok(match &*type_engine.get(*self) {
            TypeInfo::Custom {
                type_arguments: Some(type_arguments),
                ..
            } => (!type_arguments.is_empty()).then_some({
                let resolved_params = resolved_params.unwrap_or_default();
                let abi_type_arguments = type_arguments
                    .iter()
                    .zip(resolved_params.iter())
                    .map(|(v, p)| {
                        Ok(program_abi::TypeDeclaration {
                            type_id: v.initial_type_id.get_abi_type_id(handler, ctx, engines)?,
                            type_field: v.initial_type_id.get_abi_type_str(
                                &ctx.to_str_context(engines, false),
                                engines,
                                p.type_id,
                            ),
                            components: v
                                .initial_type_id
                                .get_abi_type_components(handler, ctx, engines, types, p.type_id)?,
                            type_parameters: v
                                .initial_type_id
                                .get_abi_type_parameters(handler, ctx, engines, types, p.type_id)?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                types.extend(abi_type_arguments);

                type_arguments
                    .iter()
                    .map(|arg| {
                        Ok(program_abi::TypeApplication {
                            name: "".to_string(),
                            type_id: arg.initial_type_id.get_abi_type_id(handler, ctx, engines)?,
                            type_arguments: arg.initial_type_id.get_abi_type_arguments(
                                handler,
                                ctx,
                                engines,
                                types,
                                arg.type_id,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }),
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                // Here, type_id for each type parameter should contain resolved types
                let abi_type_arguments = decl
                    .type_parameters
                    .iter()
                    .map(|v| {
                        Ok(program_abi::TypeDeclaration {
                            type_id: v.type_id.get_abi_type_id(handler, ctx, engines)?,
                            type_field: v.type_id.get_abi_type_str(
                                &ctx.to_str_context(engines, false),
                                engines,
                                v.type_id,
                            ),
                            components: v
                                .type_id
                                .get_abi_type_components(handler, ctx, engines, types, v.type_id)?,
                            type_parameters: v
                                .type_id
                                .get_abi_type_parameters(handler, ctx, engines, types, v.type_id)?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                types.extend(abi_type_arguments);

                Some(
                    decl.type_parameters
                        .iter()
                        .map(|arg| {
                            Ok(program_abi::TypeApplication {
                                name: "".to_string(),
                                type_id: arg.type_id.get_abi_type_id(handler, ctx, engines)?,
                                type_arguments: arg.type_id.get_abi_type_arguments(
                                    handler,
                                    ctx,
                                    engines,
                                    types,
                                    arg.type_id,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }

            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                // Here, type_id for each type parameter should contain resolved types
                let abi_type_arguments = decl
                    .type_parameters
                    .iter()
                    .map(|v| {
                        Ok(program_abi::TypeDeclaration {
                            type_id: v.type_id.get_abi_type_id(handler, ctx, engines)?,
                            type_field: v.type_id.get_abi_type_str(
                                &ctx.to_str_context(engines, false),
                                engines,
                                v.type_id,
                            ),
                            components: v
                                .type_id
                                .get_abi_type_components(handler, ctx, engines, types, v.type_id)?,
                            type_parameters: v
                                .type_id
                                .get_abi_type_parameters(handler, ctx, engines, types, v.type_id)?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                types.extend(abi_type_arguments);

                Some(
                    decl.type_parameters
                        .iter()
                        .map(|arg| {
                            Ok(program_abi::TypeApplication {
                                name: "".to_string(),
                                type_id: arg.type_id.get_abi_type_id(handler, ctx, engines)?,
                                type_arguments: arg.type_id.get_abi_type_arguments(
                                    handler,
                                    ctx,
                                    engines,
                                    types,
                                    arg.type_id,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            _ => None,
        })
    }
}

impl TyFunctionDecl {
    pub(self) fn generate_abi_function(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
    ) -> Result<program_abi::ABIFunction, ErrorEmitted> {
        // A list of all `program_abi::TypeDeclaration`s needed for inputs
        let input_types = self
            .parameters
            .iter()
            .map(|x| {
                Ok(program_abi::TypeDeclaration {
                    type_id: x
                        .type_argument
                        .initial_type_id
                        .get_abi_type_id(handler, ctx, engines)?,
                    type_field: x.type_argument.initial_type_id.get_abi_type_str(
                        &ctx.to_str_context(engines, false),
                        engines,
                        x.type_argument.type_id,
                    ),
                    components: x.type_argument.initial_type_id.get_abi_type_components(
                        handler,
                        ctx,
                        engines,
                        types,
                        x.type_argument.type_id,
                    )?,
                    type_parameters: x.type_argument.type_id.get_abi_type_parameters(
                        handler,
                        ctx,
                        engines,
                        types,
                        x.type_argument.type_id,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // The single `program_abi::TypeDeclaration` needed for the output
        let output_type = program_abi::TypeDeclaration {
            type_id: self
                .return_type
                .initial_type_id
                .get_abi_type_id(handler, ctx, engines)?,
            type_field: self.return_type.initial_type_id.get_abi_type_str(
                &ctx.to_str_context(engines, false),
                engines,
                self.return_type.type_id,
            ),
            components: self.return_type.type_id.get_abi_type_components(
                handler,
                ctx,
                engines,
                types,
                self.return_type.type_id,
            )?,
            type_parameters: self.return_type.type_id.get_abi_type_parameters(
                handler,
                ctx,
                engines,
                types,
                self.return_type.type_id,
            )?,
        };

        // Add the new types to `types`
        types.extend(input_types);
        types.push(output_type);

        // Generate the JSON data for the function
        Ok(program_abi::ABIFunction {
            name: self.name.as_str().to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| {
                    Ok(program_abi::TypeApplication {
                        name: x.name.to_string(),
                        type_id: x
                            .type_argument
                            .initial_type_id
                            .get_abi_type_id(handler, ctx, engines)?,
                        type_arguments: x.type_argument.initial_type_id.get_abi_type_arguments(
                            handler,
                            ctx,
                            engines,
                            types,
                            x.type_argument.type_id,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            output: program_abi::TypeApplication {
                name: "".to_string(),
                type_id: self
                    .return_type
                    .initial_type_id
                    .get_abi_type_id(handler, ctx, engines)?,
                type_arguments: self.return_type.initial_type_id.get_abi_type_arguments(
                    handler,
                    ctx,
                    engines,
                    types,
                    self.return_type.type_id,
                )?,
            },
            attributes: generate_attributes_map(&self.attributes),
        })
    }
}

fn generate_attributes_map(attr_map: &AttributesMap) -> Option<Vec<program_abi::Attribute>> {
    if attr_map.is_empty() {
        None
    } else {
        Some(
            attr_map
                .iter()
                .flat_map(|(_attr_kind, attrs)| {
                    attrs.iter().map(|attr| program_abi::Attribute {
                        name: attr.name.to_string(),
                        arguments: attr.args.iter().map(|arg| arg.name.to_string()).collect(),
                    })
                })
                .collect(),
        )
    }
}

impl TypeParameter {
    /// Returns the initial type ID of a TypeParameter. Also updates the provided list of types to
    /// append the current TypeParameter as a `program_abi::TypeDeclaration`.
    pub(self) fn get_abi_type_parameter(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
    ) -> Result<String, ErrorEmitted> {
        let type_id = self
            .initial_type_id
            .get_abi_type_id(handler, ctx, engines)?;
        let type_parameter = program_abi::TypeDeclaration {
            type_id: type_id.clone(),
            type_field: self.initial_type_id.get_abi_type_str(
                &ctx.to_str_context(engines, false),
                engines,
                self.type_id,
            ),
            components: self.initial_type_id.get_abi_type_components(
                handler,
                ctx,
                engines,
                types,
                self.type_id,
            )?,
            type_parameters: None,
        };
        types.push(type_parameter);
        Ok(type_id)
    }
}
