use crate::{Literal, Span, Rule, BuildConfig, CompileResult, error::{err, ok}, CompileError};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum Scrutinee<'sc> {
    Unit {
        span: Span<'sc>
    },
    Literal {
        value: Literal<'sc>,
        span: Span<'sc>,
    }
}

impl<'sc> Scrutinee<'sc> {
    pub fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut scrutinees = pair.into_inner();
        let scrutinee = scrutinees.next().unwrap();
        let scrutinee = check!(
            Scrutinee::parse_from_pair_inner(scrutinee.clone(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(scrutinee, warnings, errors)
    }

    pub fn parse_from_pair_inner(
        scrutinee: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.path());
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let span = Span {
            span: scrutinee.as_span(),
            path: path.clone(),
        };
        let parsed = match scrutinee.as_rule() {
            Rule::literal_value => Literal::parse_from_pair(scrutinee.clone(), config)
                .map(|(value, span)| Scrutinee::Literal { value, span })
                .unwrap_or_else(&mut warnings, &mut errors, || Scrutinee::Unit { span }),
            a => {
                eprintln!(
                    "Unimplemented expr: {:?} ({:?}) ({:?})",
                    a,
                    scrutinee.as_str(),
                    scrutinee.as_rule()
                );
                errors.push(CompileError::UnimplementedRule(
                    a,
                    Span {
                        span: scrutinee.as_span(),
                        path: path.clone(),
                    },
                ));
                // construct unit expression for error recovery
                Scrutinee::Unit {
                    span: Span {
                        span: scrutinee.as_span(),
                        path,
                    },
                }
            }
        };
        ok(parsed, warnings, errors)
    }
}