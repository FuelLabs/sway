use indexmap::IndexMap;
use sway_types::{Ident, Span, Spanned};

use crate::language::ty::{self, TyScrutinee};

/// First tuple field is `true` if the variable represented with [Span] is a struct field, otherwise `false`.
pub(crate) type MatchVariable = (bool, Span);

pub(crate) struct MatchVariableDuplicate {
    pub duplicate: MatchVariable,
    pub first_definition: MatchVariable,
}

/// Returns [MatchVariableDuplicate]s for all the duplicates found in the `scrutinee`,
/// or empty [Vec] if there are no duplicate variables in the `scrutinee`.
///
/// Alternatives are what make the algorithm more complex than just straightforward
/// scan of the `scrutinee` for variables of the same name.
/// In case of alternatives, we must have the same variables in all of the alternatives,
/// and these are, of course, not duplicates.
/// But we can still have duplicates within the alternatives, and between the alternatives
/// and the other parts of the match arm.
///
/// Consider the following examples:
///
/// ```ignore
/// Struct { x, y: x, z: x } => x.0,
/// ```
/// The second and the third `x` are the duplicates of the first `x`.
///
/// ```ignore
/// Struct { x, .. } | Struct { x, .. } => x,
/// (Struct { x, .. } | Struct { x, .. }, Struct { y, .. } | Struct { y, .. }) => if x { 0 } else { 1 } + y,
/// ```
/// Here there are no duplicates.
///
/// ```ignore
/// (Struct { x, .. } | Struct { x, .. }, Struct { x, .. } | Struct { x, .. }) => if x { 0 } else { 1 } + y,
/// ```
/// The second `x` is not a duplicate, but the third and fourth are duplicates of the first one.
///
/// ```ignore
/// Struct { x, y: x, z: x } | Struct { x, y: x, z: x } => x,
/// ```
/// The second and the third `x` are duplicates of the first one, and the last two of the fourth one.
///
/// ```ignore
/// (x, Struct { x, .. } | Struct { x, .. }, y) => y,
/// ```
/// The second and the third `x` are duplicates of the first one.
///
/// ```ignore
/// (0, Struct { x, y: x, .. } | Struct { x, .. }, x) => x,
/// ```
/// The second and the last `x` are duplicates of the first one. The third one is not a duplicate.
///
/// ```ignore
/// (x, Struct { x, y: x, .. } | Struct { x, .. }, x) => x,
/// ```
/// All `x`s are duplicates of the first one.
///
/// Why not extend `matcher` to do this analysis?
/// It would be a mixing of concerns and would complicate a clean implementation of the `matcher`.
/// The `matcher`'s purpose is to detect variables and their types and properly bind them.
/// Duplicates are seen as shadowing of variables which is perfectly fine from the `matcher`
/// perspective.
///
/// How the algorithm works?
///
/// For a match arm represented by the `scrutinee` it creates a tree whose nodes are variable names.
/// Variables are added by moving through the match arm left to right.
/// Branching in the tree occurs in the case of alternatives.
/// The algorithm traverses the branches depth-first and collects all the unique duplicates for every branch.
/// Unique means that a duplicate can occur only in one branch.
/// At the end it merges the result of all the branches in a single result.
///
/// The algorithm assumes that the `matcher` already checked the match arm.
/// This gives us the guarantee that every alternative contains the same variables and that for
/// the parts of the match arm that follows alternatives, we need to consider only the left-most
/// alternative as a potential holder of the already defined variables.
///
/// For the examples given above, the corresponding trees look like this:
///
/// ```ignore
/// Struct { x, y: x, z: x } => x.0,
/// - x - x - x
///
/// Struct { x, .. } | Struct { x, .. } => x,
///  / x
/// -
///  \ x <= this is the first, left-most x
///
/// (Struct { x, .. } | Struct { x, .. }, Struct { y, .. } | Struct { y, .. }) => if x { 0 } else { 1 } + y,
///  / x
/// -     / y
///  \ x -
///       \ y <= this is the left-most y
///
/// (Struct { x, .. } | Struct { x, .. }, Struct { x, .. } | Struct { x, .. }) => if x { 0 } else { 1 } + y,
///  / x
/// -     / x
///  \ x -
///       \ x
///
/// Struct { x, y: x, z: x } | Struct { x, y: x, z: x } => x,
///  / x - x - x
/// -
///  \ x - x - x
///
/// (x, Struct { x, .. } | Struct { x, .. }, y) => y,
///    / x
/// -x-
///    \ x - y
///
/// (0, Struct { x, y: x, .. } | Struct { x, .. }, x) => x,
///  / x
/// -
///  \ x - x - x
///
/// (x, Struct { x, y: x, .. } | Struct { x, .. }, x) => x,
///    / x
/// -x-
///    \ x - x - x
///
/// ```
///
/// And here is a some general example with nested alternatives, several variables etc.
///
/// ```ignore
/// (x, y, x | x | x, Struct { x, y, z } | Struct { x: y | y | y, x, z }, z, x | x, z | z | z)
///
///         / x
///        /         / y
/// - x - y - x   / -  y
///        \     /   \ y - x - z
///         \ x -
///              \               / x
///               \ x - y - z - z     / z
///                              \ x -  z
///                                   \ z
/// ```
pub(crate) fn collect_duplicate_match_pattern_variables(
    scrutinee: &TyScrutinee,
) -> Vec<MatchVariableDuplicate> {
    let mut left_most_branch = IndexMap::new();
    let mut branches = vec![];

    recursively_collect_duplicate_variables(&mut branches, &mut left_most_branch, scrutinee);

    branches.push(left_most_branch);

    let mut result = vec![];
    for mut branch in branches {
        for (ident, (is_struct_field, duplicates)) in branch.iter_mut() {
            for duplicate in duplicates {
                result.push(MatchVariableDuplicate {
                    duplicate: (duplicate.0, duplicate.1.clone()),
                    first_definition: (*is_struct_field, ident.span()),
                });
            }
        }
    }

    result.sort_by(|a, b| match a.duplicate.1.partial_cmp(&b.duplicate.1) {
        Some(ord) => ord,
        None => unreachable!(),
    });

    return result;

    fn recursively_collect_duplicate_variables(
        branches: &mut Vec<IndexMap<Ident, (bool, Vec<MatchVariable>)>>,
        left_most_branch: &mut IndexMap<Ident, (bool, Vec<MatchVariable>)>,
        scrutinee: &TyScrutinee,
    ) {
        match &scrutinee.variant {
            ty::TyScrutineeVariant::CatchAll => (),
            ty::TyScrutineeVariant::Variable(ident) => add_variable(left_most_branch, ident, false),
            ty::TyScrutineeVariant::Literal(_) => (),
            ty::TyScrutineeVariant::Constant { .. } => (),
            ty::TyScrutineeVariant::StructScrutinee { fields, .. } => {
                // If a field does not have a scrutinee, the field itself is a variable.
                for field in fields {
                    match &field.scrutinee {
                        Some(scrutinee) => recursively_collect_duplicate_variables(
                            branches,
                            left_most_branch,
                            scrutinee,
                        ),
                        None => add_variable(left_most_branch, &field.field, true),
                    }
                }
            }
            ty::TyScrutineeVariant::Or(scrutinees) => {
                let (first, others) = scrutinees
                    .split_first()
                    .expect("There must be at least two alternatives in TyScrutineeVariant::Or.");

                // For all other alternatives then the first (left-most) one, span a new branch and pass it as a left-most.
                // The new branch contains the identifiers collected so far in the left-most branch,
                // but without duplicates collected so far. We want to have only unique duplicates in each branch.
                for scrutinee in others {
                    let mut branch: IndexMap<Ident, (bool, Vec<(bool, Span)>)> = left_most_branch
                        .iter()
                        .map(|(ident, (is_struct_field, _))| {
                            (
                                ident.clone(),
                                (*is_struct_field, Vec::<(bool, Span)>::new()),
                            )
                        })
                        .collect();

                    recursively_collect_duplicate_variables(branches, &mut branch, scrutinee);

                    branches.push(branch);
                }

                // The variables in the left-most alternative go to the original left-most branch.
                recursively_collect_duplicate_variables(branches, left_most_branch, first);
            }
            ty::TyScrutineeVariant::Tuple(scrutinees) => {
                for scrutinee in scrutinees {
                    match &scrutinee.variant {
                        ty::TyScrutineeVariant::Variable(ident) => {
                            add_variable(left_most_branch, ident, false)
                        }
                        _ => recursively_collect_duplicate_variables(
                            branches,
                            left_most_branch,
                            scrutinee,
                        ),
                    };
                }
            }
            ty::TyScrutineeVariant::EnumScrutinee { value, .. } => {
                recursively_collect_duplicate_variables(branches, left_most_branch, value)
            }
        }

        fn add_variable(
            duplicate_variables: &mut IndexMap<Ident, (bool, Vec<MatchVariable>)>,
            ident: &Ident,
            is_struct_field: bool,
        ) {
            duplicate_variables
                .entry(ident.clone())
                .and_modify(|(_, vec)| vec.push((is_struct_field, ident.span())))
                .or_insert((is_struct_field, vec![]));
        }
    }
}

/// Returns [Ident]s for all match arm variables found in the `scrutinee`,
/// together with the information if the variable is a struct field (true)
/// or not (false), or empty [Vec] if there are no variables declared in
/// the `scrutinee`.
///
/// If the `scrutinee` contains alternatives, and thus a variable is declared
/// multiple times, each occurrence of the variable will be returned.
pub(crate) fn collect_match_pattern_variables(scrutinee: &TyScrutinee) -> Vec<(Ident, bool)> {
    let mut variables = vec![];

    recursively_collect_variables(&mut variables, scrutinee);

    return variables;

    fn recursively_collect_variables(variables: &mut Vec<(Ident, bool)>, scrutinee: &TyScrutinee) {
        match &scrutinee.variant {
            ty::TyScrutineeVariant::CatchAll => (),
            ty::TyScrutineeVariant::Variable(ident) => variables.push((ident.clone(), false)),
            ty::TyScrutineeVariant::Literal(_) => (),
            ty::TyScrutineeVariant::Constant { .. } => (),
            ty::TyScrutineeVariant::StructScrutinee { fields, .. } => {
                // If a field does not have a scrutinee, the field itself is a variable.
                for field in fields {
                    match &field.scrutinee {
                        Some(scrutinee) => recursively_collect_variables(variables, scrutinee),
                        None => variables.push((field.field.clone(), true)),
                    }
                }
            }
            ty::TyScrutineeVariant::Or(scrutinees) => {
                for scrutinee in scrutinees {
                    recursively_collect_variables(variables, scrutinee);
                }
            }
            ty::TyScrutineeVariant::Tuple(scrutinees) => {
                for scrutinee in scrutinees {
                    recursively_collect_variables(variables, scrutinee);
                }
            }
            ty::TyScrutineeVariant::EnumScrutinee { value, .. } => {
                recursively_collect_variables(variables, value)
            }
        }
    }
}
