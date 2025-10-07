use super::{TyIncludeStatement, TyUseStatement};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TySideEffect {
    IncludeStatement(TyIncludeStatement),
    UseStatement(TyUseStatement),
}
