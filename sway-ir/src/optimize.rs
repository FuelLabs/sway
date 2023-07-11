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
pub mod const_demotion;
pub use const_demotion::*;
pub mod constants;
pub use constants::*;
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

mod target_fuel;

#[cfg(test)]
pub mod tests {
    use sway_types::SourceEngine;

    use crate::{PassGroup, PassManager};

    // This function parses the IR text representation and run the specified optimizers. After that checks if the IR WAS
    // modified and captures all instructions with metadata "!0". These are checked against `expected`.
    //
    // For example:
    //
    // ```rust, ignore
    // assert_is_optimized(
    //     &["constcombine"],
    //     "entry fn main() -> u64 {
    //        entry():
    //             l = const u64 1
    //             r = const u64 2
    //             result = add l, r, !0
    //             ret u64 result
    //     }",
    //     ["const u64 3"],
    // );
    // ```
    pub(crate) fn assert_is_optimized<'a>(
        passes: &[&'static str],
        body: &str,
        expected: impl IntoIterator<Item = &'a str>,
    ) {
        let source_engine = SourceEngine::default();
        let mut context = crate::parse(
            &format!(
                "script {{
                {body}
            }}

            !0 = \"a.sw\""
            ),
            &source_engine,
        )
        .unwrap();

        let mut pass_manager = PassManager::default();
        crate::register_known_passes(&mut pass_manager);

        let mut group = PassGroup::default();
        for pass in passes {
            group.append_pass(pass);
        }

        let r = pass_manager.run(&mut context, &group).unwrap();
        assert!(r);

        let actual = context
            .to_string()
            .lines()
            .filter_map(|x| {
                if x.contains(", !0") {
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
                panic!("error: {actual:?} {expected:?}");
            } else {
                expected_matches -= 1;
            }
        }

        assert_eq!(expected_matches, 0);
    }

    // This function parses the IR text representation and run the specified optimizers. After that checks if the IR was
    // NOT modified.
    //
    // For example:
    //
    // ```rust, ignore
    // assert_is_not_optimized(
    //     &["constcombine"],
    //     "entry fn main() -> u64 {
    //        entry():
    //             l = const u64 0
    //             r = const u64 1
    //             result = sub l, r, !0
    //             ret u64 result
    //     }"
    // );
    // ```
    pub(crate) fn assert_is_not_optimized(passes: &[&'static str], body: &str) {
        let source_engine = SourceEngine::default();
        let mut context = crate::parse(
            &format!(
                "script {{
                {body}
            }}

            !0 = \"a.sw\""
            ),
            &source_engine,
        )
        .unwrap();

        let mut pass_manager = PassManager::default();
        crate::register_known_passes(&mut pass_manager);

        let mut group = PassGroup::default();
        for pass in passes {
            group.append_pass(pass);
        }

        let r = pass_manager.run(&mut context, &group).unwrap();
        assert!(!r);
    }
}
