use sway_core::{
    transform::{AttributeKind, AttributesMap},
    TypeParameter,
};
use sway_types::Spanned;

use crate::capabilities::code_actions::CodeAction;

pub(crate) const CONTRACT: &str = "Contract";
pub(crate) const TAB: &str = "    ";

pub(crate) trait GenerateImplCodeAction<'a, T: Spanned>: CodeAction<'a, T> {
    /// Returns a [String] holding the name of the declaration.
    fn decl_name(&self) -> String;

    /// Returns an optional [String] of the type parameters for the given [TypeParameter] vector.
    fn type_param_string(&self, type_params: &Vec<TypeParameter>) -> Option<String> {
        if type_params.is_empty() {
            None
        } else {
            Some(
                type_params
                    .iter()
                    .map(|param| param.name_ident.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }
    }

    /// Returns a [String] of a generated impl with the optional `for <for_name>` signature.
    /// Can be used for both ABI and Struct impls.
    fn impl_string(
        &self,
        type_params: Option<String>,
        body: String,
        for_name: Option<String>,
    ) -> String {
        let for_string = match for_name {
            Some(name) => format!(" for {name}"),
            None => "".to_string(),
        };
        let type_param_string = match type_params {
            Some(params) => format!("<{params}>"),
            None => "".to_string(),
        };
        format!(
            "\nimpl{} {}{}{} {{{}}}\n",
            type_param_string,
            self.decl_name(),
            type_param_string,
            for_string,
            body
        )
    }

    /// Returns a [String] of a an attribute map, optionally excluding comments.
    fn attribute_string(&self, attr_map: &AttributesMap, include_comments: bool) -> String {
        let attr_string = attr_map
            .iter()
            .map(|(kind, attrs)| {
                attrs
                    .iter()
                    .filter_map(|attr| match kind {
                        AttributeKind::DocComment { .. } => {
                            if include_comments {
                                return Some(format!("{}{}", TAB, attr.span.as_str()));
                            }
                            None
                        }
                        _ => Some(format!("{}{}", TAB, attr.span.as_str())),
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            })
            .collect::<Vec<String>>()
            .join("\n");
        let attribute_padding = match attr_string.len() > 1 {
            true => "\n",
            false => "",
        };
        format!("{attr_string}{attribute_padding}")
    }

    /// Returns a [String] of a generated function signature.
    fn fn_signature_string(
        &self,
        fn_name: String,
        params_string: String,
        attr_map: &AttributesMap,
        return_type_string: String,
        body: Option<String>,
    ) -> String {
        let attribute_string = self.attribute_string(attr_map, false);
        let body_string = match body {
            Some(body) => format!(" {body} "),
            None => String::new(),
        };
        format!(
            "{attribute_string}{TAB}fn {fn_name}({params_string}){return_type_string} {{{body_string}}}",
        )
    }
}
