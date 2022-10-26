use sha2::{Digest, Sha256};
use sway_types::{Ident, Span};

use crate::{
    error::*,
    language::{ty::*, Purity, Visibility},
    transform,
    type_system::*,
    types::ToFnSelector,
};

pub(crate) struct TyFunctionSignature {
    #[allow(dead_code)]
    pub(crate) visibility: Visibility,
    pub(crate) name: Ident,
    #[allow(dead_code)]
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) parameters: Vec<TyFunctionParameter>,
    pub(crate) return_type: TypeId,
    #[allow(dead_code)]
    pub(crate) return_type_span: Span,
    pub(crate) purity: Purity,
    pub(crate) is_contract_call: bool,
    #[allow(dead_code)]
    pub(crate) attributes: transform::AttributesMap,
    #[allow(dead_code)]
    origin: TyFunctionSignatureOrigin,
}

impl From<&TyFunctionDeclaration> for TyFunctionSignature {
    fn from(decl: &TyFunctionDeclaration) -> Self {
        let TyFunctionDeclaration {
            name,
            parameters,
            attributes,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            is_contract_call,
            purity,
            ..
        } = decl;
        TyFunctionSignature {
            visibility: *visibility,
            name: name.clone(),
            type_parameters: type_parameters.clone(),
            parameters: parameters.clone(),
            return_type: *return_type,
            return_type_span: return_type_span.clone(),
            purity: *purity,
            is_contract_call: *is_contract_call,
            attributes: attributes.clone(),
            origin: TyFunctionSignatureOrigin::FunctionDeclaration,
        }
    }
}

impl From<TyFunctionDeclaration> for TyFunctionSignature {
    fn from(decl: TyFunctionDeclaration) -> Self {
        let TyFunctionDeclaration {
            name,
            parameters,
            attributes,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            is_contract_call,
            purity,
            ..
        } = decl;
        TyFunctionSignature {
            visibility,
            name,
            type_parameters,
            parameters,
            return_type,
            return_type_span,
            purity,
            is_contract_call,
            attributes,
            origin: TyFunctionSignatureOrigin::FunctionDeclaration,
        }
    }
}

impl From<TyTraitFn> for TyFunctionSignature {
    fn from(decl: TyTraitFn) -> Self {
        let TyTraitFn {
            name,
            purity,
            parameters,
            return_type,
            return_type_span,
            attributes,
        } = decl;
        TyFunctionSignature {
            visibility: Visibility::Private,
            name,
            type_parameters: vec![],
            parameters,
            return_type,
            return_type_span,
            purity,
            is_contract_call: false,
            attributes,
            origin: TyFunctionSignatureOrigin::TraitFnDeclaration,
        }
    }
}

impl ToFnSelector for TyFunctionSignature {
    fn to_fn_selector_value_untruncated(&self) -> CompileResult<Vec<u8>> {
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
    fn to_fn_selector_value(&self) -> CompileResult<[u8; 4]> {
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

    fn to_selector_name(&self) -> CompileResult<String> {
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
}

enum TyFunctionSignatureOrigin {
    FunctionDeclaration,
    TraitFnDeclaration,
}
