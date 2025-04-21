use sway_error::handler::Handler;
use sway_types::{BaseIdent, Named, Spanned};

use crate::{
    decl_engine::DeclEngineGet,
    language::ty::{self, TyAstNode, TyDecl, TyEnumDecl, TyStructDecl},
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
            TyDecl::EnumDecl(_) => self.debug_auto_impl_enum(engines, decl).unwrap_or(None),
            _ => None,
        }
    }

    // Auto implements Debug for structs and returns their `AstNode`s.
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

        let body = self.generate_fmt_struct_body(engines, &struct_decl);
        let code = self.generate_fmt_code(struct_decl.name(), &struct_decl.type_parameters, body);
        let node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            struct_decl.span().source_id().cloned(),
            &code,
            crate::build_config::DbgGeneration::None,
        );

        Some(node.ok())
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

        format!("#[allow(dead_code)] impl{type_parameters_declaration} Debug for {name}{type_parameters_declaration}{type_parameters_constraints} {{
            #[allow(dead_code)]
            fn fmt(self, ref mut _f: Formatter) {{
                {body}
            }}
        }}")
    }

    fn generate_fmt_struct_body(&self, _engines: &Engines, decl: &TyStructDecl) -> String {
        let mut fields = String::new();

        for field in decl.fields.iter() {
            fields.push_str(&format!(
                ".field(\"{field_name}\", self.{field_name})\n",
                field_name = field.name.as_str(),
            ));
        }

        format!(
            "_f.debug_struct(\"{}\"){fields}.finish();",
            decl.name().as_str()
        )
    }

    // Auto implements Debug for enums and returns their `AstNode`s.
    fn debug_auto_impl_enum(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> Option<Option<TyAstNode>> {
        if self.ctx.namespace.current_package_name().as_str() == "core" {
            return Some(None);
        }

        let enum_decl_id = decl.to_enum_id(&Handler::default(), engines).unwrap();
        let enum_decl = self.ctx.engines().de().get(&enum_decl_id);

        let body = self.generate_fmt_enum_body(engines, &enum_decl);
        let code = self.generate_fmt_code(enum_decl.name(), &enum_decl.type_parameters, body);
        let node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            enum_decl.span().source_id().cloned(),
            &code,
            crate::build_config::DbgGeneration::None,
        );

        Some(node.ok())
    }

    fn generate_fmt_enum_body(&self, engines: &Engines, decl: &TyEnumDecl) -> String {
        let enum_name = decl.call_path.suffix.as_str();

        let arms = decl
            .variants
            .iter()
            .map(|variant| {
                let variant_name = variant.name.as_str();
                if engines.te().get(variant.type_argument.type_id).is_unit() {
                    format!(
                        "{enum_name}::{variant_name} => {{
                        _f.print_str(\"{variant_name}\");
                    }}, \n",
                        enum_name = enum_name,
                        variant_name = variant_name
                    )
                } else {
                    format!(
                        "{enum_name}::{variant_name}(value) => {{
                        _f.debug_tuple(\"{enum_name}\").field(value).finish();
                    }}, \n",
                        enum_name = enum_name,
                        variant_name = variant_name,
                    )
                }
            })
            .collect::<String>();

        format!("match self {{ {arms} }};")
    }
}
