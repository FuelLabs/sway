use sway_error::handler::ErrorEmitted;
use sway_utils::PerformanceData;

use super::{lexed::LexedProgram, parsed::ParseProgram, ty::TyProgram};

/// Contains the lexed, parsed, typed compilation stages of a program, as well
/// as compilation metrics.
#[derive(Clone, Debug)]
pub struct Programs {
    pub lexed: LexedProgram,
    pub parsed: ParseProgram,
    pub typed: Result<TyProgram, ErrorEmitted>,
    pub metrics: PerformanceData,
}

impl Programs {
    pub fn new(
        lexed: LexedProgram,
        parsed: ParseProgram,
        typed: Result<TyProgram, ErrorEmitted>,
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
