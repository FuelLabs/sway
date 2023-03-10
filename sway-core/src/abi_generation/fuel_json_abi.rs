use fuel_abi_types::program_abi;
use sway_types::integer_bits::IntegerBits;

use crate::{
    decl_engine::DeclEngine,
    language::{
        ty::{TyConstantDeclaration, TyFunctionDeclaration, TyProgram, TyProgramKind},
        CallPath,
    },
    transform::AttributesMap,
    TypeArgument, TypeEngine, TypeId, TypeInfo, TypeParameter,
};

pub struct JsonAbiContext<'a> {
    pub program: &'a TyProgram,
    pub json_abi_with_callpaths: bool,
}

pub fn generate_json_abi_program(
    ctx: &mut JsonAbiContext,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> program_abi::ProgramABI {
    match &ctx.program.kind {
        TyProgramKind::Contract { abi_entries, .. } => {
            let functions = abi_entries
                .iter()
                .map(|x| x.generate_json_abi_function(ctx, type_engine, decl_engine, types))
                .collect();
            let logged_types = generate_json_logged_types(ctx, type_engine, decl_engine, types);
            let messages_types = generate_json_messages_types(ctx, type_engine, decl_engine, types);
            let configurables = generate_json_configurables(ctx, type_engine, decl_engine, types);
            program_abi::ProgramABI {
                types: types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
            }
        }
        TyProgramKind::Script { main_function, .. }
        | TyProgramKind::Predicate { main_function, .. } => {
            let functions = vec![main_function.generate_json_abi_function(
                ctx,
                type_engine,
                decl_engine,
                types,
            )];
            let logged_types = generate_json_logged_types(ctx, type_engine, decl_engine, types);
            let messages_types = generate_json_messages_types(ctx, type_engine, decl_engine, types);
            let configurables = generate_json_configurables(ctx, type_engine, decl_engine, types);
            program_abi::ProgramABI {
                types: types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
            }
        }
        _ => program_abi::ProgramABI {
            types: vec![],
            functions: vec![],
            logged_types: None,
            messages_types: None,
            configurables: None,
        },
    }
}

fn generate_json_logged_types(
    ctx: &mut JsonAbiContext,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Vec<program_abi::LoggedType> {
    // A list of all `program_abi::TypeDeclaration`s needed for the logged types
    let logged_types = ctx
        .program
        .logged_types
        .iter()
        .map(|(_, type_id)| program_abi::TypeDeclaration {
            type_id: type_id.index(),
            type_field: type_id.get_json_type_str(ctx, type_engine, decl_engine, *type_id),
            components: type_id.get_json_type_components(
                ctx,
                type_engine,
                decl_engine,
                types,
                *type_id,
            ),
            type_parameters: type_id.get_json_type_parameters(
                ctx,
                type_engine,
                decl_engine,
                types,
                *type_id,
            ),
        })
        .collect::<Vec<_>>();

    // Add the new types to `types`
    types.extend(logged_types);

    // Generate the JSON data for the logged types
    ctx.program
        .logged_types
        .iter()
        .map(|(log_id, type_id)| program_abi::LoggedType {
            log_id: **log_id as u64,
            application: program_abi::TypeApplication {
                name: "".to_string(),
                type_id: type_id.index(),
                type_arguments: type_id.get_json_type_arguments(
                    ctx,
                    type_engine,
                    decl_engine,
                    types,
                    *type_id,
                ),
            },
        })
        .collect()
}

fn generate_json_messages_types(
    ctx: &mut JsonAbiContext,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Vec<program_abi::MessageType> {
    // A list of all `program_abi::TypeDeclaration`s needed for the messages types
    let messages_types = ctx
        .program
        .messages_types
        .iter()
        .map(|(_, type_id)| program_abi::TypeDeclaration {
            type_id: type_id.index(),
            type_field: type_id.get_json_type_str(ctx, type_engine, decl_engine, *type_id),
            components: type_id.get_json_type_components(
                ctx,
                type_engine,
                decl_engine,
                types,
                *type_id,
            ),
            type_parameters: type_id.get_json_type_parameters(
                ctx,
                type_engine,
                decl_engine,
                types,
                *type_id,
            ),
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
                type_arguments: type_id.get_json_type_arguments(
                    ctx,
                    type_engine,
                    decl_engine,
                    types,
                    *type_id,
                ),
            },
        })
        .collect()
}

fn generate_json_configurables(
    ctx: &mut JsonAbiContext,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    types: &mut Vec<program_abi::TypeDeclaration>,
) -> Vec<program_abi::Configurable> {
    // A list of all `program_abi::TypeDeclaration`s needed for the configurables types
    let configurables_types = ctx
        .program
        .configurables
        .iter()
        .map(
            |TyConstantDeclaration {
                 type_ascription, ..
             }| program_abi::TypeDeclaration {
                type_id: type_ascription.type_id.index(),
                type_field: type_ascription.type_id.get_json_type_str(
                    ctx,
                    type_engine,
                    decl_engine,
                    type_ascription.type_id,
                ),
                components: type_ascription.type_id.get_json_type_components(
                    ctx,
                    type_engine,
                    decl_engine,
                    types,
                    type_ascription.type_id,
                ),
                type_parameters: type_ascription.type_id.get_json_type_parameters(
                    ctx,
                    type_engine,
                    decl_engine,
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
            |TyConstantDeclaration {
                 call_path,
                 type_ascription,
                 ..
             }| program_abi::Configurable {
                name: call_path.suffix.to_string(),
                application: program_abi::TypeApplication {
                    name: "".to_string(),
                    type_id: type_ascription.type_id.index(),
                    type_arguments: type_ascription.type_id.get_json_type_arguments(
                        ctx,
                        type_engine,
                        decl_engine,
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
    /// Gives back a string that represents the type, considering what it resolves to
    pub(self) fn get_json_type_str(
        &self,
        ctx: &mut JsonAbiContext,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        resolved_type_id: TypeId,
    ) -> String {
        if self.is_generic_parameter(type_engine, decl_engine, resolved_type_id) {
            format!(
                "generic {}",
                type_engine
                    .get(*self)
                    .json_abi_str(ctx, type_engine, decl_engine)
            )
        } else {
            match (type_engine.get(*self), type_engine.get(resolved_type_id)) {
                (TypeInfo::Custom { .. }, TypeInfo::Struct { .. }) => type_engine
                    .get(resolved_type_id)
                    .json_abi_str(ctx, type_engine, decl_engine),
                (TypeInfo::Custom { .. }, TypeInfo::Enum { .. }) => type_engine
                    .get(resolved_type_id)
                    .json_abi_str(ctx, type_engine, decl_engine),
                (TypeInfo::Tuple(fields), TypeInfo::Tuple(resolved_fields)) => {
                    assert_eq!(fields.len(), resolved_fields.len());
                    let field_strs = fields
                        .iter()
                        .map(|_| "_".to_string())
                        .collect::<Vec<String>>();
                    format!("({})", field_strs.join(", "))
                }
                (TypeInfo::Array(_, count), TypeInfo::Array(_, resolved_count)) => {
                    assert_eq!(count.val(), resolved_count.val());
                    format!("[_; {}]", count.val())
                }
                (TypeInfo::Custom { .. }, _) => {
                    format!(
                        "generic {}",
                        type_engine
                            .get(*self)
                            .json_abi_str(ctx, type_engine, decl_engine)
                    )
                }
                _ => type_engine
                    .get(*self)
                    .json_abi_str(ctx, type_engine, decl_engine),
            }
        }
    }

    /// Return the type parameters of a given (potentially generic) type while considering what it
    /// actually resolves to. These parameters are essentially of type of `usize` which are
    /// basically the IDs of some set of `program_abi::TypeDeclaration`s. The method below also
    /// updates the provide list of `program_abi::TypeDeclaration`s  to add the newly discovered
    /// types.
    pub(self) fn get_json_type_parameters(
        &self,
        ctx: &mut JsonAbiContext,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<usize>> {
        match self.is_generic_parameter(type_engine, decl_engine, resolved_type_id) {
            true => None,
            false => resolved_type_id
                .get_type_parameters(type_engine, decl_engine)
                .map(|v| {
                    v.iter()
                        .map(|v| v.get_json_type_parameter(ctx, type_engine, decl_engine, types))
                        .collect::<Vec<_>>()
                }),
        }
    }
    /// Return the components of a given (potentially generic) type while considering what it
    /// actually resolves to. These components are essentially of type of
    /// `program_abi::TypeApplication`.  The method below also updates the provided list of
    /// `program_abi::TypeDeclaration`s  to add the newly discovered types.
    pub(self) fn get_json_type_components(
        &self,
        ctx: &mut JsonAbiContext,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<program_abi::TypeApplication>> {
        match type_engine.get(*self) {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(&decl_ref);
                // A list of all `program_abi::TypeDeclaration`s needed for the enum variants
                let variants = decl
                    .variants
                    .iter()
                    .map(|x| program_abi::TypeDeclaration {
                        type_id: x.type_argument.initial_type_id.index(),
                        type_field: x.type_argument.initial_type_id.get_json_type_str(
                            ctx,
                            type_engine,
                            decl_engine,
                            x.type_argument.type_id,
                        ),
                        components: x.type_argument.initial_type_id.get_json_type_components(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            x.type_argument.type_id,
                        ),
                        type_parameters: x.type_argument.initial_type_id.get_json_type_parameters(
                            ctx,
                            type_engine,
                            decl_engine,
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
                            type_arguments: x
                                .type_argument
                                .initial_type_id
                                .get_json_type_arguments(
                                    ctx,
                                    type_engine,
                                    decl_engine,
                                    types,
                                    x.type_argument.type_id,
                                ),
                        })
                        .collect(),
                )
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(&decl_ref);

                // A list of all `program_abi::TypeDeclaration`s needed for the struct fields
                let field_types = decl
                    .fields
                    .iter()
                    .map(|x| program_abi::TypeDeclaration {
                        type_id: x.type_argument.initial_type_id.index(),
                        type_field: x.type_argument.initial_type_id.get_json_type_str(
                            ctx,
                            type_engine,
                            decl_engine,
                            x.type_argument.type_id,
                        ),
                        components: x.type_argument.initial_type_id.get_json_type_components(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            x.type_argument.type_id,
                        ),
                        type_parameters: x.type_argument.initial_type_id.get_json_type_parameters(
                            ctx,
                            type_engine,
                            decl_engine,
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
                            type_arguments: x
                                .type_argument
                                .initial_type_id
                                .get_json_type_arguments(
                                    ctx,
                                    type_engine,
                                    decl_engine,
                                    types,
                                    x.type_argument.type_id,
                                ),
                        })
                        .collect(),
                )
            }
            TypeInfo::Array(..) => {
                if let TypeInfo::Array(elem_ty, _) = type_engine.get(resolved_type_id) {
                    // The `program_abi::TypeDeclaration`s needed for the array element type
                    let elem_json_ty = program_abi::TypeDeclaration {
                        type_id: elem_ty.initial_type_id.index(),
                        type_field: elem_ty.initial_type_id.get_json_type_str(
                            ctx,
                            type_engine,
                            decl_engine,
                            elem_ty.type_id,
                        ),
                        components: elem_ty.initial_type_id.get_json_type_components(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            elem_ty.type_id,
                        ),
                        type_parameters: elem_ty.initial_type_id.get_json_type_parameters(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            elem_ty.type_id,
                        ),
                    };
                    types.push(elem_json_ty);

                    // Generate the JSON data for the array. This is basically a single
                    // `program_abi::TypeApplication` for the array element type
                    Some(vec![program_abi::TypeApplication {
                        name: "__array_element".to_string(),
                        type_id: elem_ty.initial_type_id.index(),
                        type_arguments: elem_ty.initial_type_id.get_json_type_arguments(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            elem_ty.type_id,
                        ),
                    }])
                } else {
                    unreachable!();
                }
            }
            TypeInfo::Tuple(_) => {
                if let TypeInfo::Tuple(fields) = type_engine.get(resolved_type_id) {
                    // A list of all `program_abi::TypeDeclaration`s needed for the tuple fields
                    let fields_types = fields
                        .iter()
                        .map(|x| program_abi::TypeDeclaration {
                            type_id: x.initial_type_id.index(),
                            type_field: x.initial_type_id.get_json_type_str(
                                ctx,
                                type_engine,
                                decl_engine,
                                x.type_id,
                            ),
                            components: x.initial_type_id.get_json_type_components(
                                ctx,
                                type_engine,
                                decl_engine,
                                types,
                                x.type_id,
                            ),
                            type_parameters: x.initial_type_id.get_json_type_parameters(
                                ctx,
                                type_engine,
                                decl_engine,
                                types,
                                x.type_id,
                            ),
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
                                type_arguments: x.initial_type_id.get_json_type_arguments(
                                    ctx,
                                    type_engine,
                                    decl_engine,
                                    types,
                                    x.type_id,
                                ),
                            })
                            .collect(),
                    )
                } else {
                    unreachable!()
                }
            }
            TypeInfo::Custom { type_arguments, .. } => {
                if !self.is_generic_parameter(type_engine, decl_engine, resolved_type_id) {
                    // A list of all `program_abi::TypeDeclaration`s needed for the type arguments
                    let type_args = type_arguments
                        .unwrap_or_default()
                        .iter()
                        .zip(
                            resolved_type_id
                                .get_type_parameters(type_engine, decl_engine)
                                .unwrap_or_default()
                                .iter(),
                        )
                        .map(|(v, p)| program_abi::TypeDeclaration {
                            type_id: v.initial_type_id.index(),
                            type_field: v.initial_type_id.get_json_type_str(
                                ctx,
                                type_engine,
                                decl_engine,
                                p.type_id,
                            ),
                            components: v.initial_type_id.get_json_type_components(
                                ctx,
                                type_engine,
                                decl_engine,
                                types,
                                p.type_id,
                            ),
                            type_parameters: v.initial_type_id.get_json_type_parameters(
                                ctx,
                                type_engine,
                                decl_engine,
                                types,
                                p.type_id,
                            ),
                        })
                        .collect::<Vec<_>>();
                    types.extend(type_args);

                    resolved_type_id.get_json_type_components(
                        ctx,
                        type_engine,
                        decl_engine,
                        types,
                        resolved_type_id,
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Return the type arguments of a given (potentially generic) type while considering what it
    /// actually resolves to. These arguments are essentially of type of
    /// `program_abi::TypeApplication`. The method below also updates the provided list of
    /// `program_abi::TypeDeclaration`s  to add the newly discovered types.
    pub(self) fn get_json_type_arguments(
        &self,
        ctx: &mut JsonAbiContext,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        types: &mut Vec<program_abi::TypeDeclaration>,
        resolved_type_id: TypeId,
    ) -> Option<Vec<program_abi::TypeApplication>> {
        let resolved_params = resolved_type_id.get_type_parameters(type_engine, decl_engine);
        match type_engine.get(*self) {
            TypeInfo::Custom {
                type_arguments: Some(type_arguments),
                ..
            } => (!type_arguments.is_empty()).then_some({
                let resolved_params = resolved_params.unwrap_or_default();
                let json_type_arguments = type_arguments
                    .iter()
                    .zip(resolved_params.iter())
                    .map(|(v, p)| program_abi::TypeDeclaration {
                        type_id: v.initial_type_id.index(),
                        type_field: v.initial_type_id.get_json_type_str(
                            ctx,
                            type_engine,
                            decl_engine,
                            p.type_id,
                        ),
                        components: v.initial_type_id.get_json_type_components(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            p.type_id,
                        ),
                        type_parameters: v.initial_type_id.get_json_type_parameters(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            p.type_id,
                        ),
                    })
                    .collect::<Vec<_>>();
                types.extend(json_type_arguments);

                type_arguments
                    .iter()
                    .map(|arg| program_abi::TypeApplication {
                        name: "".to_string(),
                        type_id: arg.initial_type_id.index(),
                        type_arguments: arg.initial_type_id.get_json_type_arguments(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            arg.type_id,
                        ),
                    })
                    .collect::<Vec<_>>()
            }),
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(&decl_ref);
                // Here, type_id for each type parameter should contain resolved types
                let json_type_arguments = decl
                    .type_parameters
                    .iter()
                    .map(|v| program_abi::TypeDeclaration {
                        type_id: v.type_id.index(),
                        type_field: v.type_id.get_json_type_str(
                            ctx,
                            type_engine,
                            decl_engine,
                            v.type_id,
                        ),
                        components: v.type_id.get_json_type_components(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            v.type_id,
                        ),
                        type_parameters: v.type_id.get_json_type_parameters(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            v.type_id,
                        ),
                    })
                    .collect::<Vec<_>>();
                types.extend(json_type_arguments);

                Some(
                    decl.type_parameters
                        .iter()
                        .map(|arg| program_abi::TypeApplication {
                            name: "".to_string(),
                            type_id: arg.type_id.index(),
                            type_arguments: arg.type_id.get_json_type_arguments(
                                ctx,
                                type_engine,
                                decl_engine,
                                types,
                                arg.type_id,
                            ),
                        })
                        .collect::<Vec<_>>(),
                )
            }

            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(&decl_ref);
                // Here, type_id for each type parameter should contain resolved types
                let json_type_arguments = decl
                    .type_parameters
                    .iter()
                    .map(|v| program_abi::TypeDeclaration {
                        type_id: v.type_id.index(),
                        type_field: v.type_id.get_json_type_str(
                            ctx,
                            type_engine,
                            decl_engine,
                            v.type_id,
                        ),
                        components: v.type_id.get_json_type_components(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            v.type_id,
                        ),
                        type_parameters: v.type_id.get_json_type_parameters(
                            ctx,
                            type_engine,
                            decl_engine,
                            types,
                            v.type_id,
                        ),
                    })
                    .collect::<Vec<_>>();
                types.extend(json_type_arguments);

                Some(
                    decl.type_parameters
                        .iter()
                        .map(|arg| program_abi::TypeApplication {
                            name: "".to_string(),
                            type_id: arg.type_id.index(),
                            type_arguments: arg.type_id.get_json_type_arguments(
                                ctx,
                                type_engine,
                                decl_engine,
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

impl TypeInfo {
    pub fn json_abi_str(
        &self,
        ctx: &mut JsonAbiContext,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
    ) -> String {
        use TypeInfo::*;
        match self {
            Unknown => "unknown".into(),
            UnknownGeneric { name, .. } => name.to_string(),
            Placeholder(_) => "_".to_string(),
            TypeParam(n) => format!("typeparam({n})"),
            Str(x) => format!("str[{}]", x.val()),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
            }
            .into(),
            Boolean => "bool".into(),
            Custom { call_path, .. } => call_path.suffix.to_string(),
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| field.json_abi_str(ctx, type_engine, decl_engine))
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            SelfType => "Self".into(),
            B256 => "b256".into(),
            Numeric => "u64".into(), // u64 is the default
            Contract => "contract".into(),
            ErrorRecovery => "unknown due to error".into(),
            Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                format!("enum {}", call_path_display(ctx, &decl.call_path))
            }
            Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                format!("struct {}", call_path_display(ctx, &decl.call_path))
            }
            ContractCaller { abi_name, .. } => {
                format!("contract caller {abi_name}")
            }
            Array(elem_ty, length) => {
                format!(
                    "[{}; {}]",
                    elem_ty.json_abi_str(ctx, type_engine, decl_engine),
                    length.val()
                )
            }
            Storage { .. } => "contract storage".into(),
            RawUntypedPtr => "raw untyped ptr".into(),
            RawUntypedSlice => "raw untyped slice".into(),
        }
    }
}

/// `call_path_display`  returns the provided `call_path` without the first prefix in case it is equal to the program name.
/// If the program name is `my_program` and the `call_path` is `my_program::MyStruct` then this function returns only `MyStruct`.
fn call_path_display(ctx: &mut JsonAbiContext, call_path: &CallPath) -> String {
    if !ctx.json_abi_with_callpaths {
        return call_path.suffix.as_str().to_string();
    }
    let mut buf = String::new();
    for (index, prefix) in call_path.prefixes.iter().enumerate() {
        let mut skip_prefix = false;
        if index == 0 {
            if let Some(root_name) = ctx.program.root.namespace.name.clone() {
                if prefix.as_str() == root_name.as_str() {
                    skip_prefix = true;
                }
            }
        }
        if !skip_prefix {
            buf.push_str(prefix.as_str());
            buf.push_str("::");
        }
    }
    buf.push_str(&call_path.suffix.to_string());

    buf
}

impl TyFunctionDeclaration {
    pub(self) fn generate_json_abi_function(
        &self,
        ctx: &mut JsonAbiContext,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        types: &mut Vec<program_abi::TypeDeclaration>,
    ) -> program_abi::ABIFunction {
        // A list of all `program_abi::TypeDeclaration`s needed for inputs
        let input_types = self
            .parameters
            .iter()
            .map(|x| program_abi::TypeDeclaration {
                type_id: x.type_argument.initial_type_id.index(),
                type_field: x.type_argument.initial_type_id.get_json_type_str(
                    ctx,
                    type_engine,
                    decl_engine,
                    x.type_argument.type_id,
                ),
                components: x.type_argument.initial_type_id.get_json_type_components(
                    ctx,
                    type_engine,
                    decl_engine,
                    types,
                    x.type_argument.type_id,
                ),
                type_parameters: x.type_argument.type_id.get_json_type_parameters(
                    ctx,
                    type_engine,
                    decl_engine,
                    types,
                    x.type_argument.type_id,
                ),
            })
            .collect::<Vec<_>>();

        // The single `program_abi::TypeDeclaration` needed for the output
        let output_type = program_abi::TypeDeclaration {
            type_id: self.return_type.initial_type_id.index(),
            type_field: self.return_type.initial_type_id.get_json_type_str(
                ctx,
                type_engine,
                decl_engine,
                self.return_type.type_id,
            ),
            components: self.return_type.type_id.get_json_type_components(
                ctx,
                type_engine,
                decl_engine,
                types,
                self.return_type.type_id,
            ),
            type_parameters: self.return_type.type_id.get_json_type_parameters(
                ctx,
                type_engine,
                decl_engine,
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
                    type_arguments: x.type_argument.initial_type_id.get_json_type_arguments(
                        ctx,
                        type_engine,
                        decl_engine,
                        types,
                        x.type_argument.type_id,
                    ),
                })
                .collect(),
            output: program_abi::TypeApplication {
                name: "".to_string(),
                type_id: self.return_type.initial_type_id.index(),
                type_arguments: self.return_type.initial_type_id.get_json_type_arguments(
                    ctx,
                    type_engine,
                    decl_engine,
                    types,
                    self.return_type.type_id,
                ),
            },
            attributes: generate_json_abi_attributes_map(&self.attributes),
        }
    }
}

fn generate_json_abi_attributes_map(
    attr_map: &AttributesMap,
) -> Option<Vec<program_abi::Attribute>> {
    if attr_map.is_empty() {
        None
    } else {
        Some(
            attr_map
                .iter()
                .flat_map(|(_attr_kind, attrs)| {
                    attrs.iter().map(|attr| program_abi::Attribute {
                        name: attr.name.to_string(),
                        arguments: attr.args.iter().map(|arg| arg.to_string()).collect(),
                    })
                })
                .collect(),
        )
    }
}

impl TypeParameter {
    /// Returns the initial type ID of a TypeParameter. Also updates the provided list of types to
    /// append the current TypeParameter as a `program_abi::TypeDeclaration`.
    pub(self) fn get_json_type_parameter(
        &self,
        ctx: &mut JsonAbiContext,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        types: &mut Vec<program_abi::TypeDeclaration>,
    ) -> usize {
        let type_parameter = program_abi::TypeDeclaration {
            type_id: self.initial_type_id.index(),
            type_field: self.initial_type_id.get_json_type_str(
                ctx,
                type_engine,
                decl_engine,
                self.type_id,
            ),
            components: self.initial_type_id.get_json_type_components(
                ctx,
                type_engine,
                decl_engine,
                types,
                self.type_id,
            ),
            type_parameters: None,
        };
        types.push(type_parameter);
        self.initial_type_id.index()
    }
}

impl TypeArgument {
    pub(self) fn json_abi_str(
        &self,
        ctx: &mut JsonAbiContext,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
    ) -> String {
        type_engine
            .get(self.type_id)
            .json_abi_str(ctx, type_engine, decl_engine)
    }
}
