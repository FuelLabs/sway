use sway_types::{integer_bits::IntegerBits, Named};

use crate::{
    asm_generation::EvmAbiResult,
    decl_engine::DeclId,
    language::ty::{TyFunctionDecl, TyProgram, TyProgramKind},
    Engines, GenericArgument, TypeId, TypeInfo,
};

pub fn generate_abi_program(program: &TyProgram, engines: &Engines) -> EvmAbiResult {
    match &program.kind {
        TyProgramKind::Contract { abi_entries, .. } => abi_entries
            .iter()
            .map(|x| generate_abi_function(x, engines))
            .collect(),
        TyProgramKind::Script { entry_function, .. }
        | TyProgramKind::Predicate { entry_function, .. } => {
            vec![generate_abi_function(entry_function, engines)]
        }
        _ => vec![],
    }
}

/// Gives back a string that represents the type, considering what it resolves to
fn get_type_str(type_id: &TypeId, engines: &Engines, resolved_type_id: TypeId) -> String {
    let type_engine = engines.te();
    if type_id.is_generic_parameter(engines, resolved_type_id) {
        format!("generic {}", abi_str(&type_engine.get(*type_id), engines))
    } else {
        match (
            &*type_engine.get(*type_id),
            &*type_engine.get(resolved_type_id),
        ) {
            (TypeInfo::Custom { .. }, TypeInfo::Struct { .. }) => {
                format!("struct {}", abi_str(&type_engine.get(*type_id), engines))
            }
            (TypeInfo::Custom { .. }, TypeInfo::Enum { .. }) => {
                format!("enum {}", abi_str(&type_engine.get(*type_id), engines))
            }
            (TypeInfo::Tuple(fields), TypeInfo::Tuple(resolved_fields)) => {
                assert_eq!(fields.len(), resolved_fields.len());
                let field_strs = fields
                    .iter()
                    .map(|_| "_".to_string())
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            (TypeInfo::Array(_, length), TypeInfo::Array(_, resolved_length)) => {
                assert_eq!(
                    length.expr().as_literal_val().unwrap(),
                    resolved_length.expr().as_literal_val().unwrap()
                );
                format!("[_; {:?}]", engines.help_out(length.expr()))
            }
            (TypeInfo::Slice(_), TypeInfo::Slice(_)) => "__slice[_]".into(),
            (TypeInfo::Custom { .. }, _) => {
                format!("generic {}", abi_str(&type_engine.get(*type_id), engines))
            }
            _ => abi_str(&type_engine.get(*type_id), engines),
        }
    }
}

pub fn abi_str(type_info: &TypeInfo, engines: &Engines) -> String {
    use TypeInfo::*;
    let decl_engine = engines.de();
    match type_info {
        Unknown => "unknown".into(),
        Never => "never".into(),
        UnknownGeneric { name, .. } => name.to_string(),
        Placeholder(_) => "_".to_string(),
        TypeParam(param) => format!("typeparam({})", param.name()),
        StringSlice => "str".into(),
        StringArray(length) => format!("str[{:?}]", engines.help_out(length.expr())),
        UnsignedInteger(x) => match x {
            IntegerBits::Eight => "uint8",
            IntegerBits::Sixteen => "uint16",
            IntegerBits::ThirtyTwo => "uint32",
            IntegerBits::SixtyFour => "uint64",
            IntegerBits::V256 => "uint256",
        }
        .into(),
        Boolean => "bool".into(),
        Custom {
            qualified_call_path: call_path,
            ..
        } => call_path.call_path.suffix.to_string(),
        Tuple(fields) => {
            let field_strs = fields
                .iter()
                .map(|field| abi_str_type_arg(field, engines))
                .collect::<Vec<String>>();
            format!("({})", field_strs.join(", "))
        }
        B256 => "uint256".into(),
        Numeric => "u64".into(), // u64 is the default
        Contract => "contract".into(),
        ErrorRecovery(_) => "unknown due to error".into(),
        UntypedEnum(decl_id) => {
            let decl = engines.pe().get_enum(decl_id);
            format!("untyped enum {}", decl.name)
        }
        UntypedStruct(decl_id) => {
            let decl = engines.pe().get_struct(decl_id);
            format!("untyped struct {}", decl.name)
        }
        Enum(decl_ref) => {
            let decl = decl_engine.get_enum(decl_ref);
            format!("enum {}", decl.call_path.suffix)
        }
        Struct(decl_ref) => {
            let decl = decl_engine.get_struct(decl_ref);
            format!("struct {}", decl.call_path.suffix)
        }
        ContractCaller { abi_name, .. } => {
            format!("contract caller {abi_name}")
        }
        Array(elem_ty, length) => {
            format!(
                "{}[{:?}]",
                abi_str_type_arg(elem_ty, engines),
                engines.help_out(length.expr()),
            )
        }
        RawUntypedPtr => "raw untyped ptr".into(),
        RawUntypedSlice => "raw untyped slice".into(),
        Ptr(ty) => {
            format!("__ptr {}", abi_str_type_arg(ty, engines))
        }
        Slice(ty) => {
            format!("__slice {}", abi_str_type_arg(ty, engines))
        }
        Alias { ty, .. } => abi_str_type_arg(ty, engines),
        TraitType {
            name,
            trait_type_id: _,
        } => format!("trait type {}", name),
        Ref {
            to_mutable_value,
            referenced_type,
        } => {
            format!(
                "__ref {}{}", // TODO: (REFERENCES) No references in ABIs according to the RFC. Or we want to have them?
                if *to_mutable_value { "mut " } else { "" },
                abi_str_type_arg(referenced_type, engines)
            )
        }
    }
}

pub fn abi_param_type(type_info: &TypeInfo, engines: &Engines) -> ethabi::ParamType {
    use TypeInfo::*;
    let type_engine = engines.te();
    let decl_engine = engines.de();
    match type_info {
        StringArray(length) => {
            ethabi::ParamType::FixedArray(Box::new(ethabi::ParamType::String), length.expr().as_literal_val().unwrap())
        }
        UnsignedInteger(x) => match x {
            IntegerBits::Eight => ethabi::ParamType::Uint(8),
            IntegerBits::Sixteen => ethabi::ParamType::Uint(16),
            IntegerBits::ThirtyTwo => ethabi::ParamType::Uint(32),
            IntegerBits::SixtyFour => ethabi::ParamType::Uint(64),
            IntegerBits::V256 => ethabi::ParamType::Uint(256),
        },
        Boolean => ethabi::ParamType::Bool,
        B256 => ethabi::ParamType::Uint(256),
        Contract => ethabi::ParamType::Address,
        Enum { .. } => ethabi::ParamType::Uint(8),
        Tuple(fields) => ethabi::ParamType::Tuple(
            fields
                .iter()
                .map(|f| abi_param_type(&type_engine.get(f.type_id()), engines))
                .collect::<Vec<ethabi::ParamType>>(),
        ),
        Struct(decl_ref) => {
            let decl = decl_engine.get_struct(decl_ref);
            ethabi::ParamType::Tuple(
                decl.fields
                    .iter()
                    .map(|f| abi_param_type(&type_engine.get(f.type_argument.type_id()), engines))
                    .collect::<Vec<ethabi::ParamType>>(),
            )
        }
        Array(elem_ty, ..) => ethabi::ParamType::Array(Box::new(abi_param_type(
            &type_engine.get(elem_ty.type_id()),
            engines,
        ))),
        _ => panic!("cannot convert type to Solidity ABI param type: {type_info:?}",),
    }
}

fn generate_abi_function(
    fn_decl_id: &DeclId<TyFunctionDecl>,
    engines: &Engines,
) -> ethabi::operation::Operation {
    let decl_engine = engines.de();
    let fn_decl = decl_engine.get_function(fn_decl_id);
    // A list of all `ethabi::Param`s needed for inputs
    let input_types = fn_decl
        .parameters
        .iter()
        .map(|x| ethabi::Param {
            name: x.name.to_string(),
            kind: ethabi::ParamType::Address,
            internal_type: Some(get_type_str(
                &x.type_argument.type_id(),
                engines,
                x.type_argument.type_id(),
            )),
        })
        .collect::<Vec<_>>();

    // The single `ethabi::Param` needed for the output
    let output_type = ethabi::Param {
        name: String::default(),
        kind: ethabi::ParamType::Address,
        internal_type: Some(get_type_str(
            &fn_decl.return_type.type_id(),
            engines,
            fn_decl.return_type.type_id(),
        )),
    };

    // Generate the ABI data for the function
    #[allow(deprecated)]
    ethabi::operation::Operation::Function(ethabi::Function {
        name: fn_decl.name.as_str().to_string(),
        inputs: input_types,
        outputs: vec![output_type],
        constant: None,
        state_mutability: ethabi::StateMutability::Payable,
    })
}

fn abi_str_type_arg(type_arg: &GenericArgument, engines: &Engines) -> String {
    abi_str(&engines.te().get(type_arg.type_id()), engines)
}
