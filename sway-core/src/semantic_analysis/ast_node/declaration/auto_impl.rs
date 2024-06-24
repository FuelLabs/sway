use crate::{
    asm_generation::fuel::compiler_constants::MISMATCHED_SELECTOR_REVERT_CODE,
    decl_engine::{DeclEngineGet, DeclId, DeclRef},
    engine_threading::SpannedWithEngines,
    language::{
        parsed::{self, AstNodeContent, Declaration, FunctionDeclarationKind},
        ty::{self, TyAstNode, TyDecl, TyEnumDecl, TyFunctionDecl, TyStructDecl},
        Purity,
    },
    semantic_analysis::TypeCheckContext,
    Engines, TypeId, TypeInfo, TypeParameter,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_parse::Parse;
use sway_types::{integer_bits::IntegerBits, BaseIdent, Named, ProgramId, Span, Spanned};

/// Contains all information needed to implement AbiEncode
pub struct EncodingAutoImplContext<'a, 'b>
where
    'a: 'b,
{
    ctx: &'b mut TypeCheckContext<'a>,
}

impl<'a, 'b> EncodingAutoImplContext<'a, 'b>
where
    'a: 'b,
{
    pub fn new(ctx: &'b mut TypeCheckContext<'a>) -> Option<Self> {
        Some(Self { ctx })
    }

    fn parse<T>(engines: &Engines, program_id: Option<ProgramId>, input: &str) -> T
    where
        T: Parse,
    {
        // Uncomment this to see what is being generated
        //println!("{}", input);

        let handler = <_>::default();
        let source_id =
            program_id.map(|program_id| engines.se().get_autogenerated_source_id(program_id));

        let ts = sway_parse::lex(
            &handler,
            &std::sync::Arc::from(input),
            0,
            input.len(),
            source_id,
        )
        .unwrap();
        let mut p = sway_parse::Parser::new(&handler, &ts);
        p.check_double_underscore = false;

        let r = p.parse();

        assert!(!handler.has_errors(), "{:?}", handler);
        assert!(!handler.has_warnings(), "{:?}", handler);

        assert!(!p.has_errors());
        assert!(!p.has_warnings());

        r.unwrap()
    }

    fn generate_type_parameters_declaration_code(
        &self,
        type_parameters: &[TypeParameter],
    ) -> String {
        if type_parameters.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                itertools::intersperse(
                    type_parameters.iter().map(|x| { x.name_ident.as_str() }),
                    ", "
                )
                .collect::<String>()
            )
        }
    }

    fn generate_type_parameters_constraints_code(
        &self,
        type_parameters: &[TypeParameter],
        extra_constraint: &str,
    ) -> String {
        let mut code = String::new();

        for t in type_parameters.iter() {
            code.push_str(&format!(
                "{}: {},\n",
                t.name_ident.as_str(),
                itertools::intersperse(
                    [extra_constraint].into_iter().chain(
                        t.trait_constraints
                            .iter()
                            .map(|x| x.trait_name.suffix.as_str())
                    ),
                    " + "
                )
                .collect::<String>()
            ));
        }

        if !code.is_empty() {
            code = format!(" where {code}\n");
        }

        code
    }

    fn generate_abi_encode_code(
        &self,
        name: &BaseIdent,
        type_parameters: &[TypeParameter],
        body: String,
    ) -> String {
        let type_parameters_declaration =
            self.generate_type_parameters_declaration_code(type_parameters);
        let type_parameters_constraints =
            self.generate_type_parameters_constraints_code(type_parameters, "AbiEncode");

        let name = name.as_str();

        if body.is_empty() {
            format!("#[allow(dead_code)] impl{type_parameters_declaration} AbiEncode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code)]
                fn abi_encode(self, buffer: Buffer) -> Buffer {{
                    buffer
                }}
            }}")
        } else {
            format!("#[allow(dead_code)] impl{type_parameters_declaration} AbiEncode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code)]
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
        let type_parameters_declaration =
            self.generate_type_parameters_declaration_code(type_parameters);
        let type_parameters_constraints =
            self.generate_type_parameters_constraints_code(type_parameters, "AbiDecode");

        let name = name.as_str();

        if body == "Self {  }" {
            format!("#[allow(dead_code)] impl{type_parameters_declaration} AbiDecode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code)]
                fn abi_decode(ref mut _buffer: BufferReader) -> Self {{
                    {body}
                }}
            }}")
        } else {
            format!("#[allow(dead_code)] impl{type_parameters_declaration} AbiDecode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code)]
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
                field_name = f.name.as_str(),
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
                field_name = f.name.as_str(),
                field_type_name = Self::generate_type(engines, f.type_argument.type_id)?,
            ));
        }

        Some(format!("Self {{ {code} }}"))
    }

    fn generate_abi_decode_enum_body(
        &self,
        engines: &Engines,
        decl: &TyEnumDecl,
    ) -> Option<String> {
        let enum_name = decl.call_path.suffix.as_str();
        let arms = decl.variants.iter()
            .map(|x| {
                let name = x.name.as_str();
                Some(match &*engines.te().get(x.type_argument.type_id) {
                    // unit
                    TypeInfo::Tuple(fields) if fields.is_empty() => {
                        format!("{} => {}::{}, \n", x.tag, enum_name, name)
                    },
                    _ => {
                        let variant_type_name = Self::generate_type(engines, x.type_argument.type_id)?;
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

        let enum_name = decl.call_path.suffix.as_str();
        let arms = decl
            .variants
            .iter()
            .map(|x| {
                let name = x.name.as_str();
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

    pub fn parse_fn_to_ty_ast_node(
        &mut self,
        engines: &Engines,
        program_id: Option<ProgramId>,
        kind: FunctionDeclarationKind,
        code: &str,
    ) -> Result<TyAstNode, Handler> {
        let mut ctx = crate::transform::to_parsed_lang::Context::new(
            crate::BuildTarget::Fuel,
            self.ctx.experimental,
        );

        let handler = Handler::default();

        let item = Self::parse(engines, program_id, code);
        let nodes = crate::transform::to_parsed_lang::item_to_ast_nodes(
            &mut ctx,
            &handler,
            engines,
            item,
            false,
            None,
            Some(kind),
        )
        .unwrap();

        let decl = match nodes[0].content {
            AstNodeContent::Declaration(Declaration::FunctionDeclaration(f)) => f,
            _ => unreachable!("unexpected node"),
        };

        if handler.has_errors() {
            panic!(
                "{:?} {:?}",
                handler,
                program_id
                    .and_then(|x| engines.se().get_source_ids_from_program_id(x))
                    .unwrap()
                    .iter()
                    .map(|x| engines.se().get_file_name(x))
                    .collect::<Vec<_>>()
            );
        }
        assert!(!handler.has_warnings(), "{:?}", handler);

        let ctx = self.ctx.by_ref();
        let r = ctx.scoped_and_namespace(|ctx| {
            TyDecl::type_check(
                &handler,
                ctx,
                parsed::Declaration::FunctionDeclaration(decl),
            )
        });

        // Uncomment this to understand why an entry function was not generated
        //println!("{:#?}", handler);

        let (decl, namespace) = r.map_err(|_| handler.clone())?;

        if handler.has_errors() || matches!(decl, TyDecl::ErrorRecovery(_, _)) {
            Err(handler)
        } else {
            *self.ctx.namespace = namespace;
            Ok(TyAstNode {
                span: decl.span(engines),
                content: ty::TyAstNodeContent::Declaration(decl),
            })
        }
    }

    fn parse_impl_trait_to_ty_ast_node(
        &mut self,
        engines: &Engines,
        program_id: Option<ProgramId>,
        code: &str,
    ) -> Result<TyAstNode, Handler> {
        let mut ctx = crate::transform::to_parsed_lang::Context::new(
            crate::BuildTarget::Fuel,
            self.ctx.experimental,
        );

        let handler = Handler::default();

        let item = Self::parse(engines, program_id, code);
        let nodes = crate::transform::to_parsed_lang::item_to_ast_nodes(
            &mut ctx, &handler, engines, item, false, None, None,
        )
        .unwrap();

        let decl = match nodes[0].content {
            AstNodeContent::Declaration(Declaration::ImplTrait(f)) => f,
            _ => unreachable!("unexpected item"),
        };

        assert!(!handler.has_errors(), "{:?}", handler);

        let ctx = self.ctx.by_ref();
        let r = ctx.scoped_and_namespace(|ctx| {
            TyDecl::type_check(&handler, ctx, Declaration::ImplTrait(decl))
        });

        // Uncomment this to understand why auto impl failed for a type.
        //println!("{:#?}", handler);

        let (decl, namespace) = r.map_err(|_| handler.clone())?;

        if handler.has_errors() || matches!(decl, TyDecl::ErrorRecovery(_, _)) {
            Err(handler)
        } else {
            *self.ctx.namespace = namespace;
            Ok(TyAstNode {
                span: decl.span(engines),
                content: ty::TyAstNodeContent::Declaration(decl),
            })
        }
    }

    // Auto implements AbiEncode and AbiDecode for structs and returns their `AstNode`s.
    fn auto_impl_struct(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> Option<(Option<TyAstNode>, Option<TyAstNode>)> {
        if matches!(self.ctx.namespace.root().module.read(engines, |m| m.name.clone()).as_ref(), Some(x) if x.as_str() == "core")
        {
            return Some((None, None));
        }

        let implementing_for_decl_ref = decl.to_struct_ref(&Handler::default(), engines).unwrap();
        let struct_decl = self.ctx.engines().de().get(implementing_for_decl_ref.id());

        let program_id = struct_decl.span().source_id().map(|sid| sid.program_id());

        let abi_encode_body = self.generate_abi_encode_struct_body(engines, &struct_decl);
        let abi_encode_code = self.generate_abi_encode_code(
            struct_decl.name(),
            &struct_decl.type_parameters,
            abi_encode_body,
        );
        let abi_encode_node =
            self.parse_impl_trait_to_ty_ast_node(engines, program_id, &abi_encode_code);

        let abi_decode_body = self.generate_abi_decode_struct_body(engines, &struct_decl);
        let abi_decode_code = self.generate_abi_decode_code(
            struct_decl.name(),
            &struct_decl.type_parameters,
            abi_decode_body?,
        );
        let abi_decode_node =
            self.parse_impl_trait_to_ty_ast_node(engines, program_id, &abi_decode_code);

        Some((abi_encode_node.ok(), abi_decode_node.ok()))
    }

    fn auto_impl_enum(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> Option<(Option<TyAstNode>, Option<TyAstNode>)> {
        if matches!(self.ctx.namespace.root().module.read(engines, |m| m.name.clone()).as_ref(), Some(x) if x.as_str() == "core")
        {
            return Some((None, None));
        }

        let enum_decl_id = decl.to_enum_id(&Handler::default(), engines).unwrap();
        let enum_decl = self.ctx.engines().de().get(&enum_decl_id);

        let program_id = enum_decl.span().source_id().map(|sid| sid.program_id());

        let abi_encode_body = self.generate_abi_encode_enum_body(engines, &enum_decl);
        let abi_encode_code = self.generate_abi_encode_code(
            enum_decl.name(),
            &enum_decl.type_parameters,
            abi_encode_body,
        );
        let abi_encode_node =
            self.parse_impl_trait_to_ty_ast_node(engines, program_id, &abi_encode_code);

        let abi_decode_body = self.generate_abi_decode_enum_body(engines, &enum_decl);
        let abi_decode_code = self.generate_abi_decode_code(
            enum_decl.name(),
            &enum_decl.type_parameters,
            abi_decode_body?,
        );
        let abi_decode_node =
            self.parse_impl_trait_to_ty_ast_node(engines, program_id, &abi_decode_code);

        Some((abi_encode_node.ok(), abi_decode_node.ok()))
    }

    pub fn generate(
        &mut self,
        engines: &Engines,
        decl: &ty::TyDecl,
    ) -> (Option<TyAstNode>, Option<TyAstNode>) {
        match decl {
            TyDecl::StructDecl(_) => self.auto_impl_struct(engines, decl).unwrap_or((None, None)),
            TyDecl::EnumDecl(_) => self.auto_impl_enum(engines, decl).unwrap_or((None, None)),
            _ => (None, None),
        }
    }

    fn generate_type(engines: &Engines, type_id: TypeId) -> Option<String> {
        let name = match &*engines.te().get(type_id) {
            TypeInfo::UnknownGeneric { name, .. } => name.to_string(),
            TypeInfo::Placeholder(type_param) => type_param.name_ident.to_string(),
            TypeInfo::StringSlice => "str".into(),
            TypeInfo::StringArray(x) => format!("str[{}]", x.val()),
            TypeInfo::UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
                IntegerBits::V256 => "u256",
            }
            .into(),
            TypeInfo::Boolean => "bool".into(),
            TypeInfo::Custom {
                qualified_call_path: call_path,
                ..
            } => call_path.call_path.suffix.to_string(),
            TypeInfo::Tuple(fields) => {
                if fields.is_empty() {
                    return Some("()".into());
                }
                let field_strs = fields
                    .iter()
                    .map(|field| Self::generate_type(engines, field.type_id))
                    .collect::<Option<Vec<String>>>()?;
                format!("({},)", field_strs.join(", "))
            }
            TypeInfo::B256 => "b256".into(),
            TypeInfo::Enum(decl_id) => {
                let decl = engines.de().get_enum(decl_id);

                let type_parameters = decl
                    .type_parameters
                    .iter()
                    .map(|x| Self::generate_type(engines, x.type_id))
                    .collect::<Option<Vec<String>>>()?
                    .join(", ");

                let type_parameters = if !type_parameters.is_empty() {
                    format!("<{type_parameters}>")
                } else {
                    type_parameters
                };

                format!("{}{type_parameters}", decl.call_path.suffix.as_str())
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = engines.de().get(decl_ref.id());

                let type_parameters = decl
                    .type_parameters
                    .iter()
                    .map(|x| Self::generate_type(engines, x.type_id))
                    .collect::<Option<Vec<String>>>()?
                    .join(", ");

                let type_parameters = if !type_parameters.is_empty() {
                    format!("<{type_parameters}>")
                } else {
                    type_parameters
                };

                format!("{}{type_parameters}", decl.call_path.suffix.as_str())
            }
            TypeInfo::Array(elem_ty, count) => {
                format!(
                    "[{}; {}]",
                    Self::generate_type(engines, elem_ty.type_id)?,
                    count.val()
                )
            }
            TypeInfo::RawUntypedPtr => "raw_ptr".into(),
            TypeInfo::RawUntypedSlice => "raw_slice".into(),
            TypeInfo::Alias { name, .. } => name.to_string(),
            _ => return None,
        };

        Some(name)
    }

    pub(crate) fn generate_contract_entry(
        &mut self,
        engines: &Engines,
        program_id: Option<ProgramId>,
        contract_fns: &[DeclRef<DeclId<TyFunctionDecl>>],
        fallback_fn: Option<DeclId<TyFunctionDecl>>,
        handler: &Handler,
    ) -> Result<TyAstNode, ErrorEmitted> {
        let mut code = String::new();

        let mut reads = false;
        let mut writes = false;

        for r in contract_fns {
            let decl = engines.de().get(r.id());

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
                .map(|x| Self::generate_type(engines, x.type_argument.type_id))
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

            let Some(return_type) = Self::generate_type(engines, decl.return_type.type_id) else {
                let err = handler.emit_err(CompileError::UnknownType {
                    span: Span::dummy(),
                });
                return Err(err);
            };

            let method_name = decl.name.as_str();

            code.push_str(&format!("if _method_name == \"{method_name}\" {{\n"));

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

        let fallback = if let Some(fallback_fn) = fallback_fn {
            let fallback_fn = engines.de().get(&fallback_fn);
            let Some(return_type) = Self::generate_type(engines, fallback_fn.return_type.type_id)
            else {
                let err = handler.emit_err(CompileError::UnknownType {
                    span: Span::dummy(),
                });
                return Err(err);
            };
            let method_name = fallback_fn.name.as_str();
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
            format!("__revert({});", MISMATCHED_SELECTOR_REVERT_CODE)
        };

        let att: String = match (reads, writes) {
            (true, true) => "#[storage(read, write)]",
            (true, false) => "#[storage(read)]",
            (false, true) => "#[storage(write)]",
            (false, false) => "",
        }
        .into();

        let code = format!(
            "{att} pub fn __entry() {{
            let mut _buffer = BufferReader::from_second_parameter();
            let _method_name = decode_first_param::<str>();
            {code}
            {fallback}
        }}"
        );

        let entry_fn = self.parse_fn_to_ty_ast_node(
            engines,
            program_id,
            FunctionDeclarationKind::Entry,
            &code,
        );

        match entry_fn {
            Ok(entry_fn) => Ok(entry_fn),
            Err(gen_handler) => {
                Self::check_impl_is_missing(handler, &gen_handler);
                Self::check_core_is_missing(handler, &gen_handler);
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
        let program_id = decl.span.source_id().map(|sid| sid.program_id());

        let Some(args_types) = decl
            .parameters
            .iter()
            .map(|x| Self::generate_type(engines, x.type_argument.type_id))
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
            program_id,
            FunctionDeclarationKind::Entry,
            &code,
        );

        match entry_fn {
            Ok(entry_fn) => Ok(entry_fn),
            Err(gen_handler) => {
                Self::check_impl_is_missing(handler, &gen_handler);
                Self::check_core_is_missing(handler, &gen_handler);
                Err(gen_handler.emit_err(CompileError::CouldNotGenerateEntry {
                    span: Span::dummy(),
                }))
            }
        }
    }

    // Check core is missing and give a more user-friendly error message.
    fn check_core_is_missing(handler: &Handler, gen_handler: &Handler) {
        let encode_not_found = gen_handler
            .find_error(|x| matches!(x, CompileError::SymbolNotFound { .. }))
            .is_some();
        if encode_not_found {
            handler.emit_err(CompileError::CouldNotGenerateEntryMissingCore {
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
        let program_id = decl.span.source_id().map(|sid| sid.program_id());

        let Some(args_types) = decl
            .parameters
            .iter()
            .map(|x| Self::generate_type(engines, x.type_argument.type_id))
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

        let Some(return_type) = Self::generate_type(engines, decl.return_type.type_id) else {
            let err = handler.emit_err(CompileError::UnknownType {
                span: Span::dummy(),
            });
            return Err(err);
        };

        let code = if args_types == "()" {
            format!(
                "pub fn __entry() -> raw_slice {{
                let result: {return_type} = main(); 
                encode::<{return_type}>(result)
            }}"
            )
        } else {
            format!(
                "pub fn __entry() -> raw_slice {{
                let args: {args_types} = decode_script_data::<{args_types}>();
                let result: {return_type} = main({expanded_args}); 
                encode::<{return_type}>(result)
            }}"
            )
        };

        let entry_fn = self.parse_fn_to_ty_ast_node(
            engines,
            program_id,
            FunctionDeclarationKind::Entry,
            &code,
        );

        match entry_fn {
            Ok(entry_fn) => Ok(entry_fn),
            Err(gen_handler) => {
                Self::check_core_is_missing(handler, &gen_handler);
                Self::check_impl_is_missing(handler, &gen_handler);
                Err(gen_handler.emit_err(CompileError::CouldNotGenerateEntry {
                    span: Span::dummy(),
                }))
            }
        }
    }
}
