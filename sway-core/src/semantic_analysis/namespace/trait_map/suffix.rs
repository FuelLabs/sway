use sway_types::Ident;

use crate::{engine_threading::*, type_system::*};

#[derive(Clone, Debug)]
pub(super) struct TraitSuffix {
    pub(super) name: Ident,
    pub(super) args: Vec<TypeArgument>,
}

impl DisplayWithEngines for TraitSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: Engines<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.name,
            if self.args.is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    self.args
                        .iter()
                        .map(|arg| engines.help_out(arg).to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        )
    }
}

impl PartialEq for TraitSuffix {
    fn eq(&self, other: &Self) -> bool {
        let TraitSuffix { name: ln, args: la } = self;
        let TraitSuffix { name: rn, args: ra } = other;
        ln == rn
            && la.len() == ra.len()
            && la
                .iter()
                .zip(ra.iter())
                .map(|(left, right)| {
                    let TypeArgument {
                        type_id: lti,
                        // these fields are not relevant
                        initial_type_id: _,
                        span: _,
                        call_path_tree: _,
                    } = left;
                    let TypeArgument {
                        type_id: rti,
                        // these fields are not relevant
                        initial_type_id: _,
                        span: _,
                        call_path_tree: _,
                    } = right;
                    lti == rti
                })
                .all(|b| b)
    }
}

impl Eq for TraitSuffix {}

impl PartialOrd for TraitSuffix {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let TraitSuffix { name: ln, args: la } = self;
        let TraitSuffix { name: rn, args: ra } = other;
        match ln.partial_cmp(rn) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match la.len().partial_cmp(&ra.len()) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        for (left, right) in la.iter().zip(ra.iter()) {
            let TypeArgument {
                type_id: lti,
                // these fields are not relevant
                initial_type_id: _,
                span: _,
                call_path_tree: _,
            } = left;
            let TypeArgument {
                type_id: rti,
                // these fields are not relevant
                initial_type_id: _,
                span: _,
                call_path_tree: _,
            } = right;
            match lti.partial_cmp(rti) {
                Some(core::cmp::Ordering::Equal) => {}
                ord => return ord,
            }
        }
        Some(core::cmp::Ordering::Equal)
    }
}

impl Ord for TraitSuffix {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let TraitSuffix { name: ln, args: la } = self;
        let TraitSuffix { name: rn, args: ra } = other;
        ln.cmp(rn)
            .then_with(|| la.len().cmp(&ra.len()))
            .then_with(|| {
                la.iter()
                    .zip(ra.iter())
                    .fold(core::cmp::Ordering::Equal, |acc, (left, right)| {
                        let TypeArgument {
                            type_id: lti,
                            // these fields are not relevant
                            initial_type_id: _,
                            span: _,
                            call_path_tree: _,
                        } = left;
                        let TypeArgument {
                            type_id: rti,
                            // these fields are not relevant
                            initial_type_id: _,
                            span: _,
                            call_path_tree: _,
                        } = right;
                        acc.then_with(|| lti.cmp(rti))
                    })
            })
    }
}
