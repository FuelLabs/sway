use crate::{
    decl_engine::{DeclEngineGet, DeclId, DeclRef},
    language::{
        parsed::{self, AstNodeContent, Declaration, FunctionDeclarationKind},
        ty::{self, TyAstNode, TyDecl, TyEnumDecl, TyFunctionDecl, TyStructDecl},
        Purity,
    },
    semantic_analysis::TypeCheckContext,
    Engines, TypeId, TypeInfo, TypeParameter,
};
use itertools::Itertools;
use sway_error::handler::Handler;
use sway_parse::Parse;
use sway_types::{integer_bits::IntegerBits, BaseIdent, Named, Span};

/// Contains all information needed to implement AbiEncode
pub struct AutoImplAbiEncodeContext<'a, 'b>
where
    'a: 'b,
{
    ctx: &'b mut TypeCheckContext<'a>,
}

impl<'a, 'b> AutoImplAbiEncodeContext<'a, 'b>
where
    'a: 'b,
{
    pub fn new(ctx: &'b mut TypeCheckContext<'a>) -> Option<Self> {
        Some(Self { ctx })
    }

    pub fn parse<T>(input: &str) -> T
    where
        T: Parse,
    {
        // println!("[{}]", input);
        let handler = <_>::default();
        let ts =
            sway_parse::lex(&handler, &std::sync::Arc::from(input), 0, input.len(), None).unwrap();
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
                fn abi_encode(self, ref mut _buffer: Buffer) {{
                }}
            }}")
        } else {
            format!("#[allow(dead_code)] impl{type_parameters_declaration} AbiEncode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
                #[allow(dead_code)]
                fn abi_encode(self, ref mut buffer: Buffer) {{
                    {body}
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
                "self.{field_name}.abi_encode(buffer);\n",
                field_name = f.name.as_str(),
            ));
        }

        code
    }

    fn generate_abi_decode_struct_body(&self, engines: &Engines, decl: &TyStructDecl) -> String {
        let mut code = String::new();
        for f in decl.fields.iter() {
            code.push_str(&format!(
                "{field_name}: buffer.decode::<{field_type_name}>(),",
                field_name = f.name.as_str(),
                field_type_name = Self::generate_type(engines, f.type_argument.type_id),
            ));
        }

        format!("Self {{ {code} }}")
    }

    fn generate_abi_decode_enum_body(&self, engines: &Engines, decl: &TyEnumDecl) -> String {
        let enum_name = decl.call_path.suffix.as_str();
        let arms = decl.variants.iter()
            .map(|x| {
                let name = x.name.as_str();
                if engines.te().get(x.type_argument.type_id).is_unit() {
                    format!("{} => {}::{}, \n", x.tag, enum_name, name)
                } else {
                    let variant_type_name = Self::generate_type(engines, x.type_argument.type_id);
                    format!("{tag_value} => {enum_name}::{variant_name}(buffer.decode::<{variant_type}>()), \n", 
                        tag_value = x.tag,
                        enum_name = enum_name,
                        variant_name = name,
                        variant_type = variant_type_name
                    )
                }
            })
        .collect::<String>();

        use std::fmt::Write;
        let mut code = String::new();
        writeln!(&mut code, "let variant: u64 = buffer.decode::<u64>();").unwrap();
        writeln!(&mut code, "match variant {{ {arms} _ => __revert(0), }}").unwrap();

        code
    }

    fn generate_abi_encode_enum_body(&self, engines: &Engines, decl: &TyEnumDecl) -> String {
        let enum_name = decl.call_path.suffix.as_str();
        let arms = decl
            .variants
            .iter()
            .map(|x| {
                let name = x.name.as_str();
                if engines.te().get(x.type_argument.type_id).is_unit() {
                    format!(
                        "{enum_name}::{variant_name} => {{
                        {tag_value}u64.abi_encode(buffer);
                    }}, \n",
                        tag_value = x.tag,
                        enum_name = enum_name,
                        variant_name = name
                    )
                } else {
                    format!(
                        "{enum_name}::{variant_name}(value) => {{
                        {tag_value}u64.abi_encode(buffer);
                        value.abi_encode(buffer);
                    }}, \n",
                        tag_value = x.tag,
                        enum_name = enum_name,
                        variant_name = name,
                    )
                }
            })
            .collect::<String>();

        format!("match self {{ {arms} }}")
    }

    pub fn parse_item_fn_to_typed_ast_node(
        &mut self,
        engines: &Engines,
        kind: FunctionDeclarationKind,
        code: &str,
    ) -> Option<TyAstNode> {
        let mut ctx = crate::transform::to_parsed_lang::Context::new(
            crate::BuildTarget::Fuel,
            self.ctx.experimental,
        );

        let handler = Handler::default();

        let item = Self::parse(code);
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
            _ => todo!(),
        };

        assert!(!handler.has_errors(), "{:?} {}", handler, code);
        assert!(!handler.has_warnings(), "{:?}", handler);

        let ctx = self.ctx.by_ref();
        let decl = TyDecl::type_check(
            &handler,
            ctx,
            parsed::Declaration::FunctionDeclaration(decl),
        )
        .unwrap();

        assert!(!handler.has_errors(), "{:?} {}", handler, code);
        assert!(!handler.has_warnings(), "{:?}", handler);

        Some(TyAstNode {
            content: ty::TyAstNodeContent::Declaration(decl),
            span: Span::dummy(),
        })
    }

    fn parse_item_impl_to_typed_ast_node(
        &mut self,
        engines: &Engines,
        code: &str,
    ) -> Option<TyAstNode> {
        let mut ctx = crate::transform::to_parsed_lang::Context::new(
            crate::BuildTarget::Fuel,
            self.ctx.experimental,
        );

        let handler = Handler::default();

        let item = Self::parse(code);
        let nodes = crate::transform::to_parsed_lang::item_to_ast_nodes(
            &mut ctx, &handler, engines, item, false, None, None,
        )
        .unwrap();

        let decl = match nodes[0].content {
            AstNodeContent::Declaration(Declaration::ImplTrait(f)) => f,
            _ => todo!(),
        };

        assert!(!handler.has_errors(), "{:?}", handler);

        let ctx = self.ctx.by_ref();
        let decl = TyDecl::type_check(&handler, ctx, Declaration::ImplTrait(decl)).unwrap();

        if handler.has_errors() {
            None
        } else {
            Some(TyAstNode {
                content: ty::TyAstNodeContent::Declaration(decl),
                span: Span::dummy(),
            })
        }
    }

    // Auto implements AbiEncode and AbiDecode for structs and returns their `AstNode`s.
    fn auto_impl_struct(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> (Option<TyAstNode>, Option<TyAstNode>) {
        if matches!(self.ctx.namespace.root().module.name.as_ref(), Some(x) if x.as_str() == "core")
        {
            return (None, None);
        }

        let implementing_for_decl_ref = decl.get_struct_decl_ref().unwrap();
        let struct_decl = self.ctx.engines().de().get(implementing_for_decl_ref.id());

        let abi_encode_body = self.generate_abi_encode_struct_body(engines, &struct_decl);
        let abi_encode_code = self.generate_abi_encode_code(
            struct_decl.name(),
            &struct_decl.type_parameters,
            abi_encode_body,
        );
        let abi_encode_node = self.parse_item_impl_to_typed_ast_node(engines, &abi_encode_code);

        let abi_decode_body = self.generate_abi_decode_struct_body(engines, &struct_decl);
        let abi_decode_code = self.generate_abi_decode_code(
            struct_decl.name(),
            &struct_decl.type_parameters,
            abi_decode_body,
        );
        let abi_decode_node = self.parse_item_impl_to_typed_ast_node(engines, &abi_decode_code);

        (abi_encode_node, abi_decode_node)
    }

    fn auto_impl_enum(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> (Option<TyAstNode>, Option<TyAstNode>) {
        if matches!(self.ctx.namespace.root().module.name.as_ref(), Some(x) if x.as_str() == "core")
        {
            return (None, None);
        }

        let enum_decl_ref = decl.get_enum_decl_ref().unwrap();
        let enum_decl = self.ctx.engines().de().get(enum_decl_ref.id());

        let abi_decode_body = self.generate_abi_encode_enum_body(engines, &enum_decl);
        let abi_decode_code = self.generate_abi_encode_code(
            enum_decl.name(),
            &enum_decl.type_parameters,
            abi_decode_body,
        );
        let abi_encode_node = self.parse_item_impl_to_typed_ast_node(engines, &abi_decode_code);

        let abi_decode_body = self.generate_abi_decode_enum_body(engines, &enum_decl);
        let abi_decode_code = self.generate_abi_decode_code(
            enum_decl.name(),
            &enum_decl.type_parameters,
            abi_decode_body,
        );
        let abi_decode_node = self.parse_item_impl_to_typed_ast_node(engines, &abi_decode_code);

        (abi_encode_node, abi_decode_node)
    }

    pub fn generate(
        &mut self,
        engines: &Engines,
        decl: &ty::TyDecl,
    ) -> (Option<TyAstNode>, Option<TyAstNode>) {
        match decl {
            TyDecl::StructDecl(_) => self.auto_impl_struct(engines, decl),
            TyDecl::EnumDecl(_) => self.auto_impl_enum(engines, decl),
            _ => (None, None),
        }
    }

    fn generate_type(engines: &Engines, type_id: TypeId) -> String {
        match &*engines.te().get(type_id) {
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
                    return "()".into();
                }
                let field_strs = fields
                    .iter()
                    .map(|field| Self::generate_type(engines, field.type_id))
                    .collect::<Vec<_>>();
                format!("({},)", field_strs.join(", "))
            }
            TypeInfo::B256 => "b256".into(),
            TypeInfo::Enum(decl_ref) => {
                let decl = engines.de().get(decl_ref.id());

                let type_parameters = decl
                    .type_parameters
                    .iter()
                    .map(|x| Self::generate_type(engines, x.type_id))
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
                    Self::generate_type(engines, elem_ty.type_id),
                    count.val()
                )
            }
            TypeInfo::RawUntypedPtr => "raw_ptr".into(),
            TypeInfo::RawUntypedSlice => "raw_slice".into(),
            TypeInfo::Alias { name, .. } => name.to_string(),
            _ => todo!(),
        }
    }

    pub(crate) fn generate_contract_entry(
        &mut self,
        engines: &Engines,
        contract_fns: &[DeclRef<DeclId<TyFunctionDecl>>],
    ) -> Option<TyAstNode> {
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

            let args_types = itertools::intersperse(
                decl.parameters
                    .iter()
                    .map(|x| Self::generate_type(engines, x.type_argument.type_id)),
                ", ".into(),
            )
            .collect::<String>();

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

            let return_type = Self::generate_type(engines, decl.return_type.type_id);

            let method_name = decl.name.as_str();

            if args_types == "()" {
                code.push_str(&format!("if method_name == \"{method_name}\" {{
                    let result_{method_name}: raw_slice = encode::<{return_type}>(__contract_entry_{method_name}());
                    __contract_ret(result_{method_name}.ptr(), result_{method_name}.len::<u8>());
                }}\n"));
            } else {
                code.push_str(&format!("if method_name == \"{method_name}\" {{
                    let args = decode_second_param::<{args_types}>();
                    let result_{method_name}: raw_slice = encode::<{return_type}>(__contract_entry_{method_name}({expanded_args}));
                    __contract_ret(result_{method_name}.ptr(), result_{method_name}.len::<u8>());
                }}\n"));
            }
        }

        let att: String = match (reads, writes) {
            (true, true) => "#[storage(read, write)]",
            (true, false) => "#[storage(read)]",
            (false, true) => "#[storage(write)]",
            (false, false) => "",
        }
        .into();

        let code = format!(
            "{att}
        pub fn __entry() {{
            let method_name = decode_first_param::<str>();
            __log(method_name);
            {code}
        }}"
        );

        self.parse_item_fn_to_typed_ast_node(engines, FunctionDeclarationKind::Entry, &code)
    }

    pub(crate) fn generate_predicate_entry(
        &mut self,
        engines: &Engines,
        decl: &TyFunctionDecl,
    ) -> Option<TyAstNode> {
        let args_types = itertools::intersperse(
            decl.parameters
                .iter()
                .map(|x| Self::generate_type(engines, x.type_argument.type_id)),
            ", ".into(),
        )
        .collect::<String>();

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

        let code = format!(
            "pub fn __entry() -> bool {{
            let args = decode_script_data::<{args_types}>();
            main({expanded_args})
        }}"
        );
        self.parse_item_fn_to_typed_ast_node(engines, FunctionDeclarationKind::Entry, &code)
    }

    pub(crate) fn generate_script_entry(
        &mut self,
        engines: &Engines,
        decl: &TyFunctionDecl,
    ) -> Option<TyAstNode> {
        let args_types = itertools::intersperse(
            decl.parameters
                .iter()
                .map(|x| Self::generate_type(engines, x.type_argument.type_id)),
            ", ".into(),
        )
        .collect::<String>();

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

        let return_type = Self::generate_type(engines, decl.return_type.type_id);

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
                let args = decode_script_data::<{args_types}>();
                let result: {return_type} = main({expanded_args}); 
                encode::<{return_type}>(result)
            }}"
            )
        };

        self.parse_item_fn_to_typed_ast_node(engines, FunctionDeclarationKind::Entry, &code)
    }
}
