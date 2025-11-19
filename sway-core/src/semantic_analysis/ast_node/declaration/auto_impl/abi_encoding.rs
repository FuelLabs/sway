use crate::{
    asm_generation::fuel::compiler_constants::MISMATCHED_SELECTOR_REVERT_CODE,
    decl_engine::{DeclEngineGet, DeclId},
    language::{
        parsed::FunctionDeclarationKind,
        ty::{self, TyAstNode, TyDecl, TyEnumDecl, TyFunctionDecl, TyStructDecl},
        Purity,
    },
    Engines, TypeInfo, TypeParameter,
};
use itertools::Itertools;
use std::collections::BTreeMap;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{BaseIdent, Named, SourceId, Span, Spanned};

#[derive(Default)]
pub struct AbiEncodingAutoImplInfo {}

pub type AbiEncodingAutoImplContext<'a, 'b> =
    super::AutoImplContext<'a, 'b, AbiEncodingAutoImplInfo>;

impl<'a, 'b> AbiEncodingAutoImplContext<'a, 'b>
where
    'a: 'b,
{
    fn generate_abi_encode_code(
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
            self.generate_type_parameters_constraints_code(type_parameters, Some("AbiEncode"));

        let name = name.as_raw_ident_str();

        if body.is_empty() {
            format!("#[allow(dead_code, deprecated)] impl{type_parameters_declaration_expanded} AbiEncode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code, deprecated)]
                fn abi_encode(self, buffer: Buffer) -> Buffer {{
                    buffer
                }}
            }}")
        } else {
            format!("#[allow(dead_code, deprecated)] impl{type_parameters_declaration_expanded} AbiEncode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code, deprecated)]
                fn abi_encode(self, buffer: Buffer) -> Buffer {{
                    {body}
                    buffer
                }}
            }}")
        }
    }

    fn generate_abi_decode_code(
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
            self.generate_type_parameters_constraints_code(type_parameters, Some("AbiDecode"));

        let name = name.as_raw_ident_str();

        if body == "Self {  }" {
            format!("#[allow(dead_code, deprecated)] impl{type_parameters_declaration_expanded} AbiDecode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code, deprecated)]
                fn abi_decode(ref mut _buffer: BufferReader) -> Self {{
                    {body}
                }}
            }}")
        } else {
            format!("#[allow(dead_code, deprecated)] impl{type_parameters_declaration_expanded} AbiDecode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code, deprecated)]
                fn abi_decode(ref mut buffer: BufferReader) -> Self {{
                    {body}
                }}
            }}")
        }
    }

    fn generate_abi_encode_struct_body(&self, _engines: &Engines, decl: &TyStructDecl) -> String {
        let mut code = String::new();

        for f in decl.fields.iter() {
            code.push_str(&format!(
                "let buffer = self.{field_name}.abi_encode(buffer);\n",
                field_name = f.name.as_raw_ident_str(),
            ));
        }

        code
    }

    fn generate_abi_decode_struct_body(
        &self,
        engines: &Engines,
        decl: &TyStructDecl,
    ) -> Option<String> {
        let mut code = String::new();
        for f in decl.fields.iter() {
            code.push_str(&format!(
                "{field_name}: buffer.decode::<{field_type_name}>(),",
                field_name = f.name.as_raw_ident_str(),
                field_type_name = Self::generate_type(engines, &f.type_argument)?,
            ));
        }

        Some(format!("Self {{ {code} }}"))
    }

    fn generate_abi_decode_enum_body(
        &self,
        engines: &Engines,
        decl: &TyEnumDecl,
    ) -> Option<String> {
        let enum_name = decl.call_path.suffix.as_raw_ident_str();
        let arms = decl.variants.iter()
            .map(|x| {
                let name = x.name.as_raw_ident_str();
                Some(match &*engines.te().get(x.type_argument.type_id) {
                    // unit
                    TypeInfo::Tuple(fields) if fields.is_empty() => {
                        format!("{} => {}::{}, \n", x.tag, enum_name, name)
                    },
                    _ => {
                        let variant_type_name = Self::generate_type(engines, &x.type_argument)?;
                        format!("{tag_value} => {enum_name}::{variant_name}(buffer.decode::<{variant_type}>()), \n",
                            tag_value = x.tag,
                            enum_name = enum_name,
                            variant_name = name,
                            variant_type = variant_type_name
                        )
                    }
                })
            })
        .collect::<Option<String>>()?;

        use std::fmt::Write;
        let mut code = String::new();
        let _ = writeln!(&mut code, "let variant: u64 = buffer.decode::<u64>();");
        let _ = writeln!(&mut code, "match variant {{ {arms} _ => __revert(0), }}");

        Some(code)
    }

    fn generate_abi_encode_enum_body(&self, engines: &Engines, decl: &TyEnumDecl) -> String {
        if decl.variants.is_empty() {
            return "".into();
        }

        let enum_name = decl.call_path.suffix.as_raw_ident_str();
        let arms = decl
            .variants
            .iter()
            .map(|x| {
                let name = x.name.as_raw_ident_str();
                if engines.te().get(x.type_argument.type_id).is_unit() {
                    format!(
                        "{enum_name}::{variant_name} => {{
                        {tag_value}u64.abi_encode(buffer)
                    }}, \n",
                        tag_value = x.tag,
                        enum_name = enum_name,
                        variant_name = name
                    )
                } else {
                    format!(
                        "{enum_name}::{variant_name}(value) => {{
                        let buffer = {tag_value}u64.abi_encode(buffer);
                        let buffer = value.abi_encode(buffer);
                        buffer
                    }}, \n",
                        tag_value = x.tag,
                        enum_name = enum_name,
                        variant_name = name,
                    )
                }
            })
            .collect::<String>();

        format!("let buffer = match self {{ {arms} }};")
    }

    // Auto implements AbiEncode and AbiDecode for structs and returns their `AstNode`s.
    fn auto_impl_abi_encode_and_decode_for_struct(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> Option<(Option<TyAstNode>, Option<TyAstNode>)> {
        // Dependencies of the codec library in std cannot have abi encoding implemented for them.
        if self.ctx.namespace.current_package_name().as_str() == "std"
            && matches!(
                self.ctx.namespace.current_module().name().as_str(),
                "codec" | "raw_slice" | "raw_ptr" | "ops" | "primitives" | "registers" | "flags"
            )
        {
            return Some((None, None));
        }

        let implementing_for_decl_id = decl.to_struct_decl(&Handler::default(), engines).unwrap();
        let struct_decl = self.ctx.engines().de().get(&implementing_for_decl_id);

        let abi_encode_body = self.generate_abi_encode_struct_body(engines, &struct_decl);
        let abi_encode_code = self.generate_abi_encode_code(
            struct_decl.name(),
            &struct_decl.generic_parameters,
            abi_encode_body,
        );
        let abi_encode_node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            struct_decl.span().source_id(),
            &abi_encode_code,
            crate::build_config::DbgGeneration::None,
        );

        let abi_decode_body = self.generate_abi_decode_struct_body(engines, &struct_decl);
        let abi_decode_code = self.generate_abi_decode_code(
            struct_decl.name(),
            &struct_decl.generic_parameters,
            abi_decode_body?,
        );
        let abi_decode_node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            struct_decl.span().source_id(),
            &abi_decode_code,
            crate::build_config::DbgGeneration::None,
        );

        Some((abi_encode_node.ok(), abi_decode_node.ok()))
    }

    fn auto_impl_abi_encode_and_decode_for_enum(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> Option<(Option<TyAstNode>, Option<TyAstNode>)> {
        // Dependencies of the codec library in std cannot have abi encoding implemented for them.
        if self.ctx.namespace.current_package_name().as_str() == "std"
            && matches!(
                self.ctx.namespace.current_module().name().as_str(),
                "codec" | "raw_slice" | "raw_ptr" | "ops" | "primitives" | "registers" | "flags"
            )
        {
            return Some((None, None));
        }

        let enum_decl_id = decl.to_enum_id(&Handler::default(), engines).unwrap();
        let enum_decl = self.ctx.engines().de().get(&enum_decl_id);

        let abi_encode_body = self.generate_abi_encode_enum_body(engines, &enum_decl);
        let abi_encode_code = self.generate_abi_encode_code(
            enum_decl.name(),
            &enum_decl.generic_parameters,
            abi_encode_body,
        );
        let abi_encode_node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            enum_decl.span().source_id(),
            &abi_encode_code,
            crate::build_config::DbgGeneration::None,
        );

        let abi_decode_body = self.generate_abi_decode_enum_body(engines, &enum_decl);
        let abi_decode_code = self.generate_abi_decode_code(
            enum_decl.name(),
            &enum_decl.generic_parameters,
            abi_decode_body?,
        );
        let abi_decode_node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            enum_decl.span().source_id(),
            &abi_decode_code,
            crate::build_config::DbgGeneration::None,
        );

        Some((abi_encode_node.ok(), abi_decode_node.ok()))
    }

    pub fn generate_abi_encode_and_decode_impls(
        &mut self,
        engines: &Engines,
        decl: &ty::TyDecl,
    ) -> (Option<TyAstNode>, Option<TyAstNode>) {
        match decl {
            TyDecl::StructDecl(_) => self
                .auto_impl_abi_encode_and_decode_for_struct(engines, decl)
                .unwrap_or((None, None)),
            TyDecl::EnumDecl(_) => self
                .auto_impl_abi_encode_and_decode_for_enum(engines, decl)
                .unwrap_or((None, None)),
            _ => (None, None),
        }
    }

    pub(crate) fn generate_contract_entry(
        &mut self,
        engines: &Engines,
        original_source_id: Option<&SourceId>,
        contract_fns: &[DeclId<TyFunctionDecl>],
        fallback_fn: Option<DeclId<TyFunctionDecl>>,
        handler: &Handler,
    ) -> Result<TyAstNode, ErrorEmitted> {
        let mut reads = false;
        let mut writes = false;

        // used to check for name collisions
        let mut contract_methods: BTreeMap<String, Vec<Span>> = <_>::default();

        let mut arm_by_size = BTreeMap::<usize, String>::default();

        // generate code
        let mut method_names = String::new();
        for r in contract_fns {
            let decl = engines.de().get(r);

            // For contract methods, even if their names are raw identifiers,
            // we use just the name, because the generated methods will be prefixed
            // with `__contract_entry_`.
            let name = decl.name.as_str();
            if !contract_methods.contains_key(name) {
                contract_methods.insert(name.to_string(), vec![]);
            }
            contract_methods
                .get_mut(name)
                .unwrap()
                .push(decl.name.span());

            match decl.purity {
                Purity::Pure => {}
                Purity::Reads => reads = true,
                Purity::Writes => writes = true,
                Purity::ReadsWrites => {
                    reads = true;
                    writes = true;
                }
            }

            let Some(args_types) = decl
                .parameters
                .iter()
                .map(|x| Self::generate_type(engines, &x.type_argument))
                .collect::<Option<Vec<String>>>()
            else {
                let err = handler.emit_err(CompileError::UnknownType {
                    span: Span::dummy(),
                });
                return Err(err);
            };
            let args_types = itertools::intersperse(args_types, ", ".into()).collect::<String>();

            let args_types = if args_types.is_empty() {
                "()".into()
            } else {
                format!("({args_types},)")
            };

            let expanded_args = itertools::intersperse(
                decl.parameters
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("args.{i}")),
                ", ".into(),
            )
            .collect::<String>();

            let Some(return_type) = Self::generate_type(engines, &decl.return_type) else {
                let err = handler.emit_err(CompileError::UnknownType {
                    span: Span::dummy(),
                });
                return Err(err);
            };

            let method_name = decl.name.as_str();
            let offset = if let Some(offset) = method_names.find(method_name) {
                offset
            } else {
                let offset = method_names.len();
                method_names.push_str(method_name);
                offset
            };

            let method_name_len = method_name.len();
            let code = arm_by_size.entry(method_name.len()).or_default();

            code.push_str(&format!("
            let is_this_method = asm(r, ptr: _method_name_ptr, name: _method_names_ptr, len: {method_name_len}) {{ addi r name i{offset}; meq r ptr r len; r: bool }};
            if is_this_method {{\n"));

            if args_types == "()" {
                code.push_str(&format!(
                    "let _result = __contract_entry_{method_name}();\n"
                ));
            } else {
                code.push_str(&format!(
                    "let args: {args_types} = _buffer.decode::<{args_types}>();
                    let _result: {return_type} = __contract_entry_{method_name}({expanded_args});\n"
                ));
            }

            if return_type == "()" {
                code.push_str("__contract_ret(asm() { zero: raw_ptr }, 0);");
            } else {
                code.push_str(&format!(
                    "let _result: raw_slice = encode::<{return_type}>(_result);
                    __contract_ret(_result.ptr(), _result.len::<u8>());"
                ));
            }

            code.push_str("\n}\n");
        }

        // check contract methods are unique
        // we need to allow manual_try_fold to avoid short-circuit and show
        // all errors.
        #[allow(clippy::manual_try_fold)]
        contract_methods
            .into_iter()
            .fold(Ok(()), |error, (_, spans)| {
                if spans.len() > 1 {
                    Err(handler
                        .emit_err(CompileError::MultipleContractsMethodsWithTheSameName { spans }))
                } else {
                    error
                }
            })?;

        let fallback = if let Some(fallback_fn) = fallback_fn {
            let fallback_fn = engines.de().get(&fallback_fn);
            let Some(return_type) = Self::generate_type(engines, &fallback_fn.return_type) else {
                let err = handler.emit_err(CompileError::UnknownType {
                    span: Span::dummy(),
                });
                return Err(err);
            };
            let method_name = fallback_fn.name.as_raw_ident_str();
            match fallback_fn.purity {
                Purity::Pure => {}
                Purity::Reads => reads = true,
                Purity::Writes => writes = true,
                Purity::ReadsWrites => {
                    reads = true;
                    writes = true;
                }
            }
            format!("let result: raw_slice = encode::<{return_type}>({method_name}()); __contract_ret(result.ptr(), result.len::<u8>());")
        } else {
            // as the old encoding does
            format!("__revert({MISMATCHED_SELECTOR_REVERT_CODE});")
        };

        let att = match (reads, writes) {
            (true, true) => "#[storage(read, write)]",
            (true, false) => "#[storage(read)]",
            (false, true) => "#[storage(write)]",
            (false, false) => "",
        };

        let code = arm_by_size
            .iter()
            .map(|(len, code)| format!("if _method_len == {len} {{ {code} }}"))
            .join("");
        let code = format!(
            "{att} pub fn __entry() {{
            let _method_names = \"{method_names}\";
            let mut _buffer = BufferReader::from_second_parameter();
            
            let mut _first_param_buffer = BufferReader::from_first_parameter();
            let _method_len = _first_param_buffer.read::<u64>();
            let _method_name_ptr = _first_param_buffer.ptr();
            let _method_names_ptr = _method_names.as_ptr();
            {code}
            {fallback}
        }}"
        );

        let entry_fn = self.parse_fn_to_ty_ast_node(
            engines,
            original_source_id,
            FunctionDeclarationKind::Entry,
            &code,
            crate::build_config::DbgGeneration::None,
        );

        match entry_fn {
            Ok(entry_fn) => Ok(entry_fn),
            Err(gen_handler) => {
                Self::check_impl_is_missing(handler, &gen_handler);
                Self::check_std_is_missing(handler, &gen_handler);
                Err(gen_handler.emit_err(CompileError::CouldNotGenerateEntry {
                    span: Span::dummy(),
                }))
            }
        }
    }

    pub(crate) fn generate_predicate_entry(
        &mut self,
        engines: &Engines,
        decl: &TyFunctionDecl,
        handler: &Handler,
    ) -> Result<TyAstNode, ErrorEmitted> {
        let Some(args_types) = decl
            .parameters
            .iter()
            .map(|x| Self::generate_type(engines, &x.type_argument))
            .collect::<Option<Vec<String>>>()
        else {
            let err = handler.emit_err(CompileError::UnknownType {
                span: Span::dummy(),
            });
            return Err(err);
        };
        let args_types = itertools::intersperse(args_types, ", ".into()).collect::<String>();

        let expanded_args = itertools::intersperse(
            decl.parameters
                .iter()
                .enumerate()
                .map(|(i, _)| format!("args.{i}")),
            ", ".into(),
        )
        .collect::<String>();

        let code = if args_types.is_empty() {
            "pub fn __entry() -> bool { main() }".to_string()
        } else {
            let args_types = format!("({args_types},)");
            format!(
                "pub fn __entry() -> bool {{
                let args: {args_types} = decode_predicate_data::<{args_types}>();
                main({expanded_args})
            }}"
            )
        };

        let entry_fn = self.parse_fn_to_ty_ast_node(
            engines,
            decl.span.source_id(),
            FunctionDeclarationKind::Entry,
            &code,
            crate::build_config::DbgGeneration::None,
        );

        match entry_fn {
            Ok(entry_fn) => Ok(entry_fn),
            Err(gen_handler) => {
                Self::check_impl_is_missing(handler, &gen_handler);
                Self::check_std_is_missing(handler, &gen_handler);
                Err(gen_handler.emit_err(CompileError::CouldNotGenerateEntry {
                    span: Span::dummy(),
                }))
            }
        }
    }

    // Check std is missing and give a more user-friendly error message.
    fn check_std_is_missing(handler: &Handler, gen_handler: &Handler) {
        let encode_not_found = gen_handler
            .find_error(|x| matches!(x, CompileError::SymbolNotFound { .. }))
            .is_some();
        if encode_not_found {
            handler.emit_err(CompileError::CouldNotGenerateEntryMissingStd {
                span: Span::dummy(),
            });
        }
    }

    // Check cannot encode or decode type
    fn check_impl_is_missing(handler: &Handler, gen_handler: &Handler) {
        let constraint_not_satisfied = gen_handler.find_error(|x| {
            matches!(x, CompileError::TraitConstraintNotSatisfied { trait_name, .. }
                if trait_name == "AbiEncode" || trait_name == "AbiDecode" && {
                true
            })
        });
        if let Some(constraint_not_satisfied) = constraint_not_satisfied {
            let ty = match constraint_not_satisfied {
                CompileError::TraitConstraintNotSatisfied { ty, .. } => ty,
                _ => unreachable!("unexpected error"),
            };
            handler.emit_err(CompileError::CouldNotGenerateEntryMissingImpl {
                ty,
                span: Span::dummy(),
            });
        }
    }

    pub(crate) fn generate_script_entry(
        &mut self,
        engines: &Engines,
        decl: &TyFunctionDecl,
        handler: &Handler,
    ) -> Result<TyAstNode, ErrorEmitted> {
        let Some(args_types) = decl
            .parameters
            .iter()
            .map(|x| Self::generate_type(engines, &x.type_argument))
            .collect::<Option<Vec<String>>>()
        else {
            let err = handler.emit_err(CompileError::UnknownType {
                span: Span::dummy(),
            });
            return Err(err);
        };
        let args_types = itertools::intersperse(args_types, ", ".into()).collect::<String>();
        let args_types = if args_types.is_empty() {
            "()".into()
        } else {
            format!("({args_types},)")
        };

        let expanded_args = itertools::intersperse(
            decl.parameters
                .iter()
                .enumerate()
                .map(|(i, _)| format!("args.{i}")),
            ", ".into(),
        )
        .collect::<String>();

        let Some(return_type) = Self::generate_type(engines, &decl.return_type) else {
            let err = handler.emit_err(CompileError::UnknownType {
                span: Span::dummy(),
            });
            return Err(err);
        };

        let return_encode = if return_type == "()" {
            "__contract_ret(0, 0)".to_string()
        } else {
            format!("encode_and_return::<{return_type}>(_result)")
        };

        let code = if args_types == "()" {
            format!(
                "pub fn __entry() -> ! {{
                let _result: {return_type} = main();
                {return_encode}
            }}"
            )
        } else {
            format!(
                "pub fn __entry() -> ! {{
                let args: {args_types} = decode_script_data::<{args_types}>();
                let _result: {return_type} = main({expanded_args});
                {return_encode}
            }}"
            )
        };

        let entry_fn = self.parse_fn_to_ty_ast_node(
            engines,
            decl.span.source_id(),
            FunctionDeclarationKind::Entry,
            &code,
            crate::build_config::DbgGeneration::None,
        );

        match entry_fn {
            Ok(entry_fn) => Ok(entry_fn),
            Err(gen_handler) => {
                Self::check_std_is_missing(handler, &gen_handler);
                Self::check_impl_is_missing(handler, &gen_handler);
                Err(gen_handler.emit_err(CompileError::CouldNotGenerateEntry {
                    span: Span::dummy(),
                }))
            }
        }
    }
}
