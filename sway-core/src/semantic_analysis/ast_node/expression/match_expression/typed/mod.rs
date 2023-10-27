//! This module type checks `match` expressions and desugars `match` expressions to `if` expressions.
//! The desugaring does not provides any kind of optimizations. It provides a structure that
//! can later on be used for code analysis by reusing the existing analysis available for `if` expressions.
//! The optimizations will be done on the IR level.
//!
//! ## Type Checking
//!
//! The central module for type checking is the [matcher].
//!
//! The [matcher::matcher] function will type check the matched value with the match arm pattern (scrutinee).
//! Successful type check will result in an [matcher::ReqDeclTree] that accurately represents all the
//! requirements and variable declarations given by the scrutinee pattern.
//!
//! The resulting [matcher::ReqDeclTree] will be given over to [crate::ty::TyMatchBranch] for additional
//! type checking. E.g., checking for duplicates in declared variables is done on this stage.
//!
//! ## Desugaring
//!
//! Desugaring to `if` expressions starts in the [crate::ty::TyMatchBranch] where three artifacts are provided
//! for a particular match branch (arm):
//! - branch condition: Overall condition that must be `true` for the branch to match.
//! - result variable declarations: Variable declarations that needs to be added to the
//! match branch result, before the actual body. Here we distinguish between the variables
//! actually declared in the match arm pattern and so called "tuple variables" that are
//! compiler generated and contain values for variables extracted out of individual OR variants.
//! - OR variant index variables: Variable declarations that are generated in case of having
//! variables in OR patterns. Index variables hold 1-based index of the OR variant being matched
//! or zero if non of the OR variants has matched.
//!
//! Afterwards, these three artifacts coming from every individual branch are glued together in the
//! [crate::ty::TyMatchExpression] to form the final desugaring.
//!
//! The desugared `if-else` chains end either in an `else` that contains the result of the last match arm, if the
//! match arm is a catch-all arm, or in a `__revert(...)` call with the dedicated revert code.
//! These reverts can happen only if we have bugs in the implementation of match expressions and
//! is the only safe way to communicate compiler bug detectable only at runtime.
//!
//! ## Desugaring Examples
//!
//! The easiest way to explain the desugaring algorithm is to take a look at a few examples of
//! different kinds of match arm patterns, and how they are desugared.
//!
//! Applying the rules sketched below recursively, we can desugar an arbitrary match arm pattern.
//!
//! ### Literals, Constants, and Variables
//!
//! In case of literals, constants, and variables the desugared `if` expression is straightforward.
//!
//! ```ignore
//! match exp {
//!     1 => 111,
//!     CONST_X => 222,
//!     x => x + x,
//! }
//! ```
//! ```ignore
//! let __matched_value_1 = exp;
//! if __matched_value_1 == 1 {
//!     111
//! }
//! else if __matched_value_1 == CONST_X {
//!     222
//! }
//! else {
//!     let x = __matched_value_1;
//!     x + x
//! }
//! else {
//!     __revert(14757395258967588866)
//! }
//! ```
//!
//! If the last match arm is not a catch-all arm, the `if-else` chain will end in a `__revert()`.
//!
//! ```ignore
//! match exp {
//!     true => 111,
//!     false => 222,
//! }
//! ```
//! ```ignore
//! let __matched_value_1 = exp;
//! if __matched_value_1 == true {
//!     111
//! }
//! else if __matched_value_1 == false {
//!     222
//! }
//! else {
//!     __revert(14757395258967588866)
//! }
//! ```
//!
//! ### Structs, Enums, Tuples
//!
//! In case of structs, enums, and tuples the overall requirement becomes the lazy AND of
//! all requirements, and all the variables get extracted.
//!
//! The construction of the match arm condition and the extraction of variables works
//! recursively in case of nested structures. E.g., if we have struct fields being enums
//! of tuples of structs etc.
//!
//! But the resulting condition will always contain only the lazy AND operator and all the
//! variable definitions will be listed at the top of the match arm result.
//!
//! ```ignore
//! struct Point {
//!     x: u64,
//!     y: u64
//!     z: u64
//! }
//!
//! match p {
//!     Point { x: a, y: 22, z: 33 } => { a },
//!     Point { x: 11, y, z: 33} => { y },
//!     Point { z, .. } => { z },
//! }
//! ```
//! ```ignore
//! let __matched_value_1 = p;
//! if __matched_value_1.y == 22 && __matched_value_1.z == 33 {
//!     let a = __matched_value_1.x;
//!     a
//! }
//! if __matched_value_1.x == 11 && __matched_value_1.z == 33 {
//!     let y = __matched_value_1.y;
//!     y
//! }
//! else {
//!     let z = __matched_value_1.z;
//!     z
//! }
//! ```
//!
//! ### Or Patterns
//!
//! In case of or patterns without variables, the resulting desugaring is again straightforward.
//! We simply construct the overall condition by using the lazy OR operator.
//!
//! ```ignore
//! match exp {
//!     1 | 2 => 111,
//!     CONST_X | CONST_Y => 222,
//!     x => x + x,
//! }
//! ```
//! ```ignore
//! let __matched_value_1 = exp;
//! if __matched_value_1 == 1 || __matched_value_1 == 2 {
//!     111
//! }
//! else if __matched_value_1 == CONST_X || __matched_value_1 == CONST_Y {
//!     222
//! }
//! else {
//!     let x = __matched_value_1;
//!     x + x
//! }
//! ```
//!
//! In case of having or patterns with variables, the desugaring pattern gets more complex.
//! Essentially, we have to extract the variables exactly from the variant that has matched.
//! Also, we want to check the conditions for every variant exactly once.
//!
//! To accomplish this, we move the checking of variants outside of the match arm `if` and
//! track the 1-based index of the matched variant in a so called "matched or variant index variable".
//! If no variant matches this variable will be set to zero.
//!
//! We create such "matched or variant index variable" for every or pattern with variables that we
//! encounter in the match arm pattern.
//!
//! Afterwards, in the match arm `if` condition we just check if the index variable is different then
//! zero which means there is a match.
//!
//! To properly extract the variables, in the result, we again check which variant has matched and
//! store all the variables from that variant in a tuple variable called "matched or variants variables".
//! In these tuple variables, the values of the declared variables are stored ordered by the variable name.
//! We can safely do this, knowing that at this point we have fully valid variables, e.g., no duplicates.
//!
//! The final definition of the variables declared in the or pattern is then tuple access to the
//! element of the tuple that holds the value of that particular variable.
//!
//! ```ignore
//! enum Enum {
//!     A: (u64, u64, u64),
//!     B: (u64, u64, u64),
//!     C: (u64, u64, u64),
//! }
//!
//! match e {
//!     Enum::A((y, _, x)) | Enum::B((_, x, y)) | Enum::C((x, _, y)) => x + y,
//! };
//! ```
//! ```ignore
//! let __matched_value_1 = e;
//! {
//!    let __matched_or_variant_index_1 = if __matched_value_1 is Enum::A {
//!        1 // First OR variant matches.
//!    }
//!    else if __matched_value_1 is Enum::B {
//!        2 // Second OR variant matches.
//!    }
//!    else if __matched_value_1 is Enum::C {
//!        3 // Third OR variant matches.
//!    }
//!    else {
//!        0 // None of the variants matches.
//!    };
//
//!    if __matched_or_variant_index_1 != 0 { // If any of the variants has matched, means if the arm matches.
//!        // Store the values of the variables in a tuple, ordered alphabetically by the variable name.
//!        let __matched_or_variant_variables_1 = if __matched_or_variant_index_1 == 1 {
//!                 // If the first OR variant has matched.
//!                 ((__matched_value_1 as A: (u64, u64, u64)).2, // Take x from the third (2) element of Enum::A.
//!                  (__matched_value_1 as A: (u64, u64, u64)).0) // Take y from the first (0) element of Enum::A.
//!            }
//!            else if __matched_or_variant_index_1 == 2 {
//!                 // If the second OR variant has matched.
//!                 ((__matched_value_1 as B: (u64, u64, u64)).1, // Take x from the second (1) element of Enum::B.
//!                  (__matched_value_1 as B: (u64, u64, u64)).2) // Take y from the third (2) element of Enum::B.
//!            }
//!            else if __matched_or_variant_index_1 == 3 {
//!                 // If the third OR variant has matched.
//!                 ((__matched_value_1 as C: (u64, u64, u64)).0, // Take x from the first (0) element of Enum::C.
//!                  (__matched_value_1 as C: (u64, u64, u64)).2) // Take y from the third (2) element of Enum::C.
//!            }
//!            else {
//!                __revert(14757395258967588865)
//!            };
//!        
//!        // Finally, define the declared variable x and y to take their values from the tuple.
//!        let x = __matched_or_variant_variables_1.0;
//!        let y = __matched_or_variant_variables_1.1;
//!
//!        x + y
//!    }
//!    else {
//!        __revert(14757395258967588866)
//!    }
//!}
//! ```
//!
//! In the case of nested OR patterns, there will be a one `__matched_or_variant_index_<unique suffix>` variable for
//! every encountered OR pattern and they will all be listed above the match arm `if` expression.
//! Also, in that case, the `if-else` definitions of `__matched_or_variant_variables_<unique suffix>` variables will
//! be contained within the `if-else` definitions of their parent `__matched_or_variant_variables_<unique suffix>` variables.
//!
//! For the record, an alternative approach was also considered, in which the tuple variables are declared immediately
//! during the check if variants match. Such tuples would carry a boolean field to communicate if there was a match and
//! in case of a non-match the last tuple would have dummy values from the previous one. This would save us double checking
//! which variant has match, but would mean always instantiating a tuple that is not needed in a case of non-match.
//! In this trade-off we went for the option explained above.
//! Note that we will anyhow optimize match expressions on the IR level.

mod instantiate;
mod matcher;
mod typed_match_branch;
mod typed_match_expression;
mod typed_scrutinee;

use matcher::ReqDeclTree;
