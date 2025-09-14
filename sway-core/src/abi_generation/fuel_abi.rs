use fuel_abi_types::abi::program::{
    self as program_abi, ConcreteTypeId, ErrorDetails, ErrorPosition, MetadataTypeId,
    PanickingCall, TypeConcreteDeclaration,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_parse::is_valid_identifier_or_path;
use sway_types::{Named, Span, Spanned};

use crate::{
    ast_elements::type_parameter::GenericTypeParameter,
    language::ty::{TyFunctionDecl, TyProgram, TyProgramKind},
    transform::Attributes,
    Engines, PanicOccurrences, PanickingCallOccurrences, TypeId, TypeInfo,
};

use super::abi_str::AbiStrContext;

#[derive(Clone, Debug)]
pub enum AbiNameDiagnosticSpan {
    Attribute(Span),
    Type(Span),
}

impl AbiNameDiagnosticSpan {
    pub fn span(self) -> Span {
        match self {
            Self::Attribute(span) | Self::Type(span) => span,
        }
    }
}

pub struct AbiContext<'a> {
    pub program: &'a TyProgram,
    pub panic_occurrences: &'a PanicOccurrences,
    pub panicking_call_occurrences: &'a PanickingCallOccurrences,
    pub abi_with_callpaths: bool,
    pub type_ids_to_full_type_str: HashMap<String, String>,
    pub unique_names: HashMap<String, AbiNameDiagnosticSpan>,
}

impl AbiContext<'_> {
    fn to_str_context(&self) -> AbiStrContext {
        AbiStrContext {
            program_name: self.program.namespace.current_package_name().to_string(),
            abi_with_callpaths: self.abi_with_callpaths,
            abi_with_fully_specified_types: false,
            abi_root_type_without_generic_type_parameters: true,
        }
    }
}

pub fn extract_abi_name_inner(span: &Span) -> Option<Span> {
    let text = &span.src().text;
    let full_attr = span.as_str();

    // Find the "name" key.
    let name_key_pos = full_attr.find("name")?;
    let after_name = &full_attr[name_key_pos..];

    // Find the '=' after "name".
    let eq_offset_rel = after_name.find('=')?;
    let after_eq = &after_name[eq_offset_rel + 1..];

    // Find the opening quote of the literal.
    let first_quote_rel = after_eq.find('"')?;
    let ident_abs_start = span.start()
        + name_key_pos
        + eq_offset_rel
        + 1        // move past '='
        + first_quote_rel
        + 1; // move past the opening '"'

    // Starting at ident_abs_start, locate the closing quote.
    let rest_after_ident = &text[ident_abs_start..span.end()];
    let second_quote_rel = rest_after_ident.find('"')?;
    let ident_abs_end = ident_abs_start + second_quote_rel;

    Span::new(
        span.src().clone(),
        ident_abs_start - 1,
        ident_abs_end + 1,
        span.source_id().cloned(),
    )
}

impl TypeId {
    fn get_abi_name_and_span_from_type_id(
        engines: &Engines,
        type_id: TypeId,
    ) -> Result<Option<(String, Span)>, ErrorEmitted> {
        let handler = Handler::default();
        match *engines.te().get(type_id) {
            TypeInfo::Enum(decl_id) => {
                let enum_decl = engines.de().get_enum(&decl_id);
                match enum_decl.attributes.abi_name() {
                    Some(abi_name_attr) => {
                        let name = abi_name_attr
                            .args
                            .first()
                            .unwrap()
                            .get_string(&handler, abi_name_attr)?;
                        Ok(Some((name.clone(), abi_name_attr.span.clone())))
                    }
                    None => Ok(None),
                }
            }
            TypeInfo::Struct(decl_id) => {
                let struct_decl = engines.de().get_struct(&decl_id);
                match struct_decl.attributes.abi_name() {
                    Some(abi_name_attr) => {
                        let name = abi_name_attr
                            .args
                            .first()
                            .unwrap()
                            .get_string(&handler, abi_name_attr)?;
                        Ok(Some((name.clone(), abi_name_attr.span.clone())))
                    }
                    None => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    fn get_abi_type_field_and_concrete_id(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        resolved_type_id: TypeId,
    ) -> Result<(String, ConcreteTypeId), ErrorEmitted> {
        let type_str = self.get_abi_type_str(
            handler,
            &AbiStrContext {
                program_name: ctx.program.namespace.current_package_name().to_string(),
                abi_with_callpaths: true,
                abi_with_fully_specified_types: true,
                abi_root_type_without_generic_type_parameters: false,
            },
            engines,
            resolved_type_id,
        )?;

        let mut err: Option<ErrorEmitted> = None;

        // Right now ABI renaming is only supported for enum and struct types.
        let should_check_name = matches!(
            *engines.te().get(resolved_type_id),
            TypeInfo::Enum(_) | TypeInfo::Struct(_)
        );

        if should_check_name {
            let (has_abi_name_attribute, name, attribute_span) =
                match Self::get_abi_name_and_span_from_type_id(engines, resolved_type_id)? {
                    Some(res) => (true, res.0, res.1),
                    None => (false, String::new(), Span::dummy()),
                };

            let attribute_name_span =
                extract_abi_name_inner(&attribute_span).unwrap_or(attribute_span.clone());

            let type_span = match *engines.te().get(resolved_type_id) {
                TypeInfo::Enum(decl_id) => engines.de().get_enum(&decl_id).name().span(),
                TypeInfo::Struct(decl_id) => engines.de().get_struct(&decl_id).name().span(),
                _ => unreachable!(),
            };

            let insert_span = if has_abi_name_attribute {
                AbiNameDiagnosticSpan::Attribute(attribute_span.clone())
            } else {
                AbiNameDiagnosticSpan::Type(type_span.clone())
            };

            let prev_span_opt = if ctx.unique_names.contains_key(&type_str) {
                ctx.unique_names.get(&type_str).cloned()
            } else {
                ctx.unique_names.insert(type_str.clone(), insert_span)
            };

            if has_abi_name_attribute {
                if name.is_empty()
                    || !is_valid_identifier_or_path(name.as_str())
                    || name.starts_with("::")
                {
                    err = Some(handler.emit_err(CompileError::ABIInvalidName {
                        span: attribute_name_span.clone(),
                        name,
                    }));
                }

                if let Some(prev_span) = prev_span_opt {
                    let is_attribute = matches!(prev_span, AbiNameDiagnosticSpan::Attribute(_));
                    err = Some(handler.emit_err(CompileError::ABIDuplicateName {
                        span: attribute_name_span.clone(),
                        other_span: prev_span.span(),
                        is_attribute,
                    }));
                }
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(type_str.clone());
        let result = hasher.finalize();
        let type_id = format!("{result:x}");

        if let Some(old_type_str) = ctx
            .type_ids_to_full_type_str
            .insert(type_id.clone(), type_str.clone())
        {
            if old_type_str != type_str {
                err = Some(
                    handler.emit_err(sway_error::error::CompileError::ABIHashCollision {
                        span: Span::dummy(),
                        hash: type_id.clone(),
                        first_type: old_type_str,
                        second_type: type_str.clone(),
                    }),
                );
            }
        }

        match err {
            Some(err) => Err(err),
            None => Ok((type_str, ConcreteTypeId(type_id))),
        }
    }
}

fn insert_unique_type(ctx: &mut AbiContext, name: String, span: Span) {
    let _ = ctx
        .unique_names
        .insert(name, AbiNameDiagnosticSpan::Type(span));
}

fn process_type_name(ctx: &mut AbiContext, engines: &Engines, type_id: TypeId) {
    match &*engines.te().get(type_id) {
        TypeInfo::Enum(decl_id) => {
            let enum_decl = engines.de().get_enum(decl_id);
            insert_unique_type(
                ctx,
                format!("enum {}", enum_decl.name()),
                enum_decl.name().span(),
            );
        }
        TypeInfo::Struct(decl_id) => {
            let struct_decl = engines.de().get_struct(decl_id);
            insert_unique_type(
                ctx,
                format!("struct {}", struct_decl.name()),
                struct_decl.name().span(),
            );
        }
        TypeInfo::Alias { name: _, ty } => process_type_name(ctx, engines, ty.type_id()),
        _ => {}
    }
}

fn process_type_names_from_function(
    ctx: &mut AbiContext,
    engines: &Engines,
    function: &TyFunctionDecl,
) {
    process_type_name(ctx, engines, function.return_type.type_id());

    for param in &function.parameters {
        process_type_name(ctx, engines, param.type_argument.type_id());
    }
}

pub fn generate_program_abi(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    encoding_version: program_abi::Version,
    spec_version: program_abi::Version,
) -> Result<program_abi::ProgramABI, ErrorEmitted> {
    let decl_engine = engines.de();
    let metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration> = &mut vec![];
    let concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration> = &mut vec![];

    match &ctx.program.kind {
        TyProgramKind::Contract { abi_entries, .. } => {
            abi_entries.iter().for_each(|x| {
                let fn_decl = decl_engine.get_function(x);
                process_type_names_from_function(ctx, engines, &fn_decl);
            });
        }
        TyProgramKind::Script { main_function, .. } => {
            let main_function = decl_engine.get_function(main_function);
            process_type_names_from_function(ctx, engines, &main_function);
        }
        TyProgramKind::Predicate { main_function, .. } => {
            let main_function = decl_engine.get_function(main_function);
            process_type_names_from_function(ctx, engines, &main_function);
        }
        TyProgramKind::Library { .. } => {}
    };

    let mut program_abi = handler.scope(|handler| match &ctx.program.kind {
        TyProgramKind::Contract { abi_entries, .. } => {
            let functions = abi_entries
                .iter()
                .map(|x| {
                    let fn_decl = decl_engine.get_function(x);
                    fn_decl.generate_abi_function(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                    )
                })
                .collect::<Vec<_>>();
            let logged_types =
                generate_logged_types(handler, ctx, engines, metadata_types, concrete_types)?;
            let messages_types =
                generate_messages_types(handler, ctx, engines, metadata_types, concrete_types)?;
            let configurables =
                generate_configurables(handler, ctx, engines, metadata_types, concrete_types)?;
            let error_codes = generate_error_codes(ctx.panic_occurrences);
            let panicking_calls = generate_panicking_calls(ctx.panicking_call_occurrences);
            Ok(program_abi::ProgramABI {
                program_type: "contract".to_string(),
                spec_version,
                encoding_version,
                metadata_types: metadata_types.to_vec(),
                concrete_types: concrete_types.to_vec(),
                functions: functions.into_iter().collect::<Result<Vec<_>, _>>()?,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
                error_codes: Some(error_codes),
                panicking_calls: Some(panicking_calls),
            })
        }
        TyProgramKind::Script { main_function, .. } => {
            let main_function = decl_engine.get_function(main_function);
            let functions = vec![main_function.generate_abi_function(
                handler,
                ctx,
                engines,
                metadata_types,
                concrete_types,
            )?];
            let logged_types =
                generate_logged_types(handler, ctx, engines, metadata_types, concrete_types)?;
            let messages_types =
                generate_messages_types(handler, ctx, engines, metadata_types, concrete_types)?;
            let configurables =
                generate_configurables(handler, ctx, engines, metadata_types, concrete_types)?;
            let error_codes = generate_error_codes(ctx.panic_occurrences);
            let panicking_calls = generate_panicking_calls(ctx.panicking_call_occurrences);
            Ok(program_abi::ProgramABI {
                program_type: "script".to_string(),
                spec_version,
                encoding_version,
                metadata_types: metadata_types.to_vec(),
                concrete_types: concrete_types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
                error_codes: Some(error_codes),
                panicking_calls: Some(panicking_calls),
            })
        }
        TyProgramKind::Predicate { main_function, .. } => {
            let main_function = decl_engine.get_function(main_function);
            let functions = vec![main_function.generate_abi_function(
                handler,
                ctx,
                engines,
                metadata_types,
                concrete_types,
            )?];
            let logged_types =
                generate_logged_types(handler, ctx, engines, metadata_types, concrete_types)?;
            let messages_types =
                generate_messages_types(handler, ctx, engines, metadata_types, concrete_types)?;
            let configurables =
                generate_configurables(handler, ctx, engines, metadata_types, concrete_types)?;
            let error_codes = generate_error_codes(ctx.panic_occurrences);
            let panicking_calls = generate_panicking_calls(ctx.panicking_call_occurrences);
            Ok(program_abi::ProgramABI {
                program_type: "predicate".to_string(),
                spec_version,
                encoding_version,
                metadata_types: metadata_types.to_vec(),
                concrete_types: concrete_types.to_vec(),
                functions,
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: Some(configurables),
                error_codes: Some(error_codes),
                panicking_calls: Some(panicking_calls),
            })
        }
        TyProgramKind::Library { .. } => {
            let logged_types =
                generate_logged_types(handler, ctx, engines, metadata_types, concrete_types)?;
            let messages_types =
                generate_messages_types(handler, ctx, engines, metadata_types, concrete_types)?;
            let error_codes = generate_error_codes(ctx.panic_occurrences);
            let panicking_calls = generate_panicking_calls(ctx.panicking_call_occurrences);
            Ok(program_abi::ProgramABI {
                program_type: "library".to_string(),
                spec_version,
                encoding_version,
                metadata_types: metadata_types.to_vec(),
                concrete_types: concrete_types.to_vec(),
                functions: vec![],
                logged_types: Some(logged_types),
                messages_types: Some(messages_types),
                configurables: None,
                error_codes: Some(error_codes),
                panicking_calls: Some(panicking_calls),
            })
        }
    })?;

    standardize_json_abi_types(&mut program_abi);

    Ok(program_abi)
}

/// Standardize the JSON ABI data structure by eliminating duplicate types. This is an iterative
/// process because every time two types are merged, new opportunities for more merging arise.
fn standardize_json_abi_types(json_abi_program: &mut program_abi::ProgramABI) {
    // Dedup TypeMetadataDeclaration
    loop {
        // If type with id_1 is a duplicate of type with id_2, then keep track of the mapping
        // between id_1 and id_2 in the HashMap below.
        let mut old_to_new_id: HashMap<MetadataTypeId, program_abi::TypeId> = HashMap::new();

        // A vector containing unique `program_abi::TypeMetadataDeclaration`s.
        //
        // Two `program_abi::TypeMetadataDeclaration` are deemed the same if the have the same
        // `type_field`, `components`, and `type_parameters` (even if their `type_id`s are
        // different).
        let mut deduped_types: Vec<program_abi::TypeMetadataDeclaration> = Vec::new();

        // Insert values in `deduped_types` if they haven't been inserted before. Otherwise, create
        // an appropriate mapping between type IDs in the HashMap `old_to_new_id`.
        for decl in &json_abi_program.metadata_types {
            // First replace metadata_type_id with concrete_type_id when possible
            if let Some(ty) = json_abi_program.concrete_types.iter().find(|d| {
                d.type_field == decl.type_field
                    && decl.components.is_none()
                    && decl.type_parameters.is_none()
            }) {
                old_to_new_id.insert(
                    decl.metadata_type_id.clone(),
                    program_abi::TypeId::Concrete(ty.concrete_type_id.clone()),
                );
            } else {
                // Second replace metadata_type_id with metadata_type_id when possible
                if let Some(ty) = deduped_types.iter().find(|d| {
                    d.type_field == decl.type_field
                        && d.components == decl.components
                        && d.type_parameters == decl.type_parameters
                }) {
                    old_to_new_id.insert(
                        decl.metadata_type_id.clone(),
                        program_abi::TypeId::Metadata(ty.metadata_type_id.clone()),
                    );
                } else {
                    deduped_types.push(decl.clone());
                }
            }
        }

        // Nothing to do if the hash map is empty as there are not merge opportunities. We can now
        // exit the loop.
        if old_to_new_id.is_empty() {
            break;
        }

        json_abi_program.metadata_types = deduped_types;

        update_all_types(json_abi_program, &old_to_new_id);
    }

    // Dedup TypeConcreteDeclaration
    let mut concrete_declarations_map: HashMap<ConcreteTypeId, TypeConcreteDeclaration> =
        HashMap::new();
    for decl in &json_abi_program.concrete_types {
        concrete_declarations_map.insert(decl.concrete_type_id.clone(), decl.clone());
    }
    json_abi_program.concrete_types = concrete_declarations_map.values().cloned().collect();

    // Sort the `program_abi::TypeMetadataDeclaration`s
    json_abi_program
        .metadata_types
        .sort_by(|t1, t2| t1.type_field.cmp(&t2.type_field));

    // Sort the `program_abi::TypeConcreteDeclaration`s
    json_abi_program
        .concrete_types
        .sort_by(|t1, t2| t1.type_field.cmp(&t2.type_field));

    // Standardize IDs (i.e. change them to 0,1,2,... according to the alphabetical order above
    let mut old_to_new_id: HashMap<MetadataTypeId, program_abi::TypeId> = HashMap::new();
    for (ix, decl) in json_abi_program.metadata_types.iter_mut().enumerate() {
        old_to_new_id.insert(
            decl.metadata_type_id.clone(),
            program_abi::TypeId::Metadata(MetadataTypeId(ix)),
        );
        decl.metadata_type_id = MetadataTypeId(ix);
    }

    update_all_types(json_abi_program, &old_to_new_id);
}

/// Recursively updates the type IDs used in a program_abi::ProgramABI
fn update_all_types(
    json_abi_program: &mut program_abi::ProgramABI,
    old_to_new_id: &HashMap<MetadataTypeId, program_abi::TypeId>,
) {
    // Update all `program_abi::TypeMetadataDeclaration`
    for decl in &mut json_abi_program.metadata_types {
        update_json_type_metadata_declaration(decl, old_to_new_id);
    }

    // Update all `program_abi::TypeConcreteDeclaration`
    for decl in &mut json_abi_program.concrete_types {
        update_json_type_concrete_declaration(decl, old_to_new_id);
    }
}

/// Recursively updates the type IDs used in a `program_abi::TypeApplication` given a HashMap from
/// old to new IDs
fn update_json_type_application(
    type_application: &mut program_abi::TypeApplication,
    old_to_new_id: &HashMap<MetadataTypeId, program_abi::TypeId>,
) {
    if let fuel_abi_types::abi::program::TypeId::Metadata(metadata_type_id) =
        &type_application.type_id
    {
        if let Some(new_id) = old_to_new_id.get(metadata_type_id) {
            type_application.type_id = new_id.clone();
        }
    }

    if let Some(args) = &mut type_application.type_arguments {
        for arg in args.iter_mut() {
            update_json_type_application(arg, old_to_new_id);
        }
    }
}

/// Recursively updates the metadata type IDs used in a `program_abi::TypeMetadataDeclaration` given a HashMap from
/// old to new IDs
fn update_json_type_metadata_declaration(
    type_declaration: &mut program_abi::TypeMetadataDeclaration,
    old_to_new_id: &HashMap<MetadataTypeId, program_abi::TypeId>,
) {
    if let Some(params) = &mut type_declaration.type_parameters {
        for param in params.iter_mut() {
            if let Some(fuel_abi_types::abi::program::TypeId::Metadata(new_id)) =
                old_to_new_id.get(param)
            {
                *param = new_id.clone();
            }
        }
    }

    if let Some(components) = &mut type_declaration.components {
        for component in components.iter_mut() {
            update_json_type_application(component, old_to_new_id);
        }
    }
}

/// Updates the metadata type IDs used in a `program_abi::TypeConcreteDeclaration` given a HashMap from
/// old to new IDs
fn update_json_type_concrete_declaration(
    type_declaration: &mut program_abi::TypeConcreteDeclaration,
    old_to_new_id: &HashMap<MetadataTypeId, program_abi::TypeId>,
) {
    if let Some(metadata_type_id) = &mut type_declaration.metadata_type_id {
        if let Some(fuel_abi_types::abi::program::TypeId::Metadata(new_id)) =
            old_to_new_id.get(metadata_type_id)
        {
            *metadata_type_id = new_id.clone();
        }
    }
}

fn generate_concrete_type_declaration(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
    concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
    type_id: TypeId,
    resolved_type_id: TypeId,
) -> Result<ConcreteTypeId, ErrorEmitted> {
    let mut new_metadata_types_to_add = Vec::<program_abi::TypeMetadataDeclaration>::new();
    let type_metadata_decl = program_abi::TypeMetadataDeclaration {
        metadata_type_id: MetadataTypeId(type_id.index()),
        type_field: type_id.get_abi_type_str(
            handler,
            &ctx.to_str_context(),
            engines,
            resolved_type_id,
        )?,
        components: type_id.get_abi_type_components(
            handler,
            ctx,
            engines,
            metadata_types,
            concrete_types,
            resolved_type_id,
            &mut new_metadata_types_to_add,
        )?,
        type_parameters: type_id.get_abi_type_parameters(
            handler,
            ctx,
            engines,
            metadata_types,
            concrete_types,
            resolved_type_id,
            &mut new_metadata_types_to_add,
        )?,
    };

    let metadata_type_id = if type_metadata_decl.type_parameters.is_some()
        || type_metadata_decl.components.is_some()
    {
        Some(type_metadata_decl.metadata_type_id.clone())
    } else {
        None
    };
    let type_arguments = handler.scope(|handler| {
        let type_arguments = if type_metadata_decl.type_parameters.is_some() {
            type_id.get_abi_type_arguments_as_concrete_type_ids(
                handler,
                ctx,
                engines,
                metadata_types,
                concrete_types,
                resolved_type_id,
            )?
        } else {
            None
        };
        Ok(type_arguments)
    })?;

    metadata_types.push(type_metadata_decl);
    metadata_types.extend(new_metadata_types_to_add);

    let (type_field, concrete_type_id) =
        type_id.get_abi_type_field_and_concrete_id(handler, ctx, engines, resolved_type_id)?;
    let concrete_type_decl = TypeConcreteDeclaration {
        type_field,
        concrete_type_id: concrete_type_id.clone(),
        metadata_type_id,
        type_arguments,
        alias_of: None,
    };

    concrete_types.push(concrete_type_decl);

    Ok(concrete_type_id)
}

#[allow(clippy::too_many_arguments)]
fn generate_type_metadata_declaration(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
    concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
    type_id: TypeId,
    resolved_type_id: TypeId,
    metadata_types_to_add: &mut Vec<program_abi::TypeMetadataDeclaration>,
) -> Result<(), ErrorEmitted> {
    let mut new_metadata_types_to_add = Vec::<program_abi::TypeMetadataDeclaration>::new();
    let components = type_id.get_abi_type_components(
        handler,
        ctx,
        engines,
        metadata_types,
        concrete_types,
        resolved_type_id,
        &mut new_metadata_types_to_add,
    )?;
    let type_parameters = type_id.get_abi_type_parameters(
        handler,
        ctx,
        engines,
        metadata_types,
        concrete_types,
        resolved_type_id,
        &mut new_metadata_types_to_add,
    )?;
    let type_metadata_decl = program_abi::TypeMetadataDeclaration {
        metadata_type_id: MetadataTypeId(type_id.index()),
        type_field: type_id.get_abi_type_str(
            handler,
            &ctx.to_str_context(),
            engines,
            resolved_type_id,
        )?,
        components,
        type_parameters,
    };

    metadata_types_to_add.push(type_metadata_decl.clone());
    metadata_types_to_add.extend(new_metadata_types_to_add);

    Ok(())
}

fn generate_logged_types(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
    concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
) -> Result<Vec<program_abi::LoggedType>, ErrorEmitted> {
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
                    concrete_type_id: generate_concrete_type_declaration(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                        *type_id,
                        *type_id,
                    )?,
                }))
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn generate_messages_types(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
    concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
) -> Result<Vec<program_abi::MessageType>, ErrorEmitted> {
    // Generate the JSON data for the messages types
    ctx.program
        .messages_types
        .iter()
        .map(|(message_id, type_id)| {
            Ok(program_abi::MessageType {
                message_id: (**message_id as u64).to_string(),
                concrete_type_id: generate_concrete_type_declaration(
                    handler,
                    ctx,
                    engines,
                    metadata_types,
                    concrete_types,
                    *type_id,
                    *type_id,
                )?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn generate_configurables(
    handler: &Handler,
    ctx: &mut AbiContext,
    engines: &Engines,
    metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
    concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
) -> Result<Vec<program_abi::Configurable>, ErrorEmitted> {
    // Generate the JSON data for the configurables types
    ctx.program
        .configurables
        .iter()
        .map(|decl| {
            Ok(program_abi::Configurable {
                name: decl.call_path.suffix.to_string(),
                concrete_type_id: generate_concrete_type_declaration(
                    handler,
                    ctx,
                    engines,
                    metadata_types,
                    concrete_types,
                    decl.type_ascription.type_id(),
                    decl.type_ascription.type_id(),
                )?,
                offset: 0,
                indirect: false,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn generate_error_codes(panic_occurrences: &PanicOccurrences) -> BTreeMap<u64, ErrorDetails> {
    panic_occurrences
        .iter()
        .map(|(panic_occurrence, &panic_error_code)| {
            (
                panic_error_code,
                ErrorDetails {
                    pos: ErrorPosition {
                        function: panic_occurrence.function.clone(),
                        pkg: panic_occurrence.loc.pkg.clone(),
                        file: panic_occurrence.loc.file.clone(),
                        line: panic_occurrence.loc.loc.line as u64,
                        column: panic_occurrence.loc.loc.col as u64,
                    },
                    log_id: panic_occurrence
                        .log_id
                        .map(|log_id| log_id.hash_id.to_string()),
                    msg: panic_occurrence.msg.clone(),
                },
            )
        })
        .collect()
}

fn generate_panicking_calls(
    panicking_call_occurrences: &PanickingCallOccurrences,
) -> BTreeMap<u64, PanickingCall> {
    panicking_call_occurrences
        .iter()
        .map(|(panicking_call_occurrence, panicking_call_id)| {
            (
                *panicking_call_id,
                PanickingCall {
                    pos: ErrorPosition {
                        function: panicking_call_occurrence.caller_function.clone(),
                        pkg: panicking_call_occurrence.loc.pkg.clone(),
                        file: panicking_call_occurrence.loc.file.clone(),
                        line: panicking_call_occurrence.loc.loc.line as u64,
                        column: panicking_call_occurrence.loc.loc.col as u64,
                    },
                    function: panicking_call_occurrence.function.clone(),
                },
            )
        })
        .collect()
}

impl TypeId {
    /// Return the type parameters of a given (potentially generic) type while considering what it
    /// actually resolves to. These parameters are essentially of type of `usize` which are
    /// basically the IDs of some set of `program_abi::TypeMetadataDeclaration`s. The method below also
    /// updates the provide list of `program_abi::TypeMetadataDeclaration`s  to add the newly discovered
    /// types.
    #[allow(clippy::too_many_arguments)]
    pub(self) fn get_abi_type_parameters(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
        concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
        resolved_type_id: TypeId,
        metadata_types_to_add: &mut Vec<program_abi::TypeMetadataDeclaration>,
    ) -> Result<Option<Vec<MetadataTypeId>>, ErrorEmitted> {
        match self.is_generic_parameter(engines, resolved_type_id) {
            true => Ok(None),
            false => resolved_type_id
                .get_type_parameters(engines)
                .map(|v| {
                    v.iter()
                        .map(|p| {
                            let p = p
                                .as_type_parameter()
                                .expect("only works with type parameters");
                            p.get_abi_type_parameter(
                                handler,
                                ctx,
                                engines,
                                metadata_types,
                                concrete_types,
                                metadata_types_to_add,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .map_or(Ok(None), |v| v.map(Some)),
        }
    }

    /// Return the components of a given (potentially generic) type while considering what it
    /// actually resolves to. These components are essentially of type of
    /// `program_abi::TypeApplication`.  The method below also updates the provided list of
    /// `program_abi::TypeMetadataDeclaration`s  to add the newly discovered types.
    #[allow(clippy::too_many_arguments)]
    pub(self) fn get_abi_type_components(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
        concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
        resolved_type_id: TypeId,
        metadata_types_to_add: &mut Vec<program_abi::TypeMetadataDeclaration>,
    ) -> Result<Option<Vec<program_abi::TypeApplication>>, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        Ok(match &*type_engine.get(*self) {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);

                let mut new_metadata_types_to_add =
                    Vec::<program_abi::TypeMetadataDeclaration>::new();
                for x in decl.variants.iter() {
                    generate_type_metadata_declaration(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                        x.type_argument.initial_type_id(),
                        x.type_argument.type_id(),
                        &mut new_metadata_types_to_add,
                    )?;
                }

                // Generate the JSON data for the enum. This is basically a list of
                // `program_abi::TypeApplication`s
                let components = decl
                    .variants
                    .iter()
                    .map(|x| {
                        Ok(program_abi::TypeApplication {
                            name: x.name.to_string(),
                            type_id: program_abi::TypeId::Metadata(MetadataTypeId(
                                x.type_argument.initial_type_id().index(),
                            )),
                            error_message: x.attributes.error_message().cloned(),
                            type_arguments: x
                                .type_argument
                                .initial_type_id()
                                .get_abi_type_arguments(
                                    handler,
                                    ctx,
                                    engines,
                                    metadata_types,
                                    concrete_types,
                                    x.type_argument.type_id(),
                                    &mut new_metadata_types_to_add,
                                )?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                if components.is_empty() {
                    None
                } else {
                    metadata_types_to_add.extend(new_metadata_types_to_add);
                    Some(components)
                }
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);

                let mut new_metadata_types_to_add =
                    Vec::<program_abi::TypeMetadataDeclaration>::new();
                for x in decl.fields.iter() {
                    generate_type_metadata_declaration(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                        x.type_argument.initial_type_id(),
                        x.type_argument.type_id(),
                        &mut new_metadata_types_to_add,
                    )?;
                }

                // Generate the JSON data for the struct. This is basically a list of
                // `program_abi::TypeApplication`s
                let components = decl
                    .fields
                    .iter()
                    .map(|x| {
                        Ok(program_abi::TypeApplication {
                            name: x.name.to_string(),
                            type_id: program_abi::TypeId::Metadata(MetadataTypeId(
                                x.type_argument.initial_type_id().index(),
                            )),
                            error_message: None,
                            type_arguments: x
                                .type_argument
                                .initial_type_id()
                                .get_abi_type_arguments(
                                    handler,
                                    ctx,
                                    engines,
                                    metadata_types,
                                    concrete_types,
                                    x.type_argument.type_id(),
                                    &mut new_metadata_types_to_add,
                                )?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                if components.is_empty() {
                    None
                } else {
                    metadata_types_to_add.extend(new_metadata_types_to_add);
                    Some(components)
                }
            }
            TypeInfo::Array(..) => {
                if let TypeInfo::Array(elem_ty, _) = &*type_engine.get(resolved_type_id) {
                    generate_type_metadata_declaration(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                        elem_ty.initial_type_id(),
                        elem_ty.type_id(),
                        metadata_types_to_add,
                    )?;

                    // Generate the JSON data for the array. This is basically a single
                    // `program_abi::TypeApplication` for the array element type
                    Some(vec![program_abi::TypeApplication {
                        name: "__array_element".to_string(),
                        type_id: program_abi::TypeId::Metadata(MetadataTypeId(
                            elem_ty.initial_type_id().index(),
                        )),
                        error_message: None,
                        type_arguments: elem_ty.initial_type_id().get_abi_type_arguments(
                            handler,
                            ctx,
                            engines,
                            metadata_types,
                            concrete_types,
                            elem_ty.type_id(),
                            metadata_types_to_add,
                        )?,
                    }])
                } else {
                    unreachable!();
                }
            }
            TypeInfo::Slice(..) => {
                if let TypeInfo::Slice(elem_ty) = &*type_engine.get(resolved_type_id) {
                    generate_type_metadata_declaration(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                        elem_ty.initial_type_id(),
                        elem_ty.type_id(),
                        metadata_types_to_add,
                    )?;

                    // Generate the JSON data for the array. This is basically a single
                    // `program_abi::TypeApplication` for the array element type
                    Some(vec![program_abi::TypeApplication {
                        name: "__slice_element".to_string(),
                        type_id: program_abi::TypeId::Metadata(MetadataTypeId(
                            elem_ty.initial_type_id().index(),
                        )),
                        error_message: None,
                        type_arguments: elem_ty.initial_type_id().get_abi_type_arguments(
                            handler,
                            ctx,
                            engines,
                            metadata_types,
                            concrete_types,
                            elem_ty.type_id(),
                            metadata_types_to_add,
                        )?,
                    }])
                } else {
                    unreachable!();
                }
            }
            TypeInfo::Tuple(_) => {
                if let TypeInfo::Tuple(fields) = &*type_engine.get(resolved_type_id) {
                    let mut new_metadata_types_to_add =
                        Vec::<program_abi::TypeMetadataDeclaration>::new();
                    for x in fields.iter() {
                        generate_type_metadata_declaration(
                            handler,
                            ctx,
                            engines,
                            metadata_types,
                            concrete_types,
                            x.initial_type_id(),
                            x.type_id(),
                            &mut new_metadata_types_to_add,
                        )?;
                    }

                    // Generate the JSON data for the tuple. This is basically a list of
                    // `program_abi::TypeApplication`s
                    let components = fields
                        .iter()
                        .map(|x| {
                            Ok(program_abi::TypeApplication {
                                name: "__tuple_element".to_string(),
                                type_id: program_abi::TypeId::Metadata(MetadataTypeId(
                                    x.initial_type_id().index(),
                                )),
                                error_message: None,
                                type_arguments: x.initial_type_id().get_abi_type_arguments(
                                    handler,
                                    ctx,
                                    engines,
                                    metadata_types,
                                    concrete_types,
                                    x.type_id(),
                                    metadata_types_to_add,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if components.is_empty() {
                        None
                    } else {
                        metadata_types_to_add.extend(new_metadata_types_to_add);
                        Some(components)
                    }
                } else {
                    unreachable!()
                }
            }
            TypeInfo::Custom { type_arguments, .. } => {
                if !self.is_generic_parameter(engines, resolved_type_id) {
                    for (v, p) in type_arguments.clone().unwrap_or_default().iter().zip(
                        resolved_type_id
                            .get_type_parameters(engines)
                            .unwrap_or_default()
                            .iter(),
                    ) {
                        let p = p
                            .as_type_parameter()
                            .expect("only works with type parameters");
                        generate_type_metadata_declaration(
                            handler,
                            ctx,
                            engines,
                            metadata_types,
                            concrete_types,
                            v.initial_type_id(),
                            p.type_id,
                            metadata_types_to_add,
                        )?;
                    }
                    resolved_type_id.get_abi_type_components(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                        resolved_type_id,
                        metadata_types_to_add,
                    )?
                } else {
                    None
                }
            }
            TypeInfo::Alias { .. } => {
                if let TypeInfo::Alias { ty, .. } = &*type_engine.get(resolved_type_id) {
                    ty.initial_type_id().get_abi_type_components(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                        ty.type_id(),
                        metadata_types_to_add,
                    )?
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
                        metadata_types,
                        concrete_types,
                        resolved_type_id,
                        metadata_types_to_add,
                    )?
                }
            }
            _ => None,
        })
    }

    /// Return the type arguments of a given (potentially generic) type while considering what it
    /// actually resolves to. These arguments are essentially of type of
    /// `program_abi::TypeApplication`. The method below also updates the provided list of
    /// `program_abi::TypeMetadataDeclaration`s  to add the newly discovered types.
    #[allow(clippy::too_many_arguments)]
    pub(self) fn get_abi_type_arguments(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
        concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
        resolved_type_id: TypeId,
        metadata_types_to_add: &mut Vec<program_abi::TypeMetadataDeclaration>,
    ) -> Result<Option<Vec<program_abi::TypeApplication>>, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let resolved_params = resolved_type_id.get_type_parameters(engines);
        handler.scope(|handler| {
            Ok(match &*type_engine.get(*self) {
                TypeInfo::Custom {
                    type_arguments: Some(type_arguments),
                    ..
                } => (!type_arguments.is_empty()).then_some({
                    let resolved_params = resolved_params.unwrap_or_default();
                    for (v, p) in type_arguments.iter().zip(resolved_params.iter()) {
                        let p = p
                            .as_type_parameter()
                            .expect("only works with type parameters");
                        let _ = generate_type_metadata_declaration(
                            handler,
                            ctx,
                            engines,
                            metadata_types,
                            concrete_types,
                            v.type_id(),
                            p.type_id,
                            metadata_types_to_add,
                        );
                    }

                    type_arguments
                        .iter()
                        .zip(resolved_params.iter())
                        .map(|(arg, p)| {
                            let p = p
                                .as_type_parameter()
                                .expect("only works with type parameters");
                            Ok(program_abi::TypeApplication {
                                name: "".to_string(),
                                type_id: program_abi::TypeId::Metadata(MetadataTypeId(
                                    arg.initial_type_id().index(),
                                )),
                                error_message: None,
                                type_arguments: arg.initial_type_id().get_abi_type_arguments(
                                    handler,
                                    ctx,
                                    engines,
                                    metadata_types,
                                    concrete_types,
                                    p.type_id,
                                    metadata_types_to_add,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?
                }),
                TypeInfo::Enum(decl_ref) => {
                    let decl = decl_engine.get_enum(decl_ref);

                    let mut new_metadata_types_to_add =
                        Vec::<program_abi::TypeMetadataDeclaration>::new();
                    for p in decl.generic_parameters.iter() {
                        let p = p
                            .as_type_parameter()
                            .expect("only works with type parameters");
                        generate_type_metadata_declaration(
                            handler,
                            ctx,
                            engines,
                            metadata_types,
                            concrete_types,
                            p.type_id,
                            p.type_id,
                            &mut new_metadata_types_to_add,
                        )?;
                    }

                    let type_arguments = decl
                        .generic_parameters
                        .iter()
                        .map(|p| {
                            let p = p
                                .as_type_parameter()
                                .expect("only works with type parameters");
                            Ok(program_abi::TypeApplication {
                                name: "".to_string(),
                                type_id: program_abi::TypeId::Metadata(MetadataTypeId(
                                    p.type_id.index(),
                                )),
                                error_message: None,
                                type_arguments: p.type_id.get_abi_type_arguments(
                                    handler,
                                    ctx,
                                    engines,
                                    metadata_types,
                                    concrete_types,
                                    p.type_id,
                                    &mut new_metadata_types_to_add,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    if type_arguments.is_empty() {
                        None
                    } else {
                        metadata_types_to_add.extend(new_metadata_types_to_add);
                        Some(type_arguments)
                    }
                }

                TypeInfo::Struct(decl_ref) => {
                    let decl = decl_engine.get_struct(decl_ref);

                    let mut new_metadata_types_to_add =
                        Vec::<program_abi::TypeMetadataDeclaration>::new();
                    for p in decl.generic_parameters.iter() {
                        let p = p
                            .as_type_parameter()
                            .expect("only works with type parameters");
                        generate_type_metadata_declaration(
                            handler,
                            ctx,
                            engines,
                            metadata_types,
                            concrete_types,
                            p.type_id,
                            p.type_id,
                            &mut new_metadata_types_to_add,
                        )?;
                    }

                    let type_arguments = decl
                        .generic_parameters
                        .iter()
                        .map(|p| {
                            let p = p
                                .as_type_parameter()
                                .expect("only works with type parameters");
                            Ok(program_abi::TypeApplication {
                                name: "".to_string(),
                                type_id: program_abi::TypeId::Metadata(MetadataTypeId(
                                    p.type_id.index(),
                                )),
                                error_message: None,
                                type_arguments: p.type_id.get_abi_type_arguments(
                                    handler,
                                    ctx,
                                    engines,
                                    metadata_types,
                                    concrete_types,
                                    p.type_id,
                                    &mut new_metadata_types_to_add,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    if type_arguments.is_empty() {
                        None
                    } else {
                        metadata_types_to_add.extend(new_metadata_types_to_add);
                        Some(type_arguments)
                    }
                }
                _ => None,
            })
        })
    }

    /// Return the type arguments of a given (potentially generic) type while considering what it
    /// actually resolves to. These arguments are essentially of type of
    /// `program_abi::TypeApplication`. The method below also updates the provided list of
    /// `program_abi::TypeMetadataDeclaration`s  to add the newly discovered types.
    pub(self) fn get_abi_type_arguments_as_concrete_type_ids(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
        concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
        resolved_type_id: TypeId,
    ) -> Result<Option<Vec<program_abi::ConcreteTypeId>>, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let resolved_params = resolved_type_id.get_type_parameters(engines);
        Ok(match &*type_engine.get(*self) {
            TypeInfo::Custom {
                type_arguments: Some(type_arguments),
                ..
            } => (!type_arguments.is_empty()).then_some({
                let resolved_params = resolved_params.unwrap_or_default();
                type_arguments
                    .iter()
                    .zip(resolved_params.iter())
                    .map(|(arg, p)| {
                        let p = p
                            .as_type_parameter()
                            .expect("only works with type parameters");
                        generate_concrete_type_declaration(
                            handler,
                            ctx,
                            engines,
                            metadata_types,
                            concrete_types,
                            arg.initial_type_id(),
                            p.type_id,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }),
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                Some(
                    decl.generic_parameters
                        .iter()
                        .map(|p| {
                            let p = p
                                .as_type_parameter()
                                .expect("only works with type parameters");
                            generate_concrete_type_declaration(
                                handler,
                                ctx,
                                engines,
                                metadata_types,
                                concrete_types,
                                p.type_id,
                                p.type_id,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                Some(
                    decl.generic_parameters
                        .iter()
                        .map(|p| {
                            let p = p
                                .as_type_parameter()
                                .expect("only works with type parameters");
                            generate_concrete_type_declaration(
                                handler,
                                ctx,
                                engines,
                                metadata_types,
                                concrete_types,
                                p.type_id,
                                p.type_id,
                            )
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
        metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
        concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
    ) -> Result<program_abi::ABIFunction, ErrorEmitted> {
        let inputs = handler.scope(|handler| {
            self.parameters
                .iter()
                .map(|param| {
                    generate_concrete_type_declaration(
                        handler,
                        ctx,
                        engines,
                        metadata_types,
                        concrete_types,
                        param.type_argument.initial_type_id(),
                        param.type_argument.type_id(),
                    )
                    .map(|concrete_type_id| {
                        program_abi::TypeConcreteParameter {
                            name: param.name.to_string(),
                            concrete_type_id,
                        }
                    })
                })
                .collect::<Result<Vec<_>, ErrorEmitted>>()
        })?;

        // Generate the JSON data for the function
        Ok(program_abi::ABIFunction {
            name: self.name.as_str().to_string(),
            inputs,
            output: generate_concrete_type_declaration(
                handler,
                ctx,
                engines,
                metadata_types,
                concrete_types,
                self.return_type.initial_type_id(),
                self.return_type.type_id(),
            )?,
            attributes: generate_attributes(&self.attributes),
        })
    }
}

fn generate_attributes(attributes: &Attributes) -> Option<Vec<program_abi::Attribute>> {
    if attributes.is_empty() {
        None
    } else {
        Some(
            attributes
                .all()
                .map(|attr| program_abi::Attribute {
                    name: attr.name.to_string(),
                    arguments: attr.args.iter().map(|arg| arg.name.to_string()).collect(),
                })
                .collect(),
        )
    }
}

impl GenericTypeParameter {
    /// Returns the initial type ID of a TypeParameter. Also updates the provided list of types to
    /// append the current TypeParameter as a `program_abi::TypeMetadataDeclaration`.
    pub(self) fn get_abi_type_parameter(
        &self,
        handler: &Handler,
        ctx: &mut AbiContext,
        engines: &Engines,
        metadata_types: &mut Vec<program_abi::TypeMetadataDeclaration>,
        concrete_types: &mut Vec<program_abi::TypeConcreteDeclaration>,
        metadata_types_to_add: &mut Vec<program_abi::TypeMetadataDeclaration>,
    ) -> Result<MetadataTypeId, ErrorEmitted> {
        let type_id = MetadataTypeId(self.initial_type_id.index());
        let type_parameter = program_abi::TypeMetadataDeclaration {
            metadata_type_id: type_id.clone(),
            type_field: self.initial_type_id.get_abi_type_str(
                handler,
                &ctx.to_str_context(),
                engines,
                self.type_id,
            )?,
            components: self.initial_type_id.get_abi_type_components(
                handler,
                ctx,
                engines,
                metadata_types,
                concrete_types,
                self.type_id,
                metadata_types_to_add,
            )?,
            type_parameters: None,
        };
        metadata_types_to_add.push(type_parameter);
        Ok(type_id)
    }
}
