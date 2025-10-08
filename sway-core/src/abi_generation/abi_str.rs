use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{integer_bits::IntegerBits, Ident, Named};

use crate::{ast_elements::type_argument::GenericTypeArgument, language::CallPath, transform, Engines, GenericArgument, TypeId, TypeInfo};

#[derive(Clone)]
pub struct AbiStrContext {
    pub program_name: String,
    pub abi_with_callpaths: bool,
    pub abi_with_fully_specified_types: bool,
    pub abi_root_type_without_generic_type_parameters: bool,
}

impl TypeId {
    /// Gives back a string that represents the type, considering what it resolves to
    pub fn get_abi_type_str(
        &self,
        handler: &Handler,
        ctx: &AbiStrContext,
        engines: &Engines,
        resolved_type_id: TypeId,
    ) -> Result<String, ErrorEmitted> {
        let type_engine = engines.te();
        let self_abi_str = type_engine
            .get(*self)
            .abi_str(handler, ctx, engines, true)?;
        if self.is_generic_parameter(engines, resolved_type_id) {
            Ok(format!("generic {self_abi_str}"))
        } else {
            match (
                &*type_engine.get(*self),
                &*type_engine.get(resolved_type_id),
            ) {
                (TypeInfo::Custom { .. }, TypeInfo::Struct { .. })
                | (TypeInfo::Custom { .. }, TypeInfo::Enum { .. }) => type_engine
                    .get(resolved_type_id)
                    .abi_str(handler, ctx, engines, true),
                (_, TypeInfo::Alias { ty, .. }) => {
                    ty.type_id
                        .get_abi_type_str(handler, ctx, engines, ty.type_id)
                }
                (TypeInfo::Tuple(fields), TypeInfo::Tuple(resolved_fields)) => {
                    assert_eq!(fields.len(), resolved_fields.len());
                    let field_strs = resolved_fields
                        .iter()
                        .map(|f| {
                            if ctx.abi_with_fully_specified_types {
                                type_engine
                                    .get(f.type_id)
                                    .abi_str(handler, ctx, engines, false)
                            } else {
                                Ok("_".to_string())
                            }
                        })
                        .collect::<Result<Vec<String>, _>>()?;
                    Ok(format!("({})", field_strs.join(", ")))
                }
                (TypeInfo::Array(_, length), TypeInfo::Array(type_arg, resolved_length)) => {
                    assert_eq!(
                        length.expr().as_literal_val(),
                        resolved_length.expr().as_literal_val(),
                        "{:?} {:?}",
                        length.expr().as_literal_val(),
                        resolved_length.expr().as_literal_val()
                    );
                    let inner_type = if ctx.abi_with_fully_specified_types {
                        type_engine
                            .get(type_arg.type_id)
                            .abi_str(handler, ctx, engines, false)?
                    } else {
                        "_".to_string()
                    };
                    Ok(format!(
                        "[{}; {:?}]",
                        inner_type,
                        engines.help_out(length.expr())
                    ))
                }
                (TypeInfo::Slice(type_arg), TypeInfo::Slice(_)) => {
                    let inner_type = if ctx.abi_with_fully_specified_types {
                        type_engine
                            .get(type_arg.type_id)
                            .abi_str(handler, ctx, engines, false)?
                    } else {
                        "_".to_string()
                    };
                    Ok(format!("[{inner_type}]"))
                }
                (TypeInfo::Custom { .. }, _) => Ok(format!("generic {self_abi_str}")),
                _ => type_engine
                    .get(resolved_type_id)
                    .abi_str(handler, ctx, engines, true),
            }
        }
    }
}

impl TypeInfo {
    pub fn abi_str(
        &self,
        handler: &Handler,
        ctx: &AbiStrContext,
        engines: &Engines,
        is_root: bool,
    ) -> Result<String, ErrorEmitted> {
        use TypeInfo::*;
        let decl_engine = engines.de();
        match self {
            Unknown => Ok("unknown".into()),
            Never => Ok("never".into()),
            UnknownGeneric { name, .. } => Ok(name.to_string()),
            Placeholder(_) => Ok("_".to_string()),
            TypeParam(param) => Ok(format!("typeparam({})", param.name())),
            StringSlice => Ok("str".into()),
            StringArray(length) => Ok(format!("str[{:?}]", engines.help_out(length.expr()))),
            UnsignedInteger(x) => Ok(match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
                IntegerBits::V256 => "u256",
            }
            .into()),
            Boolean => Ok("bool".into()),
            Custom {
                qualified_call_path: call_path,
                ..
            } => Ok(call_path.call_path.suffix.to_string()),
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| field.abi_str(handler, ctx, engines, false))
                    .collect::<Result<Vec<String>, ErrorEmitted>>()?;
                Ok(format!("({})", field_strs.join(", ")))
            }
            B256 => Ok("b256".into()),
            Numeric => Ok("u64".into()), // u64 is the default
            Contract => Ok("contract".into()),
            ErrorRecovery(_) => Ok("unknown due to error".into()),
            UntypedEnum(decl_id) => {
                let decl = engines.pe().get_enum(decl_id);
                Ok(format!("untyped enum {}", decl.name))
            }
            UntypedStruct(decl_id) => {
                let decl = engines.pe().get_struct(decl_id);
                Ok(format!("untyped struct {}", decl.name))
            }
            Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                let type_params = if (ctx.abi_root_type_without_generic_type_parameters && is_root)
                    || decl.generic_parameters.is_empty()
                {
                    ""
                } else {
                    let params = decl
                        .generic_parameters
                        .iter()
                        .map(|p| p.abi_str(handler, engines, ctx, false))
                        .collect::<Result<Vec<_>, _>>()?;
                    &format!("<{}>", params.join(","))
                };
                let abi_call_path = get_abi_call_path(handler, &decl.call_path, &decl.attributes)?;
                Ok(format!(
                    "enum {}{}",
                    call_path_display(ctx, &abi_call_path),
                    type_params
                ))
            }
            Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                let type_params = if (ctx.abi_root_type_without_generic_type_parameters && is_root)
                    || decl.generic_parameters.is_empty()
                {
                    "".into()
                } else {
                    let params = decl
                        .generic_parameters
                        .iter()
                        .map(|p| p.abi_str(handler, engines, ctx, false))
                        .collect::<Result<Vec<_>, _>>()?;
                    format!("<{}>", params.join(","))
                };
                let abi_call_path = get_abi_call_path(handler, &decl.call_path, &decl.attributes)?;
                Ok(format!(
                    "struct {}{}",
                    call_path_display(ctx, &abi_call_path),
                    type_params
                ))
            }
            ContractCaller { abi_name, .. } => Ok(format!("contract caller {abi_name}")),
            Array(elem_ty, length) => Ok(format!(
                "[{}; {:?}]",
                elem_ty.abi_str(handler, ctx, engines, false)?,
                engines.help_out(length.expr())
            )),
            RawUntypedPtr => Ok("raw untyped ptr".into()),
            RawUntypedSlice => Ok("raw untyped slice".into()),
            Ptr(ty) => Ok(format!(
                "__ptr {}",
                ty.abi_str(handler, ctx, engines, false)?
            )),
            Slice(ty) => Ok(format!(
                "__slice {}",
                ty.abi_str(handler, ctx, engines, false)?
            )),
            Alias { ty, .. } => Ok(ty.abi_str(handler, ctx, engines, false)?),
            TraitType {
                name,
                implemented_in: _,
            } => Ok(format!("trait type {name}")),
            Ref {
                to_mutable_value,
                referenced_type,
            } => {
                Ok(format!(
                    "__ref {}{}", // TODO: (REFERENCES) No references in ABIs according to the RFC. Or we want to have them?
                    if *to_mutable_value { "mut " } else { "" },
                    referenced_type.abi_str(handler, ctx, engines, false)?
                ))
            }
        }
    }
}

fn get_abi_call_path(
    handler: &Handler,
    call_path: &CallPath,
    attributes: &transform::Attributes,
) -> Result<CallPath, ErrorEmitted> {
    let mut abi_call_path = call_path.clone();
    if let Some(abi_name_attr) = attributes.abi_name() {
        let name = abi_name_attr.args.first().unwrap();
        let ident = Ident::new_no_span(name.get_string(handler, abi_name_attr)?.clone());
        abi_call_path.suffix = ident;
    }
    Ok(abi_call_path)
}

/// `call_path_display`  returns the provided `call_path` without the first prefix in case it is equal to the program name.
/// If the program name is `my_program` and the `call_path` is `my_program::MyStruct` then this function returns only `MyStruct`.
fn call_path_display(ctx: &AbiStrContext, call_path: &CallPath) -> String {
    if !ctx.abi_with_callpaths {
        return call_path.suffix.as_str().to_string();
    }
    let mut buf = String::new();
    for (index, prefix) in call_path.prefixes.iter().enumerate() {
        if index == 0 && prefix.as_str() == ctx.program_name {
            continue;
        }
        buf.push_str(prefix.as_str());
        buf.push_str("::");
    }
    buf.push_str(&call_path.suffix.to_string());

    buf
}

impl GenericTypeArgument {
    pub(self) fn abi_str(
        &self,
        handler: &Handler,
        ctx: &AbiStrContext,
        engines: &Engines,
        is_root: bool,
    ) -> Result<String, ErrorEmitted> {
        engines
            .te()
            .get(self.type_id)
            .abi_str(handler, ctx, engines, is_root)
    }
}
