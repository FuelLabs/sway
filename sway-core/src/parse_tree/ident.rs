use crate::{build_config::BuildConfig, error::*, parser::Rule};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

pub(crate) fn parse_from_pair(
    pair: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<Ident> {
    let path = config.map(|config| config.path());
    let span = {
        if pair.as_rule() != Rule::ident {
            Span {
                span: pair.into_inner().next().unwrap().as_span(),
                path,
            }
        } else {
            Span {
                span: pair.as_span(),
                path,
            }
        }
    };
    ok(Ident::new(span), Vec::new(), Vec::new())
}
