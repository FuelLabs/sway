//! Creation of HTML anchors for types that can be linked.
use crate::{doc::module::ModuleInfo, RenderPlan};
use anyhow::{anyhow, Result};
use horrorshow::{box_html, RenderBox};
use sway_core::{AbiName, TypeInfo};
use sway_types::{Named, Spanned};

/// Handles types & nested types that should have links
/// eg. (`[]` represent types with links).
///
/// ```sway
/// struct Foo {
///     foo: ([Foo], (u32, [Foo], ([Foo], [Foo])))
/// }
/// ```
//
// TODO: Add checks for multiline types
pub(crate) fn render_type_anchor(
    type_info: TypeInfo,
    render_plan: &RenderPlan,
    current_module_info: &ModuleInfo,
) -> Result<Box<dyn RenderBox>> {
    match type_info {
        TypeInfo::Array(ty_arg, len) => {
            let inner = render_type_anchor(
                (*render_plan.engines.te().get(ty_arg.type_id)).clone(),
                render_plan,
                current_module_info,
            )?;
            let len_string = format!("{:?}", render_plan.engines.help_out(len.expr()));
            Ok(box_html! {
                : "[";
                : inner;
                : format!("; {}]", len_string);
            })
        }
        TypeInfo::Slice(ty_arg) => {
            let inner = render_type_anchor(
                (*render_plan.engines.te().get(ty_arg.type_id)).clone(),
                render_plan,
                current_module_info,
            )?;
            Ok(box_html! {
                : "__slice[";
                : inner;
                : "]";
            })
        }
        TypeInfo::Tuple(ty_args) => {
            let mut rendered_args: Vec<_> = Vec::new();
            for ty_arg in ty_args {
                rendered_args.push(render_type_anchor(
                    (*render_plan.engines.te().get(ty_arg.type_id)).clone(),
                    render_plan,
                    current_module_info,
                )?);
            }
            Ok(box_html! {
                : "(";
                @ for arg in rendered_args {
                    : arg;
                }
                : ")";
            })
        }
        TypeInfo::Enum(decl_id) => {
            let enum_decl = render_plan.engines.de().get_enum(&decl_id);
            if !render_plan.document_private_items && enum_decl.visibility.is_private() {
                Ok(box_html! {
                    : enum_decl.name().as_str();
                })
            } else {
                let module_info = ModuleInfo::from_call_path(&enum_decl.call_path);
                let file_name = format!("enum.{}.html", enum_decl.name().as_str());
                let href =
                    module_info.file_path_from_location(&file_name, current_module_info, false)?;
                Ok(box_html! {
                    a(class="enum", href=href) {
                        : enum_decl.name().as_str();
                    }
                })
            }
        }
        TypeInfo::Struct(decl_id) => {
            let struct_decl = render_plan.engines.de().get_struct(&decl_id);
            if !render_plan.document_private_items && struct_decl.visibility.is_private() {
                Ok(box_html! {
                    : struct_decl.name().as_str();
                })
            } else {
                let module_info = ModuleInfo::from_call_path(&struct_decl.call_path);
                let file_name = format!("struct.{}.html", struct_decl.name().as_str());
                let href =
                    module_info.file_path_from_location(&file_name, current_module_info, false)?;
                Ok(box_html! {
                    a(class="struct", href=href) {
                        : struct_decl.name().as_str();
                    }
                })
            }
        }
        TypeInfo::UnknownGeneric { name, .. } => Ok(box_html! {
            : name.as_str();
        }),
        TypeInfo::StringArray(len) => Ok(box_html! {
            : len.expr().span().as_str();
        }),
        TypeInfo::UnsignedInteger(int_bits) => {
            use sway_types::integer_bits::IntegerBits;
            let uint = match int_bits {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
                IntegerBits::V256 => "u256",
            };
            Ok(box_html! {
                : uint;
            })
        }
        TypeInfo::Boolean => Ok(box_html! {
            : "bool";
        }),
        TypeInfo::ContractCaller { abi_name, .. } => {
            // TODO: determine whether we should give a link to this
            if let AbiName::Known(name) = abi_name {
                Ok(box_html! {
                    : name.suffix.as_str();
                })
            } else {
                Err(anyhow!("Deferred AbiName is unhandled"))
            }
        }
        TypeInfo::Custom {
            qualified_call_path,
            ..
        } => Ok(box_html! {
            : qualified_call_path.call_path.suffix.as_str();
        }),
        TypeInfo::B256 => Ok(box_html! {
            : "b256";
        }),
        _ => Err(anyhow!("Undetermined or unusable TypeInfo")),
    }
}
