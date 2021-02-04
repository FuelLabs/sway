use crate::ast::Expression;
use crate::{CodeBlock, CompileError, Rule};
use either::Either;
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) struct FunctionDeclaration<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) body: CodeBlock<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) span: pest::Span<'sc>,
    pub(crate) return_type: TypeInfo<'sc>,
}

impl<'sc> FunctionDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut parts = pair.clone().into_inner();
        let mut signature = parts.next().unwrap().into_inner();
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap().as_str();
        let parameters = signature.next().unwrap();
        let parameters = FunctionParameter::list_from_pairs(parameters.into_inner())?;
        let return_type_signal = signature.next();
        let return_type = match return_type_signal {
            Some(_) => TypeInfo::parse_from_pair(signature.next().unwrap())?,
            None => TypeInfo::Unit,
        };
        let body = parts.next().unwrap();
        let body = CodeBlock::parse_from_pair(body)?;
        Ok(FunctionDeclaration {
            name,
            parameters,
            body,
            span: pair.as_span(),
            return_type,
        })
    }
}

#[derive(Debug)]
pub(crate) struct FunctionParameter<'sc> {
    name: &'sc str,
    r#type: TypeInfo<'sc>,
}

impl<'sc> FunctionParameter<'sc> {
    pub(crate) fn list_from_pairs(
        pairs: impl Iterator<Item = Pair<'sc, Rule>>,
    ) -> Result<Vec<FunctionParameter<'sc>>, CompileError<'sc>> {
        pairs
            .map(|pair: Pair<'sc, Rule>| {
                let mut parts = pair.clone().into_inner();
                let name = parts.next().unwrap().as_str();
                let r#type = if name == "self" {
                    TypeInfo::SelfType
                } else {
                    TypeInfo::parse_from_pair_inner(parts.next().unwrap())?
                };
                Ok(FunctionParameter { name, r#type })
            })
            .collect()
    }
}

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug)]
pub(crate) enum TypeInfo<'sc> {
    String,
    UnsignedInteger(IntegerBits),
    Boolean,
    Generic { name: &'sc str },
    Unit,
    SelfType,
}
#[derive(Debug)]
pub(crate) enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl<'sc> TypeInfo<'sc> {
    pub(crate) fn parse_from_pair(input: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut r#type = input.into_inner();
        Self::parse_from_pair_inner(r#type.next().unwrap())
    }
    pub(crate) fn parse_from_pair_inner(input: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        Ok(match input.as_str() {
            "u8" => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            "u16" => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
            "u32" => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            "u64" => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            "bool" => TypeInfo::Boolean,
            "string" => TypeInfo::String,
            "unit" => TypeInfo::Unit,
            other => TypeInfo::Generic { name: other },
        })
    }
}
