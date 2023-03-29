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
