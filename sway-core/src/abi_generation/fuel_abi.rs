use fuel_abi_types::abi::program as program_abi;
use std::collections::HashSet;

use crate::{
    language::ty::{TyConstantDecl, TyFunctionDecl, TyProgram, TyProgramKind},
    transform::AttributesMap,
    Engines, TypeId, TypeInfo, TypeParameter,
};

use super::abi_str::AbiStrContext;

pub struct AbiContext<'a> {
    pub program: &'a TyProgram,
    pub abi_with_callpaths: bool,
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
                .read(engines, |m| m.name.clone().map(|v| v.as_str().to_string())),
            abi_with_callpaths: self.abi_with_callpaths,
            abi_with_fully_specified_types,
        }
    }
}

pub fn generate_program_abi(
    ctx: &mut AbiContext,
    engines: &Engines,
    types: &mut Vec<program_abi::TypeDeclaration>,
    encoding: Option<program_abi::Version>,
) -> program_abi::ProgramABI {
    let decl_engine = engines.de();
    match &ctx.program.kind {
        TyProgramKind::Contract { abi_entries, .. } => {
            let functions = abi_entries
                .iter()
                .map(|x| {
                    let fn_decl = decl_engine.get_function(x);
                    fn_decl.generate_abi_function(ctx, engines, types)
                })
                .collect();
            let logged_types = generate_logged_types(ctx, engines, types);
            let messages_types = generate_messages_types(ctx, engines, types);
            let configurables = generate_configurables(ctx, engines, types);
            program_abi::ProgramABI {
                encoding,
                types: types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
            }
        }
        TyProgramKind::Script { main_function, .. }
        | TyProgramKind::Predicate { main_function, .. } => {
            let main_function = decl_engine.get_function(main_function);
            let functions = vec![main_function.generate_abi_function(ctx, engines, types)];
            let logged_types = generate_logged_types(ctx, engines, types);
            let messages_types = generate_messages_types(ctx, engines, types);
            let configurables = generate_configurables(ctx, engines, types);
            program_abi::ProgramABI {
                encoding,
                types: types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
            }
        }
        _ => program_abi::ProgramABI {
            encoding,
            types: vec![],
            functions: vec![],
            logged_types: None,
            messages_types: None,
            configurables: None,
        },
    }
}

fn generate_logged_types(
    ctx: &mut AbiContext,
    engines: &Engines,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Vec<program_abi::LoggedType> {
    // A list of all `program_abi::TypeDeclaration`s needed for the logged types
    let logged_types = ctx
        .program
        .logged_types
        .iter()
        .map(|(_, type_id)| program_abi::TypeDeclaration {
            type_id: type_id.index(),
            type_field: type_id.get_abi_type_str(
                &ctx.to_str_context(engines, false),
                engines,
                *type_id,
            ),
            components: type_id.get_abi_type_components(ctx, engines, types, *type_id),
            type_parameters: type_id.get_abi_type_parameters(ctx, engines, types, *type_id),
        })
        .collect::<Vec<_>>();

    // Add the new types to `types`
    types.extend(logged_types);

    // Generate the JSON data for the logged types
    let mut log_ids: HashSet<u64> = HashSet::default();
    ctx.program
        .logged_types
        .iter()
        .filter_map(|(log_id, type_id)| {
            let log_id = log_id.hash_id;
            if log_ids.contains(&log_id) {
                None
            } else {
                log_ids.insert(log_id);
                Some(program_abi::LoggedType {
                    log_id: log_id.to_string(),
                    application: program_abi::TypeApplication {
                        name: "".to_string(),
                        type_id: type_id.index(),
                        type_arguments: type_id
                            .get_abi_type_arguments(ctx, engines, types, *type_id),
                    },
                })
            }
        })
        .collect()
}

fn generate_messages_types(
    ctx: &mut AbiContext,
    engines: &Engines,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Vec<program_abi::MessageType> {
    // A list of all `program_abi::TypeDeclaration`s needed for the messages types
    let messages_types = ctx
        .program
        .messages_types
        .iter()
        .map(|(_, type_id)| program_abi::TypeDeclaration {
            type_id: type_id.index(),
            type_field: type_id.get_abi_type_str(
                &ctx.to_str_context(engines, false),
                engines,
                *type_id,
            ),
            components: type_id.get_abi_type_components(ctx, engines, types, *type_id),
            type_parameters: type_id.get_abi_type_parameters(ctx, engines, types, *type_id),
        })
        .collect::<Vec<_>>();

    // Add the new types to `types`
    types.extend(messages_types);

    // Generate the JSON data for the messages types
    ctx.program
        .messages_types
        .iter()
        .map(|(message_id, type_id)| program_abi::MessageType {
            message_id: **message_id as u64,
            application: program_abi::TypeApplication {
                name: "".to_string(),
                type_id: type_id.index(),
                type_arguments: type_id.get_abi_type_arguments(ctx, engines, types, *type_id),
            },
        })
        .collect()
}

fn generate_configurables(
    ctx: &mut AbiContext,
    engines: &Engines,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Vec<program_abi::Configurable> {
    // A list of all `program_abi::TypeDeclaration`s needed for the configurables types
    let configurables_types = ctx
        .program
        .configurables
        .iter()
        .map(
            |TyConstantDecl {
                 type_ascription, ..
             }| program_abi::TypeDeclaration {
                type_id: type_ascription.type_id.index(),
                type_field: type_ascription.type_id.get_abi_type_str(
                    &ctx.to_str_context(engines, false),
                    engines,
                    type_ascription.type_id,
                ),
                components: type_ascription.type_id.get_abi_type_components(
                    ctx,
                    engines,
                    types,
                    type_ascription.type_id,
                ),
                type_parameters: type_ascription.type_id.get_abi_type_parameters(
                    ctx,
                    engines,
                    types,
                    type_ascription.type_id,
                ),
            },
        )
        .collect::<Vec<_>>();

    // Add the new types to `types`
    types.extend(configurables_types);

    // Generate the JSON data for the configurables types
    ctx.program
        .configurables
        .iter()
        .map(
            |TyConstantDecl {
                 call_path,
                 type_ascription,
                 ..
             }| program_abi::Configurable {
                name: call_path.suffix.to_string(),
                application: program_abi::TypeApplication {
                    name: "".to_string(),
                    type_id: type_ascription.type_id.index(),
                    type_arguments: type_ascription.type_id.get_abi_type_arguments(
                        ctx,
                        engines,
                        types,
                        type_ascription.type_id,
                    ),
                },
                offset: 0,
            },
        )
        .collect()
}

impl TypeId {
    /// Return the type parameters of a given (potentially generic) type while considering what it
    /// actually resolves to. These parameters are essentially of type of `usize` which are
    /// basically the IDs of some set of `program_abi::TypeDeclaration`s. The method below also
    /// updates the provide list of `program_abi::TypeDeclaration`s  to add the newly discovered
    /// types.
    pub(self) fn get_abi_type_parameters(
        &self,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<usize>> {
        match self.is_generic_parameter(engines, resolved_type_id) {
            true => None,
            false => resolved_type_id.get_type_parameters(engines).map(|v| {
                v.iter()
                    .map(|v| v.get_abi_type_parameter(ctx, engines, types))
                    .collect::<Vec<_>>()
            }),
        }
    }
    /// Return the components of a given (potentially generic) type while considering what it
    /// actually resolves to. These components are essentially of type of
    /// `program_abi::TypeApplication`.  The method below also updates the provided list of
    /// `program_abi::TypeDeclaration`s  to add the newly discovered types.
    pub(self) fn get_abi_type_components(
        &self,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<program_abi::TypeApplication>> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        match &*type_engine.get(*self) {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                // A list of all `program_abi::TypeDeclaration`s needed for the enum variants
                let variants = decl
                    .variants
                    .iter()
                    .map(|x| program_abi::TypeDeclaration {
                        type_id: x.type_argument.initial_type_id.index(),
                        type_field: x.type_argument.initial_type_id.get_abi_type_str(
                            &ctx.to_str_context(engines, false),
                            engines,
                            x.type_argument.type_id,
                        ),
                        components: x.type_argument.initial_type_id.get_abi_type_components(
                            ctx,
                            engines,
                            types,
                            x.type_argument.type_id,
                        ),
                        type_parameters: x.type_argument.initial_type_id.get_abi_type_parameters(
                            ctx,
                            engines,
                            types,
                            x.type_argument.type_id,
                        ),
                    })
                    .collect::<Vec<_>>();
                types.extend(variants);

                // Generate the JSON data for the enum. This is basically a list of
                // `program_abi::TypeApplication`s
                Some(
                    decl.variants
                        .iter()
                        .map(|x| program_abi::TypeApplication {
                            name: x.name.to_string(),
                            type_id: x.type_argument.initial_type_id.index(),
                            type_arguments: x.type_argument.initial_type_id.get_abi_type_arguments(
                                ctx,
                                engines,
                                types,
                                x.type_argument.type_id,
                            ),
                        })
                        .collect(),
                )
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);

                // A list of all `program_abi::TypeDeclaration`s needed for the struct fields
                let field_types = decl
                    .fields
                    .iter()
                    .map(|x| program_abi::TypeDeclaration {
                        type_id: x.type_argument.initial_type_id.index(),
                        type_field: x.type_argument.initial_type_id.get_abi_type_str(
                            &ctx.to_str_context(engines, false),
                            engines,
                            x.type_argument.type_id,
                        ),
                        components: x.type_argument.initial_type_id.get_abi_type_components(
                            ctx,
                            engines,
                            types,
                            x.type_argument.type_id,
                        ),
                        type_parameters: x.type_argument.initial_type_id.get_abi_type_parameters(
                            ctx,
                            engines,
                            types,
                            x.type_argument.type_id,
                        ),
                    })
                    .collect::<Vec<_>>();
                types.extend(field_types);

                // Generate the JSON data for the struct. This is basically a list of
                // `program_abi::TypeApplication`s
                Some(
                    decl.fields
                        .iter()
                        .map(|x| program_abi::TypeApplication {
                            name: x.name.to_string(),
                            type_id: x.type_argument.initial_type_id.index(),
                            type_arguments: x.type_argument.initial_type_id.get_abi_type_arguments(
                                ctx,
                                engines,
                                types,
                                x.type_argument.type_id,
                            ),
                        })
                        .collect(),
                )
            }
            TypeInfo::Array(..) => {
                if let TypeInfo::Array(elem_ty, _) = &*type_engine.get(resolved_type_id) {
                    // The `program_abi::TypeDeclaration`s needed for the array element type
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
                        name: "__array_element".to_string(),
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
                        .map(|x| program_abi::TypeDeclaration {
                            type_id: x.initial_type_id.index(),
                            type_field: x.initial_type_id.get_abi_type_str(
                                &ctx.to_str_context(engines, false),
                                engines,
                                x.type_id,
                            ),
                            components: x
                                .initial_type_id
                                .get_abi_type_components(ctx, engines, types, x.type_id),
                            type_parameters: x
                                .initial_type_id
                                .get_abi_type_parameters(ctx, engines, types, x.type_id),
                        })
                        .collect::<Vec<_>>();

                    types.extend(fields_types);

                    // Generate the JSON data for the tuple. This is basically a list of
                    // `program_abi::TypeApplication`s
                    Some(
                        fields
                            .iter()
                            .map(|x| program_abi::TypeApplication {
                                name: "__tuple_element".to_string(),
                                type_id: x.initial_type_id.index(),
                                type_arguments: x
                                    .initial_type_id
                                    .get_abi_type_arguments(ctx, engines, types, x.type_id),
                            })
                            .collect(),
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
                        .map(|(v, p)| program_abi::TypeDeclaration {
                            type_id: v.initial_type_id.index(),
                            type_field: v.initial_type_id.get_abi_type_str(
                                &ctx.to_str_context(engines, false),
                                engines,
                                p.type_id,
                            ),
                            components: v
                                .initial_type_id
                                .get_abi_type_components(ctx, engines, types, p.type_id),
                            type_parameters: v
                                .initial_type_id
                                .get_abi_type_parameters(ctx, engines, types, p.type_id),
                        })
                        .collect::<Vec<_>>();
                    types.extend(type_args);

                    resolved_type_id.get_abi_type_components(ctx, engines, types, resolved_type_id)
                } else {
                    None
                }
            }
            TypeInfo::Alias { .. } => {
                if let TypeInfo::Alias { ty, .. } = &*type_engine.get(resolved_type_id) {
                    ty.initial_type_id
                        .get_abi_type_components(ctx, engines, types, ty.type_id)
                } else {
                    None
                }
            }
            TypeInfo::UnknownGeneric { .. } => {
                // avoid infinite recursion
                if *self == resolved_type_id {
                    None
                } else {
                    resolved_type_id.get_abi_type_components(ctx, engines, types, resolved_type_id)
                }
            }
            _ => None,
        }
    }

    /// Return the type arguments of a given (potentially generic) type while considering what it
    /// actually resolves to. These arguments are essentially of type of
    /// `program_abi::TypeApplication`. The method below also updates the provided list of
    /// `program_abi::TypeDeclaration`s  to add the newly discovered types.
    pub(self) fn get_abi_type_arguments(
        &self,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<program_abi::TypeApplication>> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let resolved_params = resolved_type_id.get_type_parameters(engines);
        match &*type_engine.get(*self) {
            TypeInfo::Custom {
                type_arguments: Some(type_arguments),
                ..
            } => (!type_arguments.is_empty()).then_some({
                let resolved_params = resolved_params.unwrap_or_default();
                let abi_type_arguments = type_arguments
                    .iter()
                    .zip(resolved_params.iter())
                    .map(|(v, p)| program_abi::TypeDeclaration {
                        type_id: v.initial_type_id.index(),
                        type_field: v.initial_type_id.get_abi_type_str(
                            &ctx.to_str_context(engines, false),
                            engines,
                            p.type_id,
                        ),
                        components: v
                            .initial_type_id
                            .get_abi_type_components(ctx, engines, types, p.type_id),
                        type_parameters: v
                            .initial_type_id
                            .get_abi_type_parameters(ctx, engines, types, p.type_id),
                    })
                    .collect::<Vec<_>>();
                types.extend(abi_type_arguments);

                type_arguments
                    .iter()
                    .map(|arg| program_abi::TypeApplication {
                        name: "".to_string(),
                        type_id: arg.initial_type_id.index(),
                        type_arguments: arg.initial_type_id.get_abi_type_arguments(
                            ctx,
                            engines,
                            types,
                            arg.type_id,
                        ),
                    })
                    .collect::<Vec<_>>()
            }),
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                // Here, type_id for each type parameter should contain resolved types
                let abi_type_arguments = decl
                    .type_parameters
                    .iter()
                    .map(|v| program_abi::TypeDeclaration {
                        type_id: v.type_id.index(),
                        type_field: v.type_id.get_abi_type_str(
                            &ctx.to_str_context(engines, false),
                            engines,
                            v.type_id,
                        ),
                        components: v
                            .type_id
                            .get_abi_type_components(ctx, engines, types, v.type_id),
                        type_parameters: v
                            .type_id
                            .get_abi_type_parameters(ctx, engines, types, v.type_id),
                    })
                    .collect::<Vec<_>>();
                types.extend(abi_type_arguments);

                Some(
                    decl.type_parameters
                        .iter()
                        .map(|arg| program_abi::TypeApplication {
                            name: "".to_string(),
                            type_id: arg.type_id.index(),
                            type_arguments: arg.type_id.get_abi_type_arguments(
                                ctx,
                                engines,
                                types,
                                arg.type_id,
                            ),
                        })
                        .collect::<Vec<_>>(),
                )
            }

            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                // Here, type_id for each type parameter should contain resolved types
                let abi_type_arguments = decl
                    .type_parameters
                    .iter()
                    .map(|v| program_abi::TypeDeclaration {
                        type_id: v.type_id.index(),
                        type_field: v.type_id.get_abi_type_str(
                            &ctx.to_str_context(engines, false),
                            engines,
                            v.type_id,
                        ),
                        components: v
                            .type_id
                            .get_abi_type_components(ctx, engines, types, v.type_id),
                        type_parameters: v
                            .type_id
                            .get_abi_type_parameters(ctx, engines, types, v.type_id),
                    })
                    .collect::<Vec<_>>();
                types.extend(abi_type_arguments);

                Some(
                    decl.type_parameters
                        .iter()
                        .map(|arg| program_abi::TypeApplication {
                            name: "".to_string(),
                            type_id: arg.type_id.index(),
                            type_arguments: arg.type_id.get_abi_type_arguments(
                                ctx,
                                engines,
                                types,
                                arg.type_id,
                            ),
                        })
                        .collect::<Vec<_>>(),
                )
            }
            _ => None,
        }
    }
}

impl TyFunctionDecl {
    pub(self) fn generate_abi_function(
        &self,
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
    ) -> program_abi::ABIFunction {
        // A list of all `program_abi::TypeDeclaration`s needed for inputs
        let input_types = self
            .parameters
            .iter()
            .map(|x| program_abi::TypeDeclaration {
                type_id: x.type_argument.initial_type_id.index(),
                type_field: x.type_argument.initial_type_id.get_abi_type_str(
                    &ctx.to_str_context(engines, false),
                    engines,
                    x.type_argument.type_id,
                ),
                components: x.type_argument.initial_type_id.get_abi_type_components(
                    ctx,
                    engines,
                    types,
                    x.type_argument.type_id,
                ),
                type_parameters: x.type_argument.type_id.get_abi_type_parameters(
                    ctx,
                    engines,
                    types,
                    x.type_argument.type_id,
                ),
            })
            .collect::<Vec<_>>();

        // The single `program_abi::TypeDeclaration` needed for the output
        let output_type = program_abi::TypeDeclaration {
            type_id: self.return_type.initial_type_id.index(),
            type_field: self.return_type.initial_type_id.get_abi_type_str(
                &ctx.to_str_context(engines, false),
                engines,
                self.return_type.type_id,
            ),
            components: self.return_type.type_id.get_abi_type_components(
                ctx,
                engines,
                types,
                self.return_type.type_id,
            ),
            type_parameters: self.return_type.type_id.get_abi_type_parameters(
                ctx,
                engines,
                types,
                self.return_type.type_id,
            ),
        };

        // Add the new types to `types`
        types.extend(input_types);
        types.push(output_type);

        // Generate the JSON data for the function
        program_abi::ABIFunction {
            name: self.name.as_str().to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| program_abi::TypeApplication {
                    name: x.name.to_string(),
                    type_id: x.type_argument.initial_type_id.index(),
                    type_arguments: x.type_argument.initial_type_id.get_abi_type_arguments(
                        ctx,
                        engines,
                        types,
                        x.type_argument.type_id,
                    ),
                })
                .collect(),
            output: program_abi::TypeApplication {
                name: "".to_string(),
                type_id: self.return_type.initial_type_id.index(),
                type_arguments: self.return_type.initial_type_id.get_abi_type_arguments(
                    ctx,
                    engines,
                    types,
                    self.return_type.type_id,
                ),
            },
            attributes: generate_attributes_map(&self.attributes),
        }
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
        ctx: &mut AbiContext,
        engines: &Engines,
        types: &mut Vec<program_abi::TypeDeclaration>,
    ) -> usize {
        let type_parameter = program_abi::TypeDeclaration {
            type_id: self.initial_type_id.index(),
            type_field: self.initial_type_id.get_abi_type_str(
                &ctx.to_str_context(engines, false),
                engines,
                self.type_id,
            ),
            components: self.initial_type_id.get_abi_type_components(
                ctx,
                engines,
                types,
                self.type_id,
            ),
            type_parameters: None,
        };
        types.push(type_parameter);
        self.initial_type_id.index()
    }
}
