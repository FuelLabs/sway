use generational_arena::Index;
use sway_types::{Ident, Span};

use crate::CompileResult;
use crate::MatchCondition;

/// Algorithm modeled after this documentation:
/// https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/thir/pattern/usefulness/index.html
/// and this paper:
/// http://moscova.inria.fr/%7Emaranget/papers/warn/index.html
pub(crate) fn check_match_expression_usefulness(
    _variable_created: Ident,
    _cases_covered: Vec<MatchCondition>,
    _span: Span,
    _namespace: Index,
) -> CompileResult<()> {
    unimplemented!()
}
