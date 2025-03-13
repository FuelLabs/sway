use sway_error::handler::Handler;
use sway_types::{BaseIdent, Named, Spanned};

use crate::{
    decl_engine::DeclEngineGet,
    language::ty::{self, TyAstNode, TyDecl, TyStructDecl},
    Engines, TypeParameter,
};

use super::abi_encoding::AbiEncodingAutoImplInfo;

pub type DebugAutoImplContext<'a, 'b> = super::AutoImplContext<'a, 'b, AbiEncodingAutoImplInfo>;

impl<'a, 'b> DebugAutoImplContext<'a, 'b>
where
    'a: 'b,
{
    pub fn generate_debug_impl(
        &mut self,
        engines: &Engines,
        decl: &ty::TyDecl,
    ) -> Option<TyAstNode> {
        match decl {
            TyDecl::StructDecl(_) => self.debug_auto_impl_struct(engines, decl).unwrap_or(None),
            TyDecl::EnumDecl(_) => None,
            _ => None,
        }
    }

    // Auto implements AbiEncode and AbiDecode for structs and returns their `AstNode`s.
    fn debug_auto_impl_struct(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> Option<Option<TyAstNode>> {
        if self.ctx.namespace.current_package_name().as_str() == "core" {
            return Some(None);
        }

        let implementing_for_decl_id = decl.to_struct_decl(&Handler::default(), engines).unwrap();
        let struct_decl = self.ctx.engines().de().get(&implementing_for_decl_id);

        let program_id = struct_decl.span().source_id().map(|sid| sid.program_id());

        let abi_encode_body = self.generate_fmt_struct_body(engines, &struct_decl);
        let abi_encode_code = self.generate_fmt_code(
            struct_decl.name(),
            &struct_decl.type_parameters,
            abi_encode_body,
        );
        let abi_encode_node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            program_id,
            &abi_encode_code,
            crate::build_config::DbgGeneration::None,
        );

        Some(abi_encode_node.ok())
    }

    fn generate_fmt_code(
        &self,
        name: &BaseIdent,
        type_parameters: &[TypeParameter],
        body: String,
    ) -> String {
        let type_parameters_declaration =
            self.generate_type_parameters_declaration_code(type_parameters);
        let type_parameters_constraints =
            self.generate_type_parameters_constraints_code(type_parameters, "Debug");

        let name = name.as_str();

        if body.is_empty() {
            format!("#[allow(dead_code)] impl{type_parameters_declaration} Debug for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code)]
                fn fmt(self, ref mut _f: Formatter) {{ }}
            }}")
        } else {
            format!("#[allow(dead_code)] impl{type_parameters_declaration} Debug for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code)]
                fn fmt(self, ref mut f: Formatter) {{
                    f.debug_struct(\"{name}\")
                        {body}
                        .finish();
                }}
            }}")
        }
    }

    fn generate_fmt_struct_body(&self, _engines: &Engines, decl: &TyStructDecl) -> String {
        let mut code = String::new();

        for f in decl.fields.iter() {
            code.push_str(&format!(
                ".field(\"{field_name}\", self.{field_name})\n",
                field_name = f.name.as_str(),
            ));
        }

        code
    }
}
