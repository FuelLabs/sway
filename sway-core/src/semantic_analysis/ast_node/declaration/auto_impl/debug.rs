use sway_error::handler::Handler;
use sway_types::{BaseIdent, Named, Spanned};

use crate::{
    decl_engine::DeclEngineGet,
    language::ty::{self, TyAstNode, TyDecl, TyEnumDecl, TyStructDecl},
    Engines, TypeParameter,
};

#[derive(Default)]
pub struct DebugAutoImplInfo {}

pub type DebugAutoImplContext<'a, 'b> = super::AutoImplContext<'a, 'b, DebugAutoImplInfo>;

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
            TyDecl::StructDecl(_) => self.auto_impl_debug_struct(engines, decl),
            TyDecl::EnumDecl(_) => self.auto_impl_debug_enum(engines, decl),
            _ => None,
        }
    }

    // checks if the current module is a dependency of the `debug` module.
    fn is_debug_dependency(&self) -> bool {
        // Dependencies of the debug library in std cannot have debug implemented for them.
        self.ctx.namespace.current_package_name().as_str() == "std"
            && matches!(
                self.ctx.namespace.current_module().name().as_str(),
                "codec"
                    | "raw_slice"
                    | "raw_ptr"
                    | "ops"
                    | "primitives"
                    | "registers"
                    | "flags"
                    | "debug"
            )
    }

    // Auto implements Debug for structs and returns their `AstNode`s.
    fn auto_impl_debug_struct(&mut self, engines: &Engines, decl: &TyDecl) -> Option<TyAstNode> {
        if self.is_debug_dependency() {
            return None;
        }

        let implementing_for_decl_id = decl.to_struct_decl(&Handler::default(), engines).unwrap();
        let struct_decl = self.ctx.engines().de().get(&implementing_for_decl_id);

        let body = self.generate_fmt_struct_body(engines, &struct_decl);
        let code =
            self.generate_fmt_code(struct_decl.name(), &struct_decl.generic_parameters, body);
        let node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            struct_decl.span().source_id(),
            &code,
            crate::build_config::DbgGeneration::None,
        );

        node.ok()
    }

    fn generate_fmt_code(
        &self,
        name: &BaseIdent,
        type_parameters: &[TypeParameter],
        body: String,
    ) -> String {
        let type_parameters_declaration_expanded =
            self.generate_type_parameters_declaration_code(type_parameters, true);
        let type_parameters_declaration =
            self.generate_type_parameters_declaration_code(type_parameters, false);
        let type_parameters_constraints =
            self.generate_type_parameters_constraints_code(type_parameters, Some("Debug"));

        let name = name.as_raw_ident_str();

        format!("#[allow(dead_code, deprecated)] impl{type_parameters_declaration_expanded} Debug for {name}{type_parameters_declaration}{type_parameters_constraints} {{
            #[allow(dead_code, deprecated)]
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
                field_name = field.name.as_raw_ident_str(),
            ));
        }

        format!(
            "_f.debug_struct(\"{}\"){fields}.finish();",
            decl.name().as_raw_ident_str()
        )
    }

    // Auto implements Debug for enums and returns their `AstNode`s.
    fn auto_impl_debug_enum(&mut self, engines: &Engines, decl: &TyDecl) -> Option<TyAstNode> {
        if self.is_debug_dependency() {
            return None;
        }

        let enum_decl_id = decl.to_enum_id(&Handler::default(), engines).unwrap();
        let enum_decl = self.ctx.engines().de().get(&enum_decl_id);

        let body = self.generate_fmt_enum_body(engines, &enum_decl);
        let code = self.generate_fmt_code(enum_decl.name(), &enum_decl.generic_parameters, body);
        let node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            enum_decl.span().source_id(),
            &code,
            crate::build_config::DbgGeneration::None,
        );

        node.ok()
    }

    fn generate_fmt_enum_body(&self, engines: &Engines, decl: &TyEnumDecl) -> String {
        let enum_name = decl.call_path.suffix.as_raw_ident_str();

        let arms = decl
            .variants
            .iter()
            .map(|variant| {
                let variant_name = variant.name.as_raw_ident_str();
                if engines.te().get(variant.type_argument.type_id()).is_unit() {
                    format!(
                        "{enum_name}::{variant_name} => {{
                        _f.print_str(\"{variant_name}\");
                    }}, \n"
                    )
                } else {
                    format!(
                        "{enum_name}::{variant_name}(value) => {{
                        _f.debug_tuple(\"{enum_name}\").field(value).finish();
                    }}, \n",
                    )
                }
            })
            .collect::<String>();

        format!("match self {{ {arms} }};")
    }
}
