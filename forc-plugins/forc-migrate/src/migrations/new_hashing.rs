#![allow(dead_code)]

use std::collections::HashSet;

use super::{MigrationStep, MigrationStepKind};
use crate::{
    migrations::{DryRun, Occurrence, ProgramInfo},
    visiting::{
        InvalidateTypedElement, LexedFnCallInfo, ProgramVisitor, TreesVisitor, TyFnCallInfo,
        VisitingContext,
    },
};
use anyhow::{Ok, Result};
use sway_ast::{Expr, StorageField, Ty};
use sway_core::{
    language::{
        ty::{TyExpression, TyStorageField, TyStructDecl},
        CallPath,
    },
    Engines, GenericArgument, TypeId, TypeInfo, TypeParameter,
};
use sway_error::formatting::{plural_s, sequence_to_str, Enclosing};
use sway_types::Spanned;

pub(super) const REVIEW_EXISTING_USAGES_OF_STORAGE_MAP_SHA256_AND_KECCAK256: MigrationStep = MigrationStep {
    title: "Review existing usages of `StorageMap`, `sha256`, and `keccak256`",
    duration: 10,
    kind: MigrationStepKind::Instruction(review_existing_usages_of_storage_map_sha256_and_keccak256),
    help: &[
        "New hashing changes the hashes of instances of the following types:",
        "  - string slices (`str`)",
        "  - string arrays (`str[N]`)",
        "  - arrays (`[T; N]`)",
        "  - raw slices (`raw_slice`)",
        "  - vectors (`std::vec::Vec<T>`)",
        "  - bytes (`std::bytes::Bytes`)",
        " ",
        "To decide if opting-in to new hashing is backward-compatible and safe or not,",
        "review if those types are directly used, or are contained in types:",
        "  - used as keys in `StorageMap`s,",
        "  - used in custom storage types,",
        "  - hashed using `sha256` or `keccak256` functions.",
        " ",
        "╔═════════════════════════════════════════════════════════════════════════════════════╗",
        "║ The above occurrences must not be seen as comprehensive, but rather as a guideline. ║",
        "║ Carefully review all the storage access and hashing patterns in your code.          ║",
        "║ E.g., using precomputed hashes, having custom `Hash` implementations, and similar.  ║",
        "╚═════════════════════════════════════════════════════════════════════════════════════╝",
    ],
};

// NOTE: When analyzing storage fields, we expect that the storage types are never nested
//       inside of non-storage types.
//       E.g., we don't expect to have a storage fields like these:
//         field_a: (u8, u8, StorageMap<...>) = (1, 2, StorageMap {}),
//         field_b: SomeNonStorageTypeStruct<StorageMap<...>> = SomeNonStorageTypeStruct { field: StorageMap {} },

fn review_existing_usages_of_storage_map_sha256_and_keccak256(
    program_info: &ProgramInfo,
) -> Result<Vec<Occurrence>> {
    struct Visitor {
        storage_map_path: CallPath,
        storage_vec_path: CallPath,
        non_affected_storage_types_paths: HashSet<CallPath>,
        hash_functions_paths: HashSet<CallPath>,
        hash_functions_names: HashSet<&'static str>,
        affected_std_structs: HashSet<CallPath>,
        non_affected_std_structs: HashSet<CallPath>,
        built_in_type_names: HashSet<&'static str>,
    }

    impl Visitor {
        fn new() -> Self {
            Self {
                storage_map_path: CallPath::fullpath(&[
                    "std",
                    "storage",
                    "storage_map",
                    "StorageMap",
                ]),
                storage_vec_path: CallPath::fullpath(&[
                    "std",
                    "storage",
                    "storage_vec",
                    "StorageVec",
                ]),
                non_affected_storage_types_paths: HashSet::from_iter(
                    vec![
                        ["std", "storage", "storage_bytes", "StorageBytes"],
                        ["std", "storage", "storage_string", "StorageString"],
                    ]
                    .into_iter()
                    .map(|path_parts| CallPath::fullpath(&path_parts)),
                ),
                hash_functions_paths: HashSet::from_iter(
                    vec![["std", "hash", "sha256"], ["std", "hash", "keccak256"]]
                        .into_iter()
                        .map(|path_parts| CallPath::fullpath(&path_parts)),
                ),
                hash_functions_names: HashSet::from_iter(vec!["sha256", "keccak256"]),
                affected_std_structs: HashSet::from_iter(
                    vec![["std", "vec", "Vec"], ["std", "bytes", "Bytes"]]
                        .into_iter()
                        .map(|path_parts| CallPath::fullpath(&path_parts)),
                ),
                non_affected_std_structs: HashSet::from_iter(
                    vec![
                        ["std", "crypto", "secp256k1", "Secp256k1"],
                        ["std", "crypto", "secp256r1", "Secp256r1"],
                        ["std", "crypto", "message", "Message"],
                        ["std", "crypto", "public_key", "PublicKey"],
                    ]
                    .into_iter()
                    .map(|path_parts| CallPath::fullpath(&path_parts))
                    .chain(vec![CallPath::fullpath(&["std", "b512", "B512"])]),
                ),
                built_in_type_names: HashSet::from_iter(vec![
                    "()", "!", "bool", "u8", "u16", "u32", "u64", "u256", "b256",
                ]),
            }
        }

        fn is_known_storage_type(&self, call_path: &CallPath) -> bool {
            self.non_affected_storage_types_paths.contains(call_path)
                || self.storage_map_path == *call_path
                || self.storage_vec_path == *call_path
        }

        /// Returns the (affected type name, help message) if the type given by `type_id` is affected by new hashing.
        /// The affected type name is the name of the type that is actually affected by new hashing.
        /// It doesn't have to be the same as the type name given by `type_id`.
        /// E.g., if `type_id` represents a `SomeStruct<str>`, the affected type name will be `str`.
        fn is_affected_type(&self, engines: &Engines, type_id: TypeId) -> Option<(String, String)> {
            fn review_type() -> Option<(String, String)> {
                Some((
                    "{unknown}".into(),
                    "Review the type of this expression.".into(),
                ))
            }

            fn review_generic_type(type_name: &str) -> Option<(String, String)> {
                Some((type_name.into(), format!("This has generic type \"{type_name}\". Review all the concrete types used with it.")))
            }

            fn review_affected_type(
                original_type_name: &str,
                type_name: &str,
                depth: usize,
            ) -> Option<(String, String)> {
                Some((type_name.into(),
                    match depth {
                        0 => format!("This has type \"{original_type_name}\"."),
                        _ => format!("This has type \"{original_type_name}\", that {}contains \"{type_name}\".",
                            if depth > 1 {
                                "recursively "
                            } else {
                                ""
                            }
                        ),
                }))
            }

            fn is_affected_type_impl(
                visitor: &Visitor,
                engines: &Engines,
                original_type_name: &str,
                type_id: TypeId,
                depth: usize,
            ) -> Option<(String, String)> {
                match &*engines.te().get(type_id) {
                    // Types not affected by new hashing.
                    TypeInfo::Never
                    | TypeInfo::UnsignedInteger(_)
                    | TypeInfo::ContractCaller { .. }
                    | TypeInfo::Boolean
                    | TypeInfo::B256
                    | TypeInfo::Numeric
                    | TypeInfo::Contract
                    | TypeInfo::RawUntypedPtr => None,

                    // Generic types.
                    TypeInfo::UnknownGeneric { .. } => review_generic_type(&engines.help_out(type_id).to_string()),

                    // Types that will not occur in a typed program compiled without any errors.
                    // Types like `Unknown` or `ErrorRecovery` will never appear in a
                    // typed program compiled without any errors. Still, we handle
                    // all of them here with the `review_the_type` message, to be on the safe side.
                    TypeInfo::Unknown
                    | TypeInfo::Placeholder(_)
                    | TypeInfo::TypeParam(_)
                    | TypeInfo::UntypedEnum(_)
                    | TypeInfo::UntypedStruct(_)
                    | TypeInfo::Custom { .. }
                    | TypeInfo::ErrorRecovery(_) => review_type(),

                    // Types that are directly affected by new hashing.
                    TypeInfo::StringSlice
                    | TypeInfo::StringArray(_)
                    | TypeInfo::Array(_, _)
                    | TypeInfo::RawUntypedSlice => review_affected_type(original_type_name, &engines.help_out(type_id).to_string(), depth),

                    // Aggregate types that might be directly or indirectly affected by new hashing.
                    TypeInfo::Enum(decl_id) => {
                        let enum_decl = engines.de().get_enum(decl_id);
                        for variant in enum_decl.variants.iter() {
                            if let GenericArgument::Type(ta) = &variant.type_argument {
                                if let Some(is_affected) = is_affected_type_impl(visitor, engines, original_type_name, ta.type_id, depth + 1) {
                                    return Some(is_affected);
                                }
                            }
                        }
                        None
                    },
                    TypeInfo::Struct(decl_id) => {
                        let struct_decl = engines.de().get_struct(decl_id);
                        if visitor.non_affected_std_structs.contains(&struct_decl.call_path) {
                            None
                        } else if visitor.affected_std_structs.contains(&struct_decl.call_path) {
                            review_affected_type(original_type_name, &engines.help_out(type_id).to_string(), depth)
                        } else {
                            for field in struct_decl.fields.iter() {
                                if let GenericArgument::Type(ta) = &field.type_argument {
                                    if let Some(is_affected) = is_affected_type_impl(visitor, engines, original_type_name, ta.type_id, depth + 1) {
                                        return Some(is_affected);
                                    }
                                }
                            }
                            None
                        }
                    },
                    TypeInfo::Tuple(generic_arguments) => {
                        for generic_argument in generic_arguments.iter() {
                            if let GenericArgument::Type(ta) = generic_argument {
                                if let Some(is_affected) = is_affected_type_impl(visitor, engines, original_type_name, ta.type_id, depth + 1) {
                                    return Some(is_affected);
                                }
                            }
                        }
                        None
                    },

                    // Types with generic arguments that might be indirectly affected by new hashing.
                    TypeInfo::Ptr(generic_argument)
                    // Typed slices are still not a fully implemented and official feature.
                    // We don't have a `Hash` implementation for them yet, so they are not affected by new hashing.
                    // Still, we will handle the type itself, to be on the safe side.
                    | TypeInfo::Slice(generic_argument)
                    | TypeInfo::Alias { ty: generic_argument, .. }
                    | TypeInfo::Ref { referenced_type: generic_argument, .. } => match generic_argument {
                        GenericArgument::Type(ta) => is_affected_type_impl(visitor, engines, original_type_name, ta.type_id, depth + 1),
                        GenericArgument::Const(_) => None,
                    },

                    // Trait type.
                    TypeInfo::TraitType { implemented_in, .. } => is_affected_type_impl(visitor, engines, original_type_name, *implemented_in, depth + 1),
                }
            }

            let original_type_name = engines.help_out(type_id).to_string();
            is_affected_type_impl(self, engines, &original_type_name, type_id, 0)
        }

        /// Returns a help message if the storage field type `type_id` might be affected by new hashing, or `None` if it is not.
        fn is_affected_storage_field_type(
            &self,
            engines: &Engines,
            type_id: TypeId,
        ) -> Option<String> {
            /// Describes why a storage field is affected by new hashing.
            #[derive(Default)]
            struct AffectedStorageField {
                /// Types of keys of a one or more nested `StorageMap`s that are affected by new hashing.
                /// E.g., `["str[3]", "[u64; 3]"`.
                /// The types are ordered left to right, in order of appearance in the storage field type declaration.
                affected_storage_map_keys: Vec<String>,
                /// Types that appear in the storage field type declaration and that could be unknown storage types.
                potential_storage_types: Vec<String>,
                // Represents situations that should never happen in a typed program compiled without any errors.
                // E.g., `StorageMap` must have exactly two type parameters. If not, this is an unexpected error.
                // We handle such errors with a message to review the storage field, to be on the safe side.
                unexpected_error: bool,
            }

            impl AffectedStorageField {
                /// Returns a help message if the storage field is affected by new hashing, or `None` if it is not.
                fn help_message(&self) -> Option<String> {
                    if self.affected_storage_map_keys.is_empty()
                        && self.potential_storage_types.is_empty()
                        && !self.unexpected_error
                    {
                        return None;
                    }

                    let message = if self.unexpected_error {
                        "Review this storage field.".into()
                    } else {
                        format!(
                            "Review this storage field, because of {}{}{}.",
                            if self.affected_storage_map_keys.is_empty() {
                                "".to_string()
                            } else {
                                format!(
                                    "{} in \"StorageMap\" key{}",
                                    sequence_to_str(
                                        &self.affected_storage_map_keys,
                                        Enclosing::DoubleQuote,
                                        usize::MAX
                                    ),
                                    plural_s(self.affected_storage_map_keys.len()),
                                )
                            },
                            if !(self.potential_storage_types.is_empty()
                                || self.affected_storage_map_keys.is_empty())
                            {
                                " and "
                            } else {
                                ""
                            },
                            if self.potential_storage_types.is_empty() {
                                "".to_string()
                            } else {
                                format!(
                                    "potential custom storage type{} {}",
                                    plural_s(self.potential_storage_types.len()),
                                    sequence_to_str(
                                        &self.potential_storage_types,
                                        Enclosing::DoubleQuote,
                                        usize::MAX
                                    ),
                                )
                            },
                        )
                    };

                    Some(message)
                }
            }

            fn is_affected_storage_field_type_impl(
                visitor: &Visitor,
                engines: &Engines,
                type_id: TypeId,
                affected_storage_field: &mut AffectedStorageField,
            ) {
                fn get_generic_parameter_type_id(type_parameter: &TypeParameter) -> Option<TypeId> {
                    match type_parameter {
                        TypeParameter::Type(ty) => Some(ty.type_id),
                        TypeParameter::Const(_) => None,
                    }
                }

                // We assume that:
                //  - only structs can be storage types,
                //  - only storage types can contain other storage types.
                // For each category of storage types, we have a visitor function
                // named `try_visit_***`, that returns `true` if the type is a storage type of that category
                // and was visited.

                fn try_visit_non_affected_known_storage_type(
                    visitor: &Visitor,
                    struct_decl: &TyStructDecl,
                ) -> bool {
                    visitor
                        .non_affected_storage_types_paths
                        .contains(&struct_decl.call_path)
                }

                fn try_visit_storage_vec(
                    visitor: &Visitor,
                    engines: &Engines,
                    struct_decl: &TyStructDecl,
                    affected_storage_field: &mut AffectedStorageField,
                ) -> bool {
                    if visitor.storage_vec_path != struct_decl.call_path {
                        return false;
                    }

                    if struct_decl.generic_parameters.len() != 1 {
                        affected_storage_field.unexpected_error = true;
                        return true;
                    }

                    let element_type_id =
                        get_generic_parameter_type_id(&struct_decl.generic_parameters[0]);
                    if element_type_id.is_none() {
                        affected_storage_field.unexpected_error = true;
                        return true;
                    }

                    let element_type_id = element_type_id.unwrap();

                    is_affected_storage_field_type_impl(
                        visitor,
                        engines,
                        element_type_id,
                        affected_storage_field,
                    );

                    true
                }

                fn try_visit_unknown_potential_storage_type(
                    visitor: &Visitor,
                    engines: &Engines,
                    struct_decl: &TyStructDecl,
                    struct_name: &str,
                    affected_storage_field: &mut AffectedStorageField,
                ) -> bool {
                    if visitor.is_known_storage_type(&struct_decl.call_path) {
                        return false;
                    }

                    // Storage types are empty structs.
                    if !struct_decl.fields.is_empty() {
                        return false;
                    }

                    affected_storage_field
                        .potential_storage_types
                        .push(struct_name.to_string());

                    for generic_parameter in struct_decl.generic_parameters.iter() {
                        if let Some(type_id) = get_generic_parameter_type_id(generic_parameter) {
                            is_affected_storage_field_type_impl(
                                visitor,
                                engines,
                                type_id,
                                affected_storage_field,
                            );
                        }
                    }

                    true
                }

                fn try_visit_storage_map(
                    visitor: &Visitor,
                    engines: &Engines,
                    struct_decl: &TyStructDecl,
                    affected_storage_field: &mut AffectedStorageField,
                ) -> bool {
                    if visitor.storage_map_path != struct_decl.call_path {
                        return false;
                    }

                    if struct_decl.generic_parameters.len() != 2 {
                        affected_storage_field.unexpected_error = true;
                        return true;
                    }

                    let key_type_id =
                        get_generic_parameter_type_id(&struct_decl.generic_parameters[0]);
                    let value_type_id =
                        get_generic_parameter_type_id(&struct_decl.generic_parameters[1]);

                    if key_type_id.is_none() || value_type_id.is_none() {
                        affected_storage_field.unexpected_error = true;
                        return true;
                    }

                    let key_type_id = key_type_id.unwrap();
                    let value_type_id = value_type_id.unwrap();

                    // `StorageMap` itself does not implement `Hash`, so it cannot be a key.
                    // So, for the key, we just check if it is affected by new hashing.
                    if let Some((type_name, _msg)) = visitor.is_affected_type(engines, key_type_id)
                    {
                        affected_storage_field
                            .affected_storage_map_keys
                            .push(type_name);
                    }

                    // For the value, we must check if it is a nested `StorageMap`, or a nested storage type.
                    is_affected_storage_field_type_impl(
                        visitor,
                        engines,
                        value_type_id,
                        affected_storage_field,
                    );

                    true
                }

                if let TypeInfo::Struct(struct_decl) = &*engines.te().get_unaliased(type_id) {
                    let struct_decl = engines.de().get_struct(struct_decl);

                    let _ = try_visit_non_affected_known_storage_type(visitor, &struct_decl)
                        || try_visit_storage_vec(
                            visitor,
                            engines,
                            &struct_decl,
                            affected_storage_field,
                        )
                        || try_visit_storage_map(
                            visitor,
                            engines,
                            &struct_decl,
                            affected_storage_field,
                        )
                        || try_visit_unknown_potential_storage_type(
                            visitor,
                            engines,
                            &struct_decl,
                            &engines.help_out(type_id).to_string(),
                            affected_storage_field,
                        );
                    // Otherwise, we have a regular struct that is not a storage type.
                }
            }

            let mut affected_storage_field = AffectedStorageField::default();
            is_affected_storage_field_type_impl(
                self,
                engines,
                type_id,
                &mut affected_storage_field,
            );

            affected_storage_field.help_message()
        }
    }

    impl TreesVisitor<Occurrence> for Visitor {
        fn visit_fn_call(
            &mut self,
            ctx: &VisitingContext,
            lexed_fn_call: &Expr,
            ty_fn_call: Option<&TyExpression>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            let ty_fn_call_info = ty_fn_call
                .map(|ty_fn_call| TyFnCallInfo::new(ctx.engines.de(), ty_fn_call))
                .transpose()?;

            // If we have the typed call info we can check via function decl if
            // it is one of the hash functions, even if an alias is used.
            if let Some(ty_fn_call_info) = ty_fn_call_info {
                if !self
                    .hash_functions_paths
                    .contains(&ty_fn_call_info.fn_decl.call_path)
                {
                    return Ok(InvalidateTypedElement::No);
                }

                let Some((_arg_name, arg_value)) = ty_fn_call_info.arguments.first() else {
                    // This should never happen. There must be exactly one argument to hash functions.
                    // But if it happens, we mark the whole call for review.
                    output.push(Occurrence::new(
                        lexed_fn_call.span(),
                        format!(
                            "Review this \"{}\" call.",
                            ty_fn_call_info.fn_decl.call_path.suffix
                        ),
                    ));
                    return Ok(InvalidateTypedElement::No);
                };

                let Some((_type_name, help_message)) =
                    self.is_affected_type(ctx.engines, arg_value.return_type)
                else {
                    return Ok(InvalidateTypedElement::No);
                };

                // We have found a call to a hash function with an affected type.
                output.push(Occurrence::new(arg_value.span.clone(), help_message));
            } else {
                // If we don't have the typed call info, we can only check the called function name.
                // If it is one of the hash functions, we mark the call for review.
                let lexed_fn_call_info = LexedFnCallInfo::new(lexed_fn_call)?;

                let Expr::Path(path) = lexed_fn_call_info.func else {
                    return Ok(InvalidateTypedElement::No);
                };

                let last_segment = path.last_segment();

                if !self
                    .hash_functions_names
                    .contains(&last_segment.name.as_str())
                {
                    return Ok(InvalidateTypedElement::No);
                }

                output.push(Occurrence::new(
                    lexed_fn_call.span(),
                    format!("Review this \"{}\" call.", last_segment.name.as_str()),
                ));
            }

            Ok(InvalidateTypedElement::No)
        }

        fn visit_storage_field_decl(
            &mut self,
            ctx: &VisitingContext,
            lexed_storage_field: &StorageField,
            ty_storage_field: Option<&TyStorageField>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            if let Some(ty_field_type) = ty_storage_field.and_then(|ty_storage_field| {
                match &ty_storage_field.type_argument {
                    GenericArgument::Type(ty) => Some(ty.type_id),
                    GenericArgument::Const(_) => None,
                }
            }) {
                let Some(help_message) =
                    self.is_affected_storage_field_type(ctx.engines, ty_field_type)
                else {
                    return Ok(InvalidateTypedElement::No);
                };

                // We have found an affected storage field.
                output.push(Occurrence::new(
                    lexed_storage_field.name.span(),
                    help_message,
                ));
            } else {
                match &lexed_storage_field.ty {
                    // We don't expect non-storage types to contain storage types.
                    // Thus, we can ignore tuples and arrays here.
                    Ty::Tuple(_)
                    | Ty::Array(_)
                    // These types cannot contain storage types, or are even not supported
                    // in storage declarations, so we can ignore them as well.
                    | Ty::StringSlice(_)
                    | Ty::StringArray { .. }
                    | Ty::Slice { .. } => {},
                    // These types cannot appear in a program compiled without any errors.
                    // Still, to be on the safe side, we mark them for review.
                    Ty::Infer { .. }
                    | Ty::Ptr { .. }
                    | Ty::Ref { .. }
                    | Ty::Never { .. }
                    | Ty::Expr(_) => {
                        output.push(Occurrence::new(lexed_storage_field.name.span(), "Review this storage field.".to_string()));
                    },
                    // Without the typed storage field, we have to be pessimistic and assume that
                    // the storage field type might be affected by new hashing.
                    // To avoid obvious false positives, we check if the storage field type is a built-in type.
                    Ty::Path(path_type) => {
                        // If it is not a built-in type.
                        if !(path_type.root_opt.is_none() && path_type.suffix.is_empty() &&
                            path_type.prefix.generics_opt.is_none() &&
                            self.built_in_type_names.contains(&path_type.prefix.name.as_str()))
                        {
                            output.push(Occurrence::new(lexed_storage_field.name.span(), "Review this storage field.".to_string()));
                        }
                    },
                }
            }

            Ok(InvalidateTypedElement::No)
        }
    }

    ProgramVisitor::visit_program(program_info, DryRun::Yes, &mut Visitor::new())
}
