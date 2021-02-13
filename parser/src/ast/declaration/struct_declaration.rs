use crate::ast::declaration::TypeInfo;
use crate::error::CompileError;
use crate::parser::{HllParser, Rule};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct StructDeclaration<'sc> {
    name: &'sc str,
    fields: Vec<StructField<'sc>>,
}

#[derive(Debug, Clone)]
pub(crate) struct StructField<'sc> {
    name: &'sc str,
    r#type: TypeInfo<'sc>,
}

impl<'sc> StructDeclaration<'sc> {
    pub(crate) fn parse_from_pair(decl: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut decl = decl.into_inner();
        let name = decl.next().unwrap().as_str();
        let fields = decl.next();
        let fields = if let Some(fields) = fields {
            StructField::parse_from_pairs(fields)?
        } else {
            Vec::new()
        };
        Ok(StructDeclaration { name, fields })
    }
}

impl<'sc> StructField<'sc> {
    pub(crate) fn parse_from_pairs(pair: Pair<'sc, Rule>) -> Result<Vec<Self>, CompileError<'sc>> {
        let mut fields = pair.into_inner().collect::<Vec<_>>();
        let mut fields_buf = Vec::new();
        for i in (0..fields.len()).step_by(2) {
            let name = fields[i].as_str();
            let r#type = TypeInfo::parse_from_pair_inner(fields[i + 1].clone())?;
            fields_buf.push(StructField { name, r#type });
        }
        Ok(fields_buf)
    }
}
