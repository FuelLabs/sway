use std::hash::{Hash, Hasher};

use pest::iterators::Pair;
use sway_types::Span;

use crate::{
    error::ok,
    type_engine::{insert_type, look_up_type_id, TypeId},
    BuildConfig, CompileResult, Rule, TypeInfo,
};

#[derive(Debug, Clone)]
pub struct TypeArgument {
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypeArgument {
    fn hash<H: Hasher>(&self, state: &mut H) {
        look_up_type_id(self.type_id).hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeArgument {
    fn eq(&self, other: &Self) -> bool {
        look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
    }
}

impl Default for TypeArgument {
    fn default() -> Self {
        TypeArgument {
            type_id: insert_type(TypeInfo::Unknown),
            span: Span::dummy(),
        }
    }
}

impl TypeArgument {
    pub(crate) fn parse_arguments_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let iter = pair.into_inner();
        let mut type_arguments = vec![];
        for type_arg in iter {
            type_arguments.push(check!(
                Self::parse_from_pair(type_arg, config),
                continue,
                warnings,
                errors
            ));
        }
        ok(type_arguments, warnings, errors)
    }

    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let span = Span::from_pest(pair.as_span(), config.map(|c| c.path()));
        let type_id = insert_type(check!(
            TypeInfo::parse_from_pair(pair, config),
            TypeInfo::ErrorRecovery,
            warnings,
            errors
        ));
        let type_argument = TypeArgument { type_id, span };
        ok(type_argument, warnings, errors)
    }

    pub(crate) fn friendly_type_str(&self) -> String {
        look_up_type_id(self.type_id).friendly_type_str()
    }

    pub(crate) fn json_abi_str(&self) -> String {
        look_up_type_id(self.type_id).json_abi_str()
    }
}
