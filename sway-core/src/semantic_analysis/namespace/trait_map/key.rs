use crate::{engine_threading::*, language::CallPath, type_system::*};

use super::*;

/// Smart wrapper for the trait key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TraitKey {
    pub(super) call_path: TraitCallPath,
    pub(super) implementing_for: TypeId,
}

impl DisplayWithEngines for TraitKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: Engines<'_>) -> std::fmt::Result {
        let TraitKey {
            call_path:
                CallPath {
                    prefixes,
                    suffix,
                    is_absolute: _,
                },
            implementing_for,
        } = self;
        let type_engine = engines.te();
        write!(
            f,
            "({}{}, {})",
            if prefixes.is_empty() {
                String::new()
            } else {
                format!(
                    "{}::",
                    prefixes
                        .iter()
                        .map(|name| name.as_str())
                        .collect::<Vec<_>>()
                        .join("::")
                )
            },
            engines.help_out(suffix),
            engines.help_out(type_engine.get(*implementing_for))
        )
    }
}
