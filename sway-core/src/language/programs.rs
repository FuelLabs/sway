use super::{lexed::LexedProgram, parsed::ParseProgram, ty::TyProgram};
use crate::semantic_analysis::program::TypeCheckFailed;
use sway_utils::PerformanceData;

/// Contains the lexed, parsed, typed compilation stages of a program, as well
/// as compilation metrics.
#[derive(Clone, Debug)]
pub struct Programs {
    pub lexed: LexedProgram,
    pub parsed: ParseProgram,
    pub typed: Result<TyProgram, TypeCheckFailed>,
    pub metrics: PerformanceData,
}

impl Programs {
    pub fn new(
        lexed: LexedProgram,
        parsed: ParseProgram,
        typed: Result<TyProgram, TypeCheckFailed>,
        metrics: PerformanceData,
    ) -> Programs {
        Programs {
            lexed,
            parsed,
            typed,
            metrics,
        }
    }
}
