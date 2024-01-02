use std::{any::Any, ops::Deref};

use crate::{
    decl_engine::{parsed_engine::{ParsedDeclEngineGet, ParsedDeclEngineInsert}, DeclEngineGet, DeclId, DeclRef},
    language::{
        parsed::{
            self, AmbiguousPathExpression, AmbiguousSuffix, AstNode, AstNodeContent, CodeBlock, Declaration, DelineatedPathExpression, Expression, ExpressionKind, FunctionApplicationExpression, FunctionDeclaration, FunctionDeclarationKind, FunctionParameter, ImplItem, ImplTrait, MatchBranch, MatchExpression, MethodApplicationExpression, MethodName, QualifiedPathRootTypes, Scrutinee, StructExpression, StructExpressionField
        }, ty::{self, TyAstNode, TyDecl, TyEnumDecl, TyFunctionDecl, TyModule, TyStructDecl}, CallPath, Literal, Purity, QualifiedCallPath
    },
    semantic_analysis::{type_check_context::EnforceTypeArguments, TypeCheckContext},
    transform::{to_parsed_lang::convert_parse_tree, AttributesMap},
    Engines, TraitConstraint, TypeArgs, TypeArgument, TypeBinding, TypeCheckTypeBinding, TypeId,
    TypeInfo, TypeParameter,
};
use itertools::Itertools;
use sway_error::handler::Handler;
use sway_ir::Type;
use sway_parse::Parse;
use sway_types::{integer_bits::IntegerBits, BaseIdent, Ident, Named, Span};

/// Contains all information needed to implement AbiEncode
pub struct AutoImplAbiEncodeContext<'a, 'b> {
    ctx: &'b mut TypeCheckContext<'a>,
    buffer_type_id: TypeId,
    buffer_reader_type_id: TypeId,
    abi_encode_call_path: CallPath,
    abi_decode_call_path: CallPath,
}

impl<'a, 'b> AutoImplAbiEncodeContext<'a, 'b> {
    /// This function fails if the context does not have access to the "core" module
    pub fn new(ctx: &'b mut TypeCheckContext<'a>) -> Option<Self> {
        let buffer_type_id =
            Self::resolve_type(ctx, CallPath::absolute(&["core", "codec", "Buffer"]))?;
        let buffer_reader_type_id =
            Self::resolve_type(ctx, CallPath::absolute(&["core", "codec", "BufferReader"]))?;
        Some(Self {
            ctx,
            buffer_type_id,
            buffer_reader_type_id,
            abi_encode_call_path: CallPath::absolute(&["core", "codec", "AbiEncode"]),
            abi_decode_call_path: CallPath::absolute(&["core", "codec", "AbiDecode"]),
        })
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

    fn resolve_type(ctx: &mut TypeCheckContext<'_>, call_path: CallPath) -> Option<TypeId> {
        let handler = Handler::default();
        let buffer_type_id = ctx.engines.te().insert(
            ctx.engines,
            TypeInfo::Custom {
                qualified_call_path: QualifiedCallPath {
                    call_path,
                    qualified_path_root: None,
                },
                type_arguments: None,
                root_type_id: None,
            },
            None,
        );
        let buffer_type_id = ctx
            .resolve_type(
                &handler,
                buffer_type_id,
                &Span::dummy(),
                EnforceTypeArguments::No,
                None,
            )
            .ok()?;
        Some(buffer_type_id)
    }

    fn import_core_codec(&mut self) -> bool {
        // Check if the compilation context has acces to the
        // core library.
        let handler = Handler::default();
        let _ = self.ctx.star_import(
            &handler,
            &[
                Ident::new_no_span("core".into()),
                Ident::new_no_span("codec".into()),
            ],
            true,
        );

        !handler.has_errors()
    }

    fn build_implementing_for_with_type_parameters(
        &mut self,
        suffix: BaseIdent,
        type_parameters: &Vec<(BaseIdent, TypeParameter)>,
    ) -> (TypeId, TypeArgument) {
        let qualified_call_path = QualifiedCallPath {
            call_path: CallPath {
                prefixes: vec![],
                suffix,
                is_absolute: false,
            },
            qualified_path_root: None,
        };

        let type_arguments = if type_parameters.is_empty() {
            None
        } else {
            Some(
                type_parameters
                    .iter()
                    .map(|(_, x)| {
                        let type_id = self.ctx.engines().te().insert(
                            self.ctx.engines(),
                            TypeInfo::Custom {
                                qualified_call_path: QualifiedCallPath {
                                    call_path: CallPath {
                                        prefixes: vec![],
                                        suffix: x.name_ident.clone(),
                                        is_absolute: false,
                                    },
                                    qualified_path_root: None,
                                },
                                type_arguments: None,
                                root_type_id: None,
                            },
                            None,
                        );

                        TypeArgument {
                            type_id,
                            initial_type_id: type_id,
                            span: Span::dummy(),
                            call_path_tree: None,
                        }
                    })
                    .collect(),
            )
        };

        let type_id = self.ctx.engines.te().insert(
            self.ctx.engines,
            TypeInfo::Custom {
                qualified_call_path,
                type_arguments,
                root_type_id: None,
            },
            None,
        );

        let implementing_for = TypeArgument {
            type_id,
            initial_type_id: type_id,
            span: Span::dummy(),
            call_path_tree: None,
        };

        (type_id, implementing_for)
    }

    // This duplicates the "decl" type parameters and adds an extra constraint, for example:
    //
    // ```
    // enum E<T> where T: SomeTrait {
    //
    // }
    // ```
    //
    // This will return `T: SomeTrait + ExtraConstraint`
    fn duplicate_type_parameters_with_extra_constraint(
        &mut self,
        type_parameters: &[TypeParameter],
        constraint_name: &str,
    ) -> Vec<(BaseIdent, TypeParameter)> {
        type_parameters
            .iter()
            .map(|type_parameter| {
                let type_id = self.ctx.engines.te().insert(
                    self.ctx.engines(),
                    TypeInfo::Custom {
                        qualified_call_path: QualifiedCallPath {
                            call_path: CallPath {
                                prefixes: vec![],
                                suffix: Ident::new_no_span(
                                    type_parameter.name_ident.as_str().into(),
                                ),
                                is_absolute: false,
                            },
                            qualified_path_root: None,
                        },
                        type_arguments: None,
                        root_type_id: None,
                    },
                    None,
                );

                let mut trait_constraints: Vec<_> = type_parameter
                    .trait_constraints
                    .iter()
                    .map(|x| TraitConstraint {
                        trait_name: CallPath {
                            prefixes: vec![],
                            suffix: Ident::new_no_span(x.trait_name.suffix.as_str().into()),
                            is_absolute: false,
                        },
                        type_arguments: x
                            .type_arguments
                            .iter()
                            .map(|x| {
                                let name = match &*self.ctx.engines.te().get(x.type_id) {
                                    TypeInfo::Custom {
                                        qualified_call_path,
                                        ..
                                    } => Ident::new_no_span(
                                        qualified_call_path.call_path.suffix.as_str().into(),
                                    ),
                                    _ => todo!(),
                                };

                                let type_id = self.ctx.engines.te().insert(
                                    self.ctx.engines(),
                                    TypeInfo::Custom {
                                        qualified_call_path: QualifiedCallPath {
                                            call_path: CallPath {
                                                prefixes: vec![],
                                                suffix: name,
                                                is_absolute: false,
                                            },
                                            qualified_path_root: None,
                                        },
                                        type_arguments: None,
                                        root_type_id: None,
                                    },
                                    None,
                                );

                                TypeArgument {
                                    type_id,
                                    initial_type_id: type_id,
                                    span: Span::dummy(),
                                    call_path_tree: None,
                                }
                            })
                            .collect(),
                    })
                    .collect();

                trait_constraints.push(TraitConstraint {
                    trait_name: CallPath {
                        prefixes: vec![],
                        suffix: Ident::new_no_span(constraint_name.into()),
                        is_absolute: false,
                    },
                    type_arguments: vec![],
                });

                (
                    type_parameter.name_ident.clone(),
                    TypeParameter {
                        type_id,
                        initial_type_id: type_id,
                        name_ident: Ident::new_no_span(type_parameter.name_ident.as_str().into()),
                        trait_constraints,
                        trait_constraints_span: Span::dummy(),
                        is_from_parent: false,
                    },
                )
            })
            .collect()
    }

    fn auto_impl_abi_encode(
        &mut self,
        type_parameters: &[TypeParameter],
        suffix: BaseIdent,
        unit_type_id: TypeId,
        engines: &Engines,
        contents: Vec<AstNode>,
    ) -> Option<TyAstNode> {
        let type_parameters =
            self.duplicate_type_parameters_with_extra_constraint(type_parameters, "AbiEncode");
        let (implementing_for_type_id, implementing_for) =
            self.build_implementing_for_with_type_parameters(suffix, &type_parameters);

        let fn_trait_item = self.build_abi_encode_fn_trait_item(
            contents,
            implementing_for_type_id,
            unit_type_id,
            engines,
        );

        let type_parameters = type_parameters.into_iter().map(|x| x.1).collect();
        self.type_check_impl(type_parameters, implementing_for, fn_trait_item, engines)
    }

    fn type_check_impl(
        &mut self,
        type_parameters: Vec<TypeParameter>,
        implementing_for: TypeArgument,
        fn_abi_encode_trait_item: crate::decl_engine::parsed_id::ParsedDeclId<FunctionDeclaration>,
        engines: &Engines,
    ) -> Option<TyAstNode> {
        let impl_abi_encode_for_enum = ImplTrait {
            impl_type_parameters: type_parameters,
            trait_name: self.abi_encode_call_path.clone(),
            trait_type_arguments: vec![],
            implementing_for,
            items: vec![ImplItem::Fn(fn_abi_encode_trait_item)],
            block_span: Span::dummy(),
        };
        let impl_abi_encode_for_enum = engines.pe().insert(impl_abi_encode_for_enum);
        let impl_abi_encode_for_enum = Declaration::ImplTrait(impl_abi_encode_for_enum);
        let handler = Handler::default();
        let impl_abi_encode_for_enum = TyAstNode::type_check(
            &handler,
            self.ctx.by_ref(),
            AstNode {
                content: AstNodeContent::Declaration(impl_abi_encode_for_enum),
                span: Span::dummy(),
            },
        )
        .ok()?;

        assert!(!handler.has_errors(), "{:?}", handler);
        assert!(!handler.has_warnings(), "{:?}", handler);

        Some(impl_abi_encode_for_enum)
    }

    fn build_abi_encode_fn_trait_item(
        &mut self,
        contents: Vec<AstNode>,
        implementing_for_type_id: TypeId,
        unit_type_id: TypeId,
        engines: &Engines,
    ) -> crate::decl_engine::parsed_id::ParsedDeclId<FunctionDeclaration> {
        let fn_abi_encode_trait_item = FunctionDeclaration {
            purity: crate::language::Purity::Pure,
            attributes: AttributesMap::default(),
            name: Ident::new_no_span("abi_encode".into()),
            visibility: crate::language::Visibility::Public,
            body: CodeBlock {
                contents,
                whole_block_span: Span::dummy(),
            },
            parameters: vec![
                FunctionParameter {
                    name: Ident::new_no_span("self".into()),
                    is_reference: false,
                    is_mutable: false,
                    mutability_span: Span::dummy(),
                    type_argument: TypeArgument {
                        type_id: implementing_for_type_id,
                        initial_type_id: implementing_for_type_id,
                        span: Span::dummy(),
                        call_path_tree: None,
                    },
                },
                FunctionParameter {
                    name: Ident::new_no_span("buffer".into()),
                    is_reference: true,
                    is_mutable: true,
                    mutability_span: Span::dummy(),
                    type_argument: TypeArgument {
                        type_id: self.buffer_type_id,
                        initial_type_id: self.buffer_type_id,
                        span: Span::dummy(),
                        call_path_tree: None,
                    },
                },
            ],
            span: Span::dummy(),
            return_type: TypeArgument {
                type_id: unit_type_id,
                initial_type_id: unit_type_id,
                span: Span::dummy(),
                call_path_tree: None,
            },
            type_parameters: vec![],
            where_clause: vec![],
            kind: FunctionDeclarationKind::Default,
        };
        let fn_abi_encode_trait_item = engines.pe().insert(fn_abi_encode_trait_item);
        fn_abi_encode_trait_item
    }

    // Check if a struct can implement AbiEncode and AbiDecode
    fn can_auto_impl_struct(&mut self, engines: &Engines, decl: &TyDecl) -> (bool, bool) {
        // skip module "core"
        // Because of ordering, we cannot guarantee auto impl
        // for structs inside "core"
        if matches!(self.ctx.namespace.root().module.name.as_ref(), Some(x) if x.as_str() == "core")
        {
            return (false, false);
        }

        let Some(decl_ref) = decl.get_struct_decl_ref() else {
            return (false, false);
        };
        let struct_decl = self.ctx.engines().de().get(decl_ref.id());

        // Do not support types with generic constraints
        // because this generates a circular impl trait
        if struct_decl.type_parameters.iter().any(|x| {
            x.trait_constraints
                .iter()
                .any(|c| !c.type_arguments.is_empty())
        }) {
            return (false, false);
        }

        let all_fields_are_abi_encode = struct_decl.fields.iter().all(|field| {
            let r = self.ctx.check_type_impls_traits(
                field.type_argument.type_id,
                &[TraitConstraint {
                    trait_name: self.abi_encode_call_path.clone(),
                    type_arguments: vec![],
                }],
            );

            if r {
                return true;
            }

            let field_type_id = self.apply_constraints(engines, field.type_argument.type_id, self.abi_encode_call_path.clone());
            let r = self.ctx.check_type_impls_traits(
                field_type_id,
                &[TraitConstraint {
                    trait_name: self.abi_encode_call_path.clone(),
                    type_arguments: vec![],
                }],
            );

            // dbg!(&field.name, engines.help_out(field_type_id), r);

            r
        });

        let all_fields_are_abi_decode = struct_decl.fields.iter().all(|field| {
            let r = self.ctx.check_type_impls_traits(
                field.type_argument.type_id,
                &[TraitConstraint {
                    trait_name: self.abi_decode_call_path.clone(),
                    type_arguments: vec![],
                }],
            );

            if r {
                return true;
            }

            let field_type_id = self.apply_constraints(engines, field.type_argument.type_id, self.abi_decode_call_path.clone());
            let r = self.ctx.check_type_impls_traits(
                field_type_id,
                &[TraitConstraint {
                    trait_name: self.abi_decode_call_path.clone(),
                    type_arguments: vec![],
                }],
            );

            // dbg!(&field.name, engines.help_out(field_type_id), r);

            r
        });

        (all_fields_are_abi_encode, all_fields_are_abi_decode)
    }

    fn get_type_parameter(
        &self,
        engines: &Engines,
        type_id: TypeId,
        type_parameters: &[(BaseIdent, TypeParameter)],
    ) -> TypeId {
        match &*engines.te().get(type_id) {
            TypeInfo::UnknownGeneric { name, .. } => {
                type_parameters
                    .iter()
                    .find(|x| x.0 == *name)
                    .unwrap()
                    .1
                    .type_id
            }
            _ => type_id,
        }
    }

    fn generate_type_parameters_declaration_code(&self, type_parameters: &[TypeParameter]) -> String {
        if type_parameters.is_empty() {
            String::new()
        } else {
            format!("<{}>",
                itertools::intersperse(
                    type_parameters.iter()
                        .map(|x| {
                            x.name_ident.as_str()
                        }),
                    ", "
                ).collect::<String>()
            )
        }
    }

    fn generate_type_parameters_constraints_code(&self, type_parameters: &[TypeParameter], extra_constraint: &str) -> String {
        let mut code = String::new();

        for t in type_parameters.iter() {
            code.push_str(&format!("{}: {},\n", 
                t.name_ident.as_str(),
                itertools::intersperse(
                    [extra_constraint].into_iter().chain(
                        t.trait_constraints.iter().map(|x| x.trait_name.suffix.as_str())
                    ), 
                    " + "
                ).collect::<String>()
            ));
        }

        if !code.is_empty() {
            code = format!(" where {code}\n");
        }

        code
    }

    fn generate_abi_encode_code(&self, name: &BaseIdent, type_parameters: &[TypeParameter], body: String) -> String {
        let type_parameters_declaration = self.generate_type_parameters_declaration_code(type_parameters);
        let type_parameters_constraints = self.generate_type_parameters_constraints_code(type_parameters, "AbiEncode");

        let name = name.as_str();
        format!("#[allow(dead_code)] impl{type_parameters_declaration} AbiEncode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
            #[allow(dead_code)]
            fn abi_encode(self, ref mut buffer: Buffer) {{
                {body}
            }}
        }}")
    }

    fn generate_abi_decode_code(&self, name: &BaseIdent, type_parameters: &[TypeParameter], body: String) -> String {
        let type_parameters_declaration = self.generate_type_parameters_declaration_code(type_parameters);
        let type_parameters_constraints = self.generate_type_parameters_constraints_code(type_parameters, "AbiDecode");

        let name = name.as_str();
        format!("#[allow(dead_code)] impl{type_parameters_declaration} AbiDecode for {name}{type_parameters_declaration}{type_parameters_constraints} {{
            #[allow(dead_code)]
            fn abi_decode(ref mut buffer: BufferReader) -> Self {{
                {body}
            }}
        }}")
    }

    fn generate_abi_encode_struct_body(&self, engines: &Engines, decl: &TyStructDecl) -> String {
        let mut code = String::new();

        for f in decl.fields.iter() {
            code.push_str(&format!("self.{field_name}.abi_encode(buffer);\n", 
                field_name = f.name.as_str(),
            ));
        }

        code
    }

    fn generate_abi_decode_struct_body(&self, engines: &Engines, decl: &TyStructDecl) -> String {
        let mut code = String::new();
        for f in decl.fields.iter() {
            code.push_str(&format!("{field_name}: buffer.decode::<{field_type_name}>(),", 
                field_name = f.name.as_str(),
                field_type_name = self.generate_type(engines, f.type_argument.type_id),
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
                    let variant_type_name = self.generate_type(engines, x.type_argument.type_id);
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
        write!(&mut code, "let variant: u64 = buffer.decode::<u64>();\n").unwrap();
        write!(&mut code, "match variant {{ {arms} _ => __revert(0), }}").unwrap();

        code
    }

    fn generate_abi_encode_enum_body(&self, engines: &Engines, decl: &TyEnumDecl) -> String {
        let enum_name = decl.call_path.suffix.as_str();
        let arms = decl.variants.iter()
            .map(|x| {
                let name = x.name.as_str();
                if engines.te().get(x.type_argument.type_id).is_unit() {
                    format!("{enum_name}::{variant_name} => {{
                        {tag_value}u64.abi_encode(buffer);
                    }}, \n", tag_value = x.tag, enum_name = enum_name, variant_name = name)
                } else {
                    format!("{enum_name}::{variant_name}(value) => {{
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

    pub fn parse_item_fn_to_typed_ast_node(&mut self, engines: &Engines, kind: FunctionDeclarationKind, code: &str) -> Option<TyAstNode> {
        // println!("{}", code);
        
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
            _ => todo!()
        };

        assert!(!handler.has_errors(), "{:?} {}", handler, code);
        assert!(!handler.has_warnings(), "{:?}", handler);

        let ctx = self.ctx.by_ref();
        let decl = TyDecl::type_check(&handler, ctx, parsed::Declaration::FunctionDeclaration(decl)).unwrap();

        assert!(!handler.has_errors(), "{:?} {}", handler, code);
        assert!(!handler.has_warnings(), "{:?}", handler);

        Some(TyAstNode {
            content: ty::TyAstNodeContent::Declaration(decl),
            span: Span::dummy(),
        })
    }

    fn parse_item_impl_to_typed_ast_node(&mut self, engines: &Engines, code: &str) -> Option<TyAstNode> {
        // println!("{}", code);

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
            None,
        )
        .unwrap();

        let decl = match nodes[0].content {
            AstNodeContent::Declaration(Declaration::ImplTrait(f)) => f,
            _ => todo!()
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
        let abi_encode_code = self.generate_abi_encode_code(struct_decl.name(), &struct_decl.type_parameters, abi_encode_body);
        let abi_encode_node = self.parse_item_impl_to_typed_ast_node(engines, &abi_encode_code);

        let abi_decode_body = self.generate_abi_decode_struct_body(engines, &struct_decl);
        let abi_decode_code = self.generate_abi_decode_code(struct_decl.name(), &struct_decl.type_parameters, abi_decode_body);
        let abi_decode_node = self.parse_item_impl_to_typed_ast_node(engines, &abi_decode_code);

        (abi_encode_node, abi_decode_node)
    }

    fn apply_constraints(&mut self, engines: &Engines, type_id: TypeId, trait_name: CallPath) -> TypeId {
        let type_info = self.ctx.engines().te().get(type_id);

        let handler = Handler::default();

        let variant_type_info = type_info.apply_constraints(&handler, engines, &[TraitConstraint {
            trait_name,
            type_arguments: vec![],
        }]);

        assert!(!handler.has_errors(), "{:?}", handler);
        
        let variant_type_info = variant_type_info.unwrap();
        // dbg!(engines.help_out(&variant_type_info));
        // dbg!(&variant_type_info);

        let type_id = engines.te().insert(engines, variant_type_info, None);

        // let type_id = self.ctx
        //     .resolve_type(
        //         &handler,
        //         type_id,
        //         &Span::dummy(),
        //         EnforceTypeArguments::No,
        //         None,
        //     ).unwrap();
        // assert!(!handler.has_errors(), "{:?}", handler);

        type_id
    }

    /// Verify if an enum has all variants that can be implement AbiEncode and AbiDecode.
    fn can_auto_impl_enum(&mut self, engines: &Engines, decl: &ty::TyDecl) -> (bool, bool) {
        if matches!(self.ctx.namespace.root().module.name.as_ref(), Some(x) if x.as_str() == "core")
        {
            return (false, false);
        }
        
        let handler = Handler::default();
        let Ok(enum_decl) = decl
            .to_enum_ref(&handler, self.ctx.engines())
            .map(|enum_ref| self.ctx.engines().de().get(enum_ref.id()))
        else {
            return (false, false);
        };

        if enum_decl.name().as_str().contains("SomeEnum") {
            // dbg!(&enum_decl);
        }

        let all_variants_are_abi_encode = enum_decl.variants.iter().all(|variant| {
            let r = self.ctx.check_type_impls_traits(
                variant.type_argument.type_id,
                &[TraitConstraint {
                    trait_name: self.abi_encode_call_path.clone(),
                    type_arguments: vec![],
                }],
            );

            if r {
                return true;
            }

            let variant_type_id = self.apply_constraints(engines, variant.type_argument.type_id, self.abi_encode_call_path.clone());
            let r = self.ctx.check_type_impls_traits(
                variant_type_id,
                &[TraitConstraint {
                    trait_name: self.abi_encode_call_path.clone(),
                    type_arguments: vec![],
                }],
            );

            // dbg!(&variant.name, engines.help_out(variant.type_argument.type_id), r);

            r
        });

        let all_variants_are_abi_decode = enum_decl.variants.iter().all(|variant| {
            let r = self.ctx.check_type_impls_traits(
                variant.type_argument.type_id,
                &[TraitConstraint {
                    trait_name: self.abi_decode_call_path.clone(),
                    type_arguments: vec![],
                }],
            );

            if r {
                return true;
            }

            let variant_type_id = self.apply_constraints(engines, variant.type_argument.type_id, self.abi_decode_call_path.clone());
            let r = self.ctx.check_type_impls_traits(
                variant_type_id,
                &[TraitConstraint {
                    trait_name: self.abi_decode_call_path.clone(),
                    type_arguments: vec![],
                }],
            );

            // dbg!(&variant.name, r);

            r
        });

        (all_variants_are_abi_encode, all_variants_are_abi_decode)
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
        let abi_decode_code = self.generate_abi_encode_code(enum_decl.name(), &enum_decl.type_parameters, abi_decode_body);
        let abi_encode_node = self.parse_item_impl_to_typed_ast_node(engines, &abi_decode_code);

        let abi_decode_body = self.generate_abi_decode_enum_body(engines, &enum_decl);
        let abi_decode_code = self.generate_abi_decode_code(enum_decl.name(), &enum_decl.type_parameters, abi_decode_body);
        let abi_decode_node = self.parse_item_impl_to_typed_ast_node(engines, &abi_decode_code);

        (abi_encode_node, abi_decode_node)
    }

    pub fn generate(
        &mut self,
        engines: &Engines,
        decl: &ty::TyDecl,
    ) -> (Option<TyAstNode>, Option<TyAstNode>) {
        // println!("{}", decl.friendly_name(engines));
        let r = match decl {
            TyDecl::StructDecl(_) => self.auto_impl_struct(engines, decl),
            TyDecl::EnumDecl(_) => self.auto_impl_enum(engines, decl),
            _ => (None, None),
        };
        // println!("Done");
        r
    }

    fn generate_type(&self, engines: &Engines, type_id: TypeId) -> String {
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
                    return format!("()");
                }
                let field_strs = fields
                    .iter()
                    .map(|field|  self.generate_type(engines, field.type_id))
                    .collect::<Vec<_>>();
                format!("({},)", field_strs.join(", "))
            }
            TypeInfo::B256 => "b256".into(),
            TypeInfo::Enum(decl_ref) => {
                let decl = engines.de().get(decl_ref.id());

                let type_parameters = decl.type_parameters.iter().map(|x| {
                    self.generate_type(engines, x.type_id)
                }).join(", ");

                let type_parameters = if !type_parameters.is_empty() {
                    format!("<{type_parameters}>")
                } else {
                    type_parameters
                };

                format!("{}{type_parameters}", decl.call_path.suffix.as_str())
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = engines.de().get(decl_ref.id());
                
                let type_parameters = decl.type_parameters.iter().map(|x| {
                    self.generate_type(engines, x.type_id)
                }).join(", ");

                let type_parameters = if !type_parameters.is_empty() {
                    format!("<{type_parameters}>")
                } else {
                    type_parameters
                };

                format!("{}{type_parameters}", decl.call_path.suffix.as_str())
            }
            TypeInfo::Array(elem_ty, count) => {
                format!("[{}; {}]",  self.generate_type(engines, elem_ty.type_id), count.val())
            }
            TypeInfo::RawUntypedPtr => "raw_ptr".into(),
            TypeInfo::RawUntypedSlice => "raw_slice".into(),
            TypeInfo::Alias { name, .. } => name.to_string(),
            _ => todo!()
        }
    }

    pub(crate) fn generate_contract_entry(&mut self, engines: &Engines, contract_fns: &[DeclRef<DeclId<TyFunctionDecl>>]) -> Option<TyAstNode> {
        let mut code = String::new();

        let mut reads = false;
        let mut writes = false;

        for r in contract_fns {
            let decl = engines.de().get(r.id());

            match decl.purity {
                Purity::Pure => {},
                Purity::Reads => {reads = true},
                Purity::Writes => {writes = true},
                Purity::ReadsWrites => {
                    reads = true;
                    writes = true;
                },
            }

            let args_types = itertools::intersperse(
                decl.parameters.iter().map(|x| {
                    self.generate_type(engines, x.type_argument.type_id)
                }),
                ", ".into()
            ).collect::<String>();

            let args_types = if args_types.is_empty() {
                "()".into()
            } else {
                format!("({args_types},)")
            };

            let expanded_args = itertools::intersperse(
                decl.parameters.iter()
                    .enumerate()
                    .map(|(i, _)| format!("args.{i}")),
                ", ".into()
            ).collect::<String>();

            let return_type = self.generate_type(engines, decl.return_type.type_id);

            let method_name = decl.name.as_str();

            code.push_str(&format!("if method_name == \"{method_name}\" {{
                let args = decode_second_param::<{args_types}>();
                let result_{method_name}: raw_slice = encode::<{return_type}>(__contract_entry_{method_name}({expanded_args}));
                __contract_ret(result_{method_name}.ptr(), result_{method_name}.len::<u8>());
            }}\n"));
        }

        let att: String = match (reads, writes) {
            (true, true) => "#[storage(read, write)]",
            (true, false) => "#[storage(read)]",
            (false, true) => "#[storage(write)]",
            (false, false) => ""
        }.into();

        let code = format!("{att}
        pub fn __entry() {{
            let method_name = decode_first_param::<str>();
            {code}
        }}");

        self.parse_item_fn_to_typed_ast_node(engines, FunctionDeclarationKind::Entry, &code)
    }

    pub(crate) fn generate_predicate_entry(&mut self, engines: &Engines, decl: &TyFunctionDecl) -> Option<TyAstNode> {
        let args_types = itertools::intersperse(
            decl.parameters.iter().map(|x| {
                self.generate_type(engines, x.type_argument.type_id)
            }),
            ", ".into()
        ).collect::<String>();

        let args_types = if args_types.is_empty() {
            "()".into()
        } else {
            format!("({args_types},)")
        };

        let expanded_args = itertools::intersperse(
            decl.parameters.iter()
                .enumerate()
                .map(|(i, _)| format!("args.{i}")),
            ", ".into()
        ).collect::<String>();

        let code = format!("pub fn __entry() -> bool {{
            let args = decode_script_data::<{args_types}>();
            main({expanded_args})
        }}");
        self.parse_item_fn_to_typed_ast_node(engines, FunctionDeclarationKind::Entry, &code)
    }

    pub(crate) fn generate_script_entry(&mut self, engines: &Engines, decl: &TyFunctionDecl) -> Option<TyAstNode> {
        let args_types = itertools::intersperse(
            decl.parameters.iter().map(|x| {
                self.generate_type(engines, x.type_argument.type_id)
            }),
            ", ".into()
        ).collect::<String>();

        let args_types = if args_types.is_empty() {
            "()".into()
        } else {
            format!("({args_types},)")
        };

        let expanded_args = itertools::intersperse(
            decl.parameters.iter()
                .enumerate()
                .map(|(i, _)| format!("args.{i}")),
            ", ".into()
        ).collect::<String>();

        let return_type = self.generate_type(engines, decl.return_type.type_id);

        let code = format!("pub fn __entry() -> raw_slice {{
            let args = decode_script_data::<{args_types}>();
            let result: {return_type} = main({expanded_args}); 
            encode::<{return_type}>(result)
        }}");

        // println!("Generate Script Entry");
        // println!("---------------------");
        self.parse_item_fn_to_typed_ast_node(engines, FunctionDeclarationKind::Entry, &code)
    }
}
