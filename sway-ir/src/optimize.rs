//! A collection of optimization passes.
//!
//! Each of these modules are a collection of typical code optimisation passes.
//!
//! Currently there is no pass manager, as there are only a couple of passes, but this is something
//! which will be added in the future.
//!
//! So each of the functions under this module will return a boolean indicating whether a
//! modification to the IR was made.  Typically the passes will be just re-run until they no longer
//! make any such modifications, implying they've optimized as much possible.
//!
//! When writing passes one should keep in mind that when a modification is made then any iterators
//! over blocks or instructions can be invalidated, and starting over is a safer option than trying
//! to attempt multiple changes at once.

pub mod arg_demotion;
pub use arg_demotion::*;
pub mod arg_mutability_tagger;
pub use arg_mutability_tagger::*;
pub mod const_demotion;
pub use const_demotion::*;
pub mod constants;
pub use constants::*;
pub mod conditional_constprop;
pub use conditional_constprop::*;
pub mod cse;
pub use cse::*;
pub mod dce;
pub use dce::*;
pub mod inline;
pub use inline::*;
pub mod mem2reg;
pub use mem2reg::*;
pub mod memcpyopt;
pub use memcpyopt::*;
pub mod misc_demotion;
pub use misc_demotion::*;
pub mod ret_demotion;
pub use ret_demotion::*;
pub mod simplify_cfg;
pub use simplify_cfg::*;
pub mod sroa;
pub use sroa::*;
pub mod fn_dedup;
pub use fn_dedup::*;
pub mod init_aggr_lowering;
pub use init_aggr_lowering::*;

mod target_fuel;

#[cfg(test)]
pub mod tests {
    use crate::{Backtrace, PassGroup, PassManager};
    use sway_features::ExperimentalFeatures;
    use sway_types::SourceEngine;

    /// This function parses the IR text representation and run the specified optimizers passes.
    /// Then, depending on the `expected` parameter it checks if the IR was optimized or not.
    ///
    /// This comparison is done by capturing all instructions with metadata "!0".
    ///
    /// For example:
    ///
    /// ```rust, ignore
    /// assert_optimization(
    ///     &[CONST_FOLDING_NAME],
    ///     "entry fn main() -> u64 {
    ///        entry():
    ///             l = const u64 1
    ///             r = const u64 2
    ///             result = add l, r, !0
    ///             ret u64 result
    ///     }",
    ///     ["const u64 3"],
    /// );
    /// ```
    pub(crate) fn assert_optimization<'a>(
        passes: &[&'static str],
        body: &str,
        expected: Option<impl IntoIterator<Item = &'a str>>,
    ) {
        let source_engine = SourceEngine::default();
        let mut context = crate::parse(
            &format!(
                "script {{
                {body}
            }}

            !0 = \"a.sw\"
            "
            ),
            &source_engine,
            ExperimentalFeatures::default(),
            Backtrace::default(),
        )
        .unwrap();

        let mut pass_manager = PassManager::default();
        crate::register_known_passes(&mut pass_manager);

        let mut group = PassGroup::default();
        for pass in passes {
            group.append_pass(pass);
        }

        let before = context.to_string();
        let modified = pass_manager.run(&mut context, &group).unwrap();
        let after = context.to_string();

        // print diff to help debug
        if std::env::args().any(|x| x == "--nocapture") {
            println!("{}", prettydiff::diff_lines(&before, &after));
        }

        assert_eq!(expected.is_some(), modified);

        let Some(expected) = expected else {
            return;
        };

        let actual = context
            .to_string()
            .lines()
            .filter_map(|x| {
                if x.contains(", !") {
                    Some(format!("{}\n", x.trim()))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        assert!(!actual.is_empty());

        let mut expected_matches = actual.len();

        for (actual, expected) in actual.iter().zip(expected) {
            if !actual.contains(expected) {
                panic!("Actual: {actual:?} does not contains expected: {expected:?}. (Run with --nocapture to see a diff)");
            } else {
                expected_matches -= 1;
            }
        }

        assert_eq!(expected_matches, 0);
    }
}
