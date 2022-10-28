use sha2::{Digest, Sha256};
use sway_types::{Ident, JsonABIFunction, JsonTypeApplication, JsonTypeDeclaration, Span, Spanned};

use crate::{
    declaration_engine::*,
    error::*,
    language::{parsed, ty::*, Inline, Purity, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug, Eq)]
pub struct TyFunctionDeclaration {
    pub name: Ident,
    pub body: TyCodeBlock,
    pub parameters: Vec<TyFunctionParameter>,
    pub span: Span,
    pub attributes: transform::AttributesMap,
    pub return_type: TypeId,
    pub initial_return_type: TypeId,
    pub type_parameters: Vec<TypeParameter>,
    /// Used for error messages -- the span pointing to the return type
    /// annotation of the function
    pub return_type_span: Span,
    pub visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub is_contract_call: bool,
    pub purity: Purity,
    pub inline: Inline,
}

impl From<&TyFunctionDeclaration> for TyAstNode {
    fn from(o: &TyFunctionDeclaration) -> Self {
        let span = o.span.clone();
        TyAstNode {
            content: TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration(
                de_insert_function(o.clone()),
            )),
            span,
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyFunctionDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.body == other.body
            && self.parameters == other.parameters
            && look_up_type_id(self.return_type) == look_up_type_id(other.return_type)
            && self.type_parameters == other.type_parameters
            && self.visibility == other.visibility
            && self.is_contract_call == other.is_contract_call
            && self.purity == other.purity
            && self.inline == other.inline
    }
}

impl CopyTypes for TyFunctionDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.return_type.copy_types(type_mapping);
        self.body.copy_types(type_mapping);
    }
}

impl ReplaceSelfType for TyFunctionDeclaration {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(self_type));
        self.parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(self_type));
        self.return_type.replace_self_type(self_type);
        self.body.replace_self_type(self_type);
    }
}

impl Spanned for TyFunctionDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyFunctionDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl UnconstrainedTypeParameters for TyFunctionDeclaration {
    fn type_parameter_is_unconstrained(&self, type_parameter: &TypeParameter) -> bool {
        let type_parameter_info = look_up_type_id(type_parameter.type_id);
        if self
            .type_parameters
            .iter()
            .map(|type_param| look_up_type_id(type_param.type_id))
            .any(|x| x == type_parameter_info)
        {
            return false;
        }
        if self
            .parameters
            .iter()
            .map(|param| look_up_type_id(param.type_id))
            .any(|x| x == type_parameter_info)
        {
            return true;
        }
        if look_up_type_id(self.return_type) == type_parameter_info {
            return true;
        }

        false
    }
}

impl TyFunctionDeclaration {
    /// Used to create a stubbed out function when the function fails to compile, preventing cascading
    /// namespace errors
    pub(crate) fn error(decl: parsed::FunctionDeclaration) -> TyFunctionDeclaration {
        let parsed::FunctionDeclaration {
            name,
            return_type,
            span,
            return_type_span,
            visibility,
            purity,
            inline,
            ..
        } = decl;
        let initial_return_type = insert_type(return_type);
        TyFunctionDeclaration {
            inline,
            purity,
            name,
            body: TyCodeBlock {
                contents: Default::default(),
            },
            span,
            attributes: Default::default(),
            is_contract_call: false,
            return_type_span,
            parameters: Default::default(),
            visibility,
            return_type: initial_return_type,
            initial_return_type,
            type_parameters: Default::default(),
        }
    }

    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
                self.parameters[0].name.span(),
                |acc, TyFunctionParameter { type_span, .. }| Span::join(acc, type_span.clone()),
            )
        } else {
            self.name.span()
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

    /// Converts a [TyFunctionDeclaration] into a value that is to be used in contract function
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
        buf.copy_from_slice(&hash[..4]);
        ok(buf, warnings, errors)
    }

    pub fn to_selector_name(&self) -> CompileResult<String> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let named_params = self
            .parameters
            .iter()
            .map(
                |TyFunctionParameter {
                     type_id, type_span, ..
                 }| {
                    to_typeinfo(*type_id, type_span)
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

    pub(crate) fn generate_json_abi_function(
        &self,
        types: &mut Vec<JsonTypeDeclaration>,
    ) -> JsonABIFunction {
        // A list of all `JsonTypeDeclaration`s needed for inputs
        let input_types = self
            .parameters
            .iter()
            .map(|x| JsonTypeDeclaration {
                type_id: *x.initial_type_id,
                type_field: x.initial_type_id.get_json_type_str(x.type_id),
                components: x.initial_type_id.get_json_type_components(types, x.type_id),
                type_parameters: x.type_id.get_json_type_parameters(types, x.type_id),
            })
            .collect::<Vec<_>>();

        // The single `JsonTypeDeclaration` needed for the output
        let output_type = JsonTypeDeclaration {
            type_id: *self.initial_return_type,
            type_field: self.initial_return_type.get_json_type_str(self.return_type),
            components: self
                .return_type
                .get_json_type_components(types, self.return_type),
            type_parameters: self
                .return_type
                .get_json_type_parameters(types, self.return_type),
        };

        // Add the new types to `types`
        types.extend(input_types);
        types.push(output_type);

        // Generate the JSON data for the function
        JsonABIFunction {
            name: self.name.as_str().to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| JsonTypeApplication {
                    name: x.name.to_string(),
                    type_id: *x.initial_type_id,
                    type_arguments: x.initial_type_id.get_json_type_arguments(types, x.type_id),
                })
                .collect(),
            output: JsonTypeApplication {
                name: "".to_string(),
                type_id: *self.initial_return_type,
                type_arguments: self
                    .initial_return_type
                    .get_json_type_arguments(types, self.return_type),
            },
        }
    }

    /// Whether or not this function is the default entry point.
    pub fn is_main_entry(&self) -> bool {
        // NOTE: We may want to make this check more sophisticated or customisable in the future,
        // but for now this assumption is baked in throughout the compiler.
        self.name.as_str() == sway_types::constants::DEFAULT_ENTRY_POINT_FN_NAME
    }

    /// Whether or not this function is a unit test, i.e. decorated with `#[test]`.
    pub fn is_test(&self) -> bool {
        self.attributes
            .contains_key(&transform::AttributeKind::Test)
    }

    /// Whether or not this function describes a program entry point.
    pub fn is_entry(&self) -> bool {
        self.is_main_entry() || self.is_test()
    }
}

#[derive(Debug, Clone, Eq)]
pub struct TyFunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub type_span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyFunctionParameter {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.is_mutable == other.is_mutable
    }
}

impl CopyTypes for TyFunctionParameter {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        self.type_id.copy_types(type_mapping);
    }
}

impl ReplaceSelfType for TyFunctionParameter {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl TyFunctionParameter {
    pub fn is_self(&self) -> bool {
        self.name.as_str() == "self"
    }
}
