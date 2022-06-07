mod function_parameter;
pub use function_parameter::*;

use crate::{
    error::*, namespace::*, parse_tree::*, semantic_analysis::*, style::*, type_engine::*, types::*,
};
use fuels_types::{Function, Property};
use sha2::{Digest, Sha256};
use sway_types::{Ident, Span, Spanned};

#[derive(Clone, Debug, Eq)]
pub struct TypedFunctionDeclaration {
    pub name: Ident,
    pub body: TypedCodeBlock,
    pub parameters: Vec<TypedFunctionParameter>,
    pub(crate) span: Span,
    pub(crate) return_type: TypeId,
    pub(crate) type_parameters: Vec<TypeParameter>,
    /// Used for error messages -- the span pointing to the return type
    /// annotation of the function
    pub(crate) return_type_span: Span,
    pub(crate) visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub(crate) is_contract_call: bool,
    pub(crate) purity: Purity,
}

impl From<&TypedFunctionDeclaration> for TypedAstNode {
    fn from(o: &TypedFunctionDeclaration) -> Self {
        let span = o.span.clone();
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                o.clone(),
            )),
            span,
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedFunctionDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.body == other.body
            && self.parameters == other.parameters
            && look_up_type_id(self.return_type) == look_up_type_id(other.return_type)
            && self.type_parameters == other.type_parameters
            && self.visibility == other.visibility
            && self.is_contract_call == other.is_contract_call
            && self.purity == other.purity
    }
}

impl CopyTypes for TypedFunctionDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));

        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));

        self.return_type
            .update_type(type_mapping, &self.return_type_span);
        self.body.copy_types(type_mapping);
    }
}

impl Spanned for TypedFunctionDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TypedFunctionDeclaration {
    type Output = TypedFunctionDeclaration;

    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }

    fn monomorphize_inner(
        self,
        type_mapping: &TypeMapping,
        _namespace: &mut Items,
    ) -> Self::Output {
        let mut new_decl = self;
        new_decl.copy_types(type_mapping);
        new_decl
    }
}

impl ToJsonAbi for TypedFunctionDeclaration {
    type Output = Function;

    fn generate_json_abi(&self) -> Self::Output {
        Function {
            name: self.name.as_str().to_string(),
            type_field: "function".to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| Property {
                    name: x.name.as_str().to_string(),
                    type_field: x.r#type.json_abi_str(),
                    components: x.r#type.generate_json_abi(),
                })
                .collect(),
            outputs: vec![Property {
                name: "".to_string(),
                type_field: self.return_type.json_abi_str(),
                components: self.return_type.generate_json_abi(),
            }],
        }
    }
}

impl TypedFunctionDeclaration {
    pub fn type_check(
        arguments: TypeCheckArguments<'_, FunctionDeclaration>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let TypeCheckArguments {
            checkee: fn_decl,
            namespace,
            self_type,
            mode,
            mut opts,
            ..
        } = arguments;

        let FunctionDeclaration {
            name,
            body,
            parameters,
            span,
            return_type,
            mut type_parameters,
            return_type_span,
            visibility,
            purity,
            ..
        } = fn_decl;

        is_snake_case(&name).ok(&mut warnings, &mut errors);
        opts.purity = purity;

        // insert parameters and generic type declarations into namespace
        let mut namespace = namespace.clone();

        // insert type parameters as Unknown types
        let type_mapping = insert_type_parameters(&type_parameters);

        // update the types in the type parameters, insert the type parameters
        // into the decl namespace, and check to see if the type parameters
        // shadow one another
        for type_parameter in type_parameters.iter_mut() {
            check!(
                type_parameter.update_types_with_self(&type_mapping, &mut namespace, self_type),
                return err(warnings, errors),
                warnings,
                errors
            );
            let type_parameter_decl = TypedDeclaration::GenericTypeForFunctionScope {
                name: type_parameter.name_ident.clone(),
                type_id: type_parameter.type_id,
            };
            check!(
                namespace.insert_symbol(type_parameter.name_ident.clone(), type_parameter_decl),
                continue,
                warnings,
                errors
            );
        }

        // type check the parameters and insert them into the function namespace
        let mut new_parameters = vec![];
        for parameter in parameters.into_iter() {
            let parameter = check!(
                TypedFunctionParameter::type_check(
                    parameter,
                    &mut namespace,
                    self_type,
                    EnforceTypeArguments::Yes,
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            namespace.insert_symbol(
                parameter.name.clone(),
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    name: parameter.name.clone(),
                    body: TypedExpression {
                        expression: TypedExpressionVariant::FunctionParameter,
                        return_type: parameter.r#type,
                        is_constant: IsConstant::No,
                        span: parameter.name.span(),
                    },
                    is_mutable: VariableMutability::Immutable,
                    const_decl_origin: false,
                    type_ascription: parameter.r#type,
                }),
            );
            new_parameters.push(parameter);
        }

        // type check the return type
        let return_type = check!(
            namespace.resolve_type_with_self(
                return_type,
                self_type,
                &return_type_span,
                EnforceTypeArguments::Yes
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = check!(
            TypedCodeBlock::type_check(TypeCheckArguments {
                checkee: body,
                namespace: &mut namespace,
                return_type_annotation: return_type,
                help_text:
                    "Function body's return type does not match up with its return type annotation.",
                self_type,
                mode: Mode::NonAbi,
                opts,
            }),
            (
                TypedCodeBlock { contents: vec![] },
                insert_type(TypeInfo::ErrorRecovery)
            ),
            warnings,
            errors
        );

        // gather the return statements from the function body
        let return_statements: Vec<&TypedExpression> = body
            .contents
            .iter()
            .flat_map(|node| -> Vec<&TypedReturnStatement> { node.gather_return_statements() })
            .map(|TypedReturnStatement { expr, .. }| expr)
            .collect();

        // unify the types of the return statements with the return type of the function
        for stmt in return_statements {
            let (mut new_warnings, new_errors) = unify_with_self(
                stmt.return_type,
                return_type,
                self_type,
                &stmt.span,
                "Return statement must return the declared function return type.",
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }

        let function_decl = TypedFunctionDeclaration {
            name,
            body,
            parameters: new_parameters,
            span,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            // if this is for a contract, then it is a contract call
            is_contract_call: mode == Mode::ImplAbiFn,
            purity,
        };

        ok(function_decl, warnings, errors)
    }

    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
                self.parameters[0].name.span(),
                |acc, TypedFunctionParameter { type_span, .. }| Span::join(acc, type_span.clone()),
            )
        } else {
            self.name.span()
        }
    }

    pub(crate) fn replace_self_types(self, self_type: TypeId) -> Self {
        TypedFunctionDeclaration {
            parameters: self
                .parameters
                .iter()
                .map(|x| {
                    let mut x = x.clone();
                    x.r#type = match look_up_type_id(x.r#type) {
                        TypeInfo::SelfType => self_type,
                        _otherwise => x.r#type,
                    };
                    x
                })
                .collect(),
            span: self.span.clone(),
            return_type: match look_up_type_id(self.return_type) {
                TypeInfo::SelfType => self_type,
                _otherwise => self.return_type,
            },
            type_parameters: self.type_parameters.clone(),
            return_type_span: self.return_type_span.clone(),
            ..self
        }
    }

    pub fn to_fn_selector_value_untruncated(&self) -> CompileResult<Vec<u8>> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut hasher = Sha256::new();
        let data = check!(
            self.to_selector_name(),
            return err(warnings, errors),
            warnings,
            errors
        );
        hasher.update(data);
        let hash = hasher.finalize();
        ok(hash.to_vec(), warnings, errors)
    }

    /// Converts a [TypedFunctionDeclaration] into a value that is to be used in contract function
    /// selectors.
    /// Hashes the name and parameters using SHA256, and then truncates to four bytes.
    pub fn to_fn_selector_value(&self) -> CompileResult<[u8; 4]> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let hash = check!(
            self.to_fn_selector_value_untruncated(),
            return err(warnings, errors),
            warnings,
            errors
        );
        // 4 bytes truncation via copying into a 4 byte buffer
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&hash[0..4]);
        ok(buf, warnings, errors)
    }

    pub fn to_selector_name(&self) -> CompileResult<String> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let named_params = self
            .parameters
            .iter()
            .map(
                |TypedFunctionParameter {
                     r#type, type_span, ..
                 }| {
                    resolve_type(*r#type, type_span)
                        .expect("unreachable I think?")
                        .to_selector_name(type_span)
                },
            )
            .filter_map(|name| name.ok(&mut warnings, &mut errors))
            .collect::<Vec<String>>();

        ok(
            format!("{}({})", self.name.as_str(), named_params.join(","),),
            warnings,
            errors,
        )
    }
}

#[test]
fn test_function_selector_behavior() {
    use crate::type_engine::IntegerBits;
    let decl = TypedFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_no_span("foo"),
        body: TypedCodeBlock { contents: vec![] },
        parameters: vec![],
        span: Span::dummy(),
        return_type: 0.into(),
        type_parameters: vec![],
        return_type_span: Span::dummy(),
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "foo()".to_string());

    let decl = TypedFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_with_override("bar", Span::dummy()),
        body: TypedCodeBlock { contents: vec![] },
        parameters: vec![
            TypedFunctionParameter {
                name: Ident::new_no_span("foo"),
                r#type: crate::type_engine::insert_type(TypeInfo::Str(5)),
                type_span: Span::dummy(),
            },
            TypedFunctionParameter {
                name: Ident::new_no_span("baz"),
                r#type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
                type_span: Span::dummy(),
            },
        ],
        span: Span::dummy(),
        return_type: 0.into(),
        type_parameters: vec![],
        return_type_span: Span::dummy(),
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}
