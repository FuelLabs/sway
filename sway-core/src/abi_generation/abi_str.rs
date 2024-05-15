use sway_types::integer_bits::IntegerBits;

use crate::{language::CallPath, Engines, TypeArgument, TypeId, TypeInfo};

pub struct AbiStrContext {
    pub program_name: Option<String>,
    pub abi_with_callpaths: bool,
    pub abi_with_fully_specified_types: bool,
}

impl TypeId {
    /// Gives back a string that represents the type, considering what it resolves to
    pub fn get_abi_type_str(
        &self,
        ctx: &AbiStrContext,
        engines: &Engines,
        resolved_type_id: TypeId,
    ) -> String {
        let type_engine = engines.te();
        if self.is_generic_parameter(engines, resolved_type_id) {
            format!("generic {}", type_engine.get(*self).abi_str(ctx, engines))
        } else {
            match (
                &*type_engine.get(*self),
                &*type_engine.get(resolved_type_id),
            ) {
                (TypeInfo::Custom { .. }, TypeInfo::Struct { .. }) => {
                    type_engine.get(resolved_type_id).abi_str(ctx, engines)
                }
                (TypeInfo::Custom { .. }, TypeInfo::Enum { .. }) => {
                    type_engine.get(resolved_type_id).abi_str(ctx, engines)
                }
                (TypeInfo::Custom { .. }, TypeInfo::Alias { .. }) => {
                    type_engine.get(resolved_type_id).abi_str(ctx, engines)
                }
                (TypeInfo::Tuple(fields), TypeInfo::Tuple(resolved_fields)) => {
                    assert_eq!(fields.len(), resolved_fields.len());
                    let field_strs = fields
                        .iter()
                        .map(|f| {
                            if ctx.abi_with_fully_specified_types {
                                type_engine.get(f.type_id).abi_str(ctx, engines)
                            } else {
                                "_".to_string()
                            }
                        })
                        .collect::<Vec<String>>();
                    format!("({})", field_strs.join(", "))
                }
                (TypeInfo::Array(type_arg, count), TypeInfo::Array(_, resolved_count)) => {
                    assert_eq!(count.val(), resolved_count.val());
                    let inner_type = if ctx.abi_with_fully_specified_types {
                        type_engine.get(type_arg.type_id).abi_str(ctx, engines)
                    } else {
                        "_".to_string()
                    };
                    format!("[{}; {}]", inner_type, count.val())
                }
                (TypeInfo::Custom { .. }, _) => {
                    format!("generic {}", type_engine.get(*self).abi_str(ctx, engines))
                }
                _ => type_engine.get(resolved_type_id).abi_str(ctx, engines),
            }
        }
    }
}

impl TypeInfo {
    pub fn abi_str(&self, ctx: &AbiStrContext, engines: &Engines) -> String {
        use TypeInfo::*;
        let decl_engine = engines.de();
        let type_engine = engines.te();
        match self {
            Unknown => "unknown".into(),
            Never => "never".into(),
            UnknownGeneric { name, .. } => name.to_string(),
            Placeholder(_) => "_".to_string(),
            TypeParam(n) => format!("typeparam({n})"),
            StringSlice => "str".into(),
            StringArray(x) => format!("str[{}]", x.val()),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
                IntegerBits::V256 => "u256",
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
                    .map(|field| field.abi_str(ctx, engines))
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            B256 => "b256".into(),
            Numeric => "u64".into(), // u64 is the default
            Contract => "contract".into(),
            ErrorRecovery(_) => "unknown due to error".into(),
            Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                let type_params =
                    if !ctx.abi_with_fully_specified_types || decl.type_parameters.is_empty() {
                        "".into()
                    } else {
                        format!(
                            "<{}>",
                            decl.type_parameters
                                .iter()
                                .map(|p| type_engine.get(p.type_id).abi_str(ctx, engines))
                                .collect::<Vec<_>>()
                                .join(",")
                        )
                    };
                format!(
                    "enum {}{}",
                    call_path_display(ctx, &decl.call_path),
                    type_params
                )
            }
            Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                let type_params =
                    if !ctx.abi_with_fully_specified_types || decl.type_parameters.is_empty() {
                        "".into()
                    } else {
                        format!(
                            "<{}>",
                            decl.type_parameters
                                .iter()
                                .map(|p| type_engine.get(p.type_id).abi_str(ctx, engines))
                                .collect::<Vec<_>>()
                                .join(",")
                        )
                    };
                format!(
                    "struct {}{}",
                    call_path_display(ctx, &decl.call_path),
                    type_params
                )
            }
            ContractCaller { abi_name, .. } => {
                format!("contract caller {abi_name}")
            }
            Array(elem_ty, length) => {
                format!("[{}; {}]", elem_ty.abi_str(ctx, engines), length.val())
            }
            Storage { .. } => "contract storage".into(),
            RawUntypedPtr => "raw untyped ptr".into(),
            RawUntypedSlice => "raw untyped slice".into(),
            Ptr(ty) => {
                format!("__ptr {}", ty.abi_str(ctx, engines))
            }
            Slice(ty) => {
                format!("__slice {}", ty.abi_str(ctx, engines))
            }
            Alias { ty, .. } => ty.abi_str(ctx, engines),
            TraitType {
                name,
                trait_type_id: _,
            } => format!("trait type {}", name),
            Ref {
                to_mutable_value,
                referenced_type,
            } => {
                format!(
                    "__ref {}{}", // TODO-IG: No references in ABIs according to the RFC. Or we want to have them?
                    if *to_mutable_value { "mut " } else { "" },
                    referenced_type.abi_str(ctx, engines)
                )
            }
        }
    }
}

/// `call_path_display`  returns the provided `call_path` without the first prefix in case it is equal to the program name.
/// If the program name is `my_program` and the `call_path` is `my_program::MyStruct` then this function returns only `MyStruct`.
fn call_path_display(ctx: &AbiStrContext, call_path: &CallPath) -> String {
    if !ctx.abi_with_callpaths {
        return call_path.suffix.as_str().to_string();
    }
    let mut buf = String::new();
    for (index, prefix) in call_path.prefixes.iter().enumerate() {
        let mut skip_prefix = false;
        if index == 0 {
            if let Some(root_name) = &ctx.program_name {
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

impl TypeArgument {
    pub(self) fn abi_str(&self, ctx: &AbiStrContext, engines: &Engines) -> String {
        engines.te().get(self.type_id).abi_str(ctx, engines)
    }
}
