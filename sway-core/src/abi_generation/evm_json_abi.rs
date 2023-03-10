use sway_types::integer_bits::IntegerBits;

use crate::{
    asm_generation::EvmAbiResult,
    decl_engine::DeclEngine,
    language::ty::{TyFunctionDeclaration, TyProgram, TyProgramKind},
    Engines, TypeArgument, TypeEngine, TypeId, TypeInfo,
};

pub fn generate_json_abi_program(program: &TyProgram, engines: &Engines<'_>) -> EvmAbiResult {
    let type_engine = engines.te();
    let decl_engine = engines.de();
    match &program.kind {
        TyProgramKind::Contract { abi_entries, .. } => abi_entries
            .iter()
            .map(|x| generate_json_abi_function(x, type_engine, decl_engine))
            .collect(),
        TyProgramKind::Script { main_function, .. }
        | TyProgramKind::Predicate { main_function, .. } => {
            vec![generate_json_abi_function(
                main_function,
                type_engine,
                decl_engine,
            )]
        }
        _ => vec![],
    }
}

/// Gives back a string that represents the type, considering what it resolves to
fn get_json_type_str(
    type_id: &TypeId,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    resolved_type_id: TypeId,
) -> String {
    if type_id.is_generic_parameter(type_engine, decl_engine, resolved_type_id) {
        format!(
            "generic {}",
            json_abi_str(&type_engine.get(*type_id), type_engine, decl_engine)
        )
    } else {
        match (type_engine.get(*type_id), type_engine.get(resolved_type_id)) {
            (TypeInfo::Custom { .. }, TypeInfo::Struct { .. }) => {
                format!(
                    "struct {}",
                    json_abi_str(&type_engine.get(*type_id), type_engine, decl_engine)
                )
            }
            (TypeInfo::Custom { .. }, TypeInfo::Enum { .. }) => {
                format!(
                    "enum {}",
                    json_abi_str(&type_engine.get(*type_id), type_engine, decl_engine)
                )
            }
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
                    json_abi_str(&type_engine.get(*type_id), type_engine, decl_engine)
                )
            }
            _ => json_abi_str(&type_engine.get(*type_id), type_engine, decl_engine),
        }
    }
}

pub fn json_abi_str(
    type_info: &TypeInfo,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
) -> String {
    use TypeInfo::*;
    match type_info {
        Unknown => "unknown".into(),
        UnknownGeneric { name, .. } => name.to_string(),
        Placeholder(_) => "_".to_string(),
        TypeParam(n) => format!("typeparam({n})"),
        Str(x) => format!("str[{}]", x.val()),
        UnsignedInteger(x) => match x {
            IntegerBits::Eight => "uint8",
            IntegerBits::Sixteen => "uint16",
            IntegerBits::ThirtyTwo => "uint32",
            IntegerBits::SixtyFour => "uint64",
        }
        .into(),
        Boolean => "bool".into(),
        Custom { call_path, .. } => call_path.suffix.to_string(),
        Tuple(fields) => {
            let field_strs = fields
                .iter()
                .map(|field| json_abi_str_type_arg(field, type_engine, decl_engine))
                .collect::<Vec<String>>();
            format!("({})", field_strs.join(", "))
        }
        SelfType => "Self".into(),
        B256 => "uint256".into(),
        Numeric => "u64".into(), // u64 is the default
        Contract => "contract".into(),
        ErrorRecovery => "unknown due to error".into(),
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
                "{}[{}]",
                json_abi_str_type_arg(elem_ty, type_engine, decl_engine),
                length.val()
            )
        }
        Storage { .. } => "contract storage".into(),
        RawUntypedPtr => "raw untyped ptr".into(),
        RawUntypedSlice => "raw untyped slice".into(),
    }
}

pub fn json_abi_param_type(
    type_info: &TypeInfo,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
) -> ethabi::ParamType {
    use TypeInfo::*;
    match type_info {
        Str(x) => ethabi::ParamType::FixedArray(Box::new(ethabi::ParamType::String), x.val()),
        UnsignedInteger(x) => match x {
            IntegerBits::Eight => ethabi::ParamType::Uint(8),
            IntegerBits::Sixteen => ethabi::ParamType::Uint(16),
            IntegerBits::ThirtyTwo => ethabi::ParamType::Uint(32),
            IntegerBits::SixtyFour => ethabi::ParamType::Uint(64),
        },
        Boolean => ethabi::ParamType::Bool,
        B256 => ethabi::ParamType::Uint(256),
        Contract => ethabi::ParamType::Address,
        Enum { .. } => ethabi::ParamType::Uint(8),
        Tuple(fields) => ethabi::ParamType::Tuple(
            fields
                .iter()
                .map(|f| json_abi_param_type(&type_engine.get(f.type_id), type_engine, decl_engine))
                .collect::<Vec<ethabi::ParamType>>(),
        ),
        Struct(decl_ref) => {
            let decl = decl_engine.get_struct(decl_ref);
            ethabi::ParamType::Tuple(
                decl.fields
                    .iter()
                    .map(|f| {
                        json_abi_param_type(
                            &type_engine.get(f.type_argument.type_id),
                            type_engine,
                            decl_engine,
                        )
                    })
                    .collect::<Vec<ethabi::ParamType>>(),
            )
        }
        Array(elem_ty, ..) => ethabi::ParamType::Array(Box::new(json_abi_param_type(
            &type_engine.get(elem_ty.type_id),
            type_engine,
            decl_engine,
        ))),
        _ => panic!("cannot convert type to Solidity ABI param type: {type_info:?}",),
    }
}

pub(self) fn generate_json_abi_function(
    fn_decl: &TyFunctionDeclaration,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
) -> ethabi::operation::Operation {
    // A list of all `ethabi::Param`s needed for inputs
    let input_types = fn_decl
        .parameters
        .iter()
        .map(|x| ethabi::Param {
            name: x.name.to_string(),
            kind: ethabi::ParamType::Address,
            internal_type: Some(get_json_type_str(
                &x.type_argument.type_id,
                type_engine,
                decl_engine,
                x.type_argument.type_id,
            )),
        })
        .collect::<Vec<_>>();

    // The single `ethabi::Param` needed for the output
    let output_type = ethabi::Param {
        name: String::default(),
        kind: ethabi::ParamType::Address,
        internal_type: Some(get_json_type_str(
            &fn_decl.return_type.type_id,
            type_engine,
            decl_engine,
            fn_decl.return_type.type_id,
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

pub(self) fn json_abi_str_type_arg(
    type_arg: &TypeArgument,
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
) -> String {
    json_abi_str(&type_engine.get(type_arg.type_id), type_engine, decl_engine)
}
