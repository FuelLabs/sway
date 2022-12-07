use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    error::{err, ok},
    language::ty,
    type_system::TypeId,
    CompileResult, Engines,
};

use super::{
    constructor_factory::ConstructorFactory, matrix::Matrix, patstack::PatStack, pattern::Pattern,
    reachable_report::ReachableReport, witness_report::WitnessReport,
};

/// Given the arms of a match expression, checks to see if the arms are
/// exhaustive and checks to see if each arm is reachable.
///
/// ---
///
/// Modeled after this paper:
/// http://moscova.inria.fr/%7Emaranget/papers/warn/warn004.html
///
/// Implemented in Rust here:
/// https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/thir/pattern/usefulness/index.html
///
/// ---
///
/// In general, match expressions are constructed as so:
///
/// ```ignore
/// match value {
///     pattern => result,
///     pattern => result,
///     pattern => result
/// }
/// ```
///
/// where `value` is the "matched value", and each `pattern => result` is a
/// "match arm", and `value` will "match" one of the `patterns` in the match
/// arms. A match happens when a `pattern` has the same "type" and "shape" as
/// the `value`, at some level of generality. For example `1` will match `1`,
/// `a`, and `_`, but will not match `2`.
///
/// The goal of this algorithm is to:
/// 1. Check to see if the arms are exhaustive (i.e. all cases for which the
///    matched value could be are included in the provided arms)
/// 2. Check to see if each arm is reachable (i.e. if each arm is able to
///    "catch" at least on hypothetical matched value without the previous arms
///    "catching" all the values)
///
/// # `Pattern`
///
/// A `Pattern` is an object that is able to be matched upon. A `Pattern` is
/// semantically constructed of a "constructor" and its "arguments". For
/// example, given the tuple `(1,2)` "a tuple with 2 elements" is the
/// constructor and "1, 2" are the arguments. Given the u64 `2`, "2" is the
/// constructor and it has no arguments (you can think of this by imagining
/// that u64 is the enum type and each u64 value is a variant of that enum type,
/// making the value itself a constructor).
///
/// `Pattern`s are semantically categorized into three categories: wildcard
/// patterns (the catchall pattern `_` and variable binding patterns like `a`),
/// constructed patterns (`(1,2)` aka "a tuple with 2 elements" with arguments
/// "1, 2"), and or-patterns (`1 | 2 | .. `).
///
/// `Pattern`s are used in the exhaustivity algorithm.
///
/// # Usefulness
///
/// A pattern is "useful" when it covers at least one case of a possible
/// matched value that had been left uncovered by previous patterns.
///
/// For example, given:
///
/// ```ignore
/// let x = true;
/// match x {
///     true => ..,
///     false => ..
/// }
/// ```
///
/// the pattern `false` is useful because it covers at least one case (i.e.
/// `false`) that had been left uncovered by the previous patterns.
///
/// Given:
///
/// ```ignore
/// let x = 5;
/// match x {
///     0 => ..,
///     1 => ..,
///     _ => ..
/// }
/// ```
///
/// the pattern `_` is useful because it covers at least one case (i.e. all
/// cases other than 0 and 1) that had been left uncovered by the previous
/// patterns.
///
/// In another example, given:
///
/// ```ignore
/// let x = 5;
/// match x {
///     0 => ..,
///     1 => ..,
///     1 => .., // <--
///     _ => ..
/// }
/// ```
///
/// the pattern `1` (noted with an arrow) is not useful as it does not cover any
/// case that is not already covered by a previous pattern.
///
/// Given:
///
/// ```ignore
/// let x = 5;
/// match x {
///     0 => ..,
///     1 => ..,
///     _ => ..,
///     2 => .. // <--
/// }
/// ```
///
/// the pattern `2` is not useful as it does not cover any case that is not
/// already covered by a previous pattern. Even though there is only one pattern
/// `2`, any cases that the pattern `2` covers would previously be caught by the
/// catchall pattern.
///
/// Usefulness used in the exhaustivity algorithm.
///
/// # Witnesses
///
/// A "witness" to a pattern is a concrete example of a matched value that would
/// be caught by that pattern that would not have been caught by previous
/// patterns.
///
/// For example, given:
///
/// ```ignore
/// let x = 5;
/// match x {
///     0 => ..,
///     1 => ..,
///     _ => ..
/// }
/// ```
///
/// the witness for pattern `1` would be the pattern "1" as the pattern `1`
/// would catch the concrete hypothetical matched value "1" and no other
/// previous cases would have caught it. The witness for pattern `_` is an
/// or-pattern of all of the remaining integers they wouldn't be caught by `0`
/// and `1`, so "3 | .. | MAX".
///
/// Given:
///
/// ```ignore
/// let x = 5;
/// match x {
///     0 => ..,
///     1 => ..,
///     1 => .., // <--
///     _ => ..
/// }
/// ```
///
/// the pattern `1` (noted with an arrow) would not have any witnesses as there
/// that it catches that are not caught by previous patterns.
///
/// # Putting it all together
///
/// Given the definitions above, we can say several things:
///
/// 1. A pattern is useful when it has witnesses to its usefulness (i.e. it has
///    at least one hypothetical value that it catches that is not caught by
///    previous patterns).
/// 2. A match arm is reachable when its pattern is useful.
/// 3. A match expression is exhaustive when, if you add an additional wildcard
///    pattern to the existing patterns, this new wildcard pattern is not
///    useful.
///
/// # Details
///
/// This algorithm checks is a match expression is exhaustive and if its match
/// arms are reachable by applying the above definitions of usefulness and
/// witnesses. This algorithm sequentionally creates a `WitnessReport` for every
/// match arm by calling *U(P, q)*, where *P* is the `Matrix` of patterns seen
/// so far and *q* is the current pattern under investigation for its
/// reachability. A match arm is reachable if its `WitnessReport` is non-empty.
/// Once all existing match arms have been analyzed, the match expression is
/// analyzed for its exhaustivity. *U(P, q)* is called again to create another
/// `WitnessReport`, this time where *P* is the `Matrix` of all patterns and `q`
/// is an imaginary additional wildcard pattern. The match expression is
/// exhaustive if the imaginary additional wildcard pattern has an empty
/// `WitnessReport`.
pub(crate) fn check_match_expression_usefulness(
    engines: Engines<'_>,
    type_id: TypeId,
    scrutinees: Vec<ty::TyScrutinee>,
    span: Span,
) -> CompileResult<(WitnessReport, Vec<ReachableReport>)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut matrix = Matrix::empty();
    let mut arms_reachability = vec![];
    let factory = check!(
        ConstructorFactory::new(engines.te(), type_id, &span),
        return err(warnings, errors),
        warnings,
        errors
    );
    for scrutinee in scrutinees.into_iter() {
        let pat = check!(
            Pattern::from_scrutinee(scrutinee.clone()),
            return err(warnings, errors),
            warnings,
            errors
        );
        let v = PatStack::from_pattern(pat);
        let witness_report = check!(
            is_useful(engines, &factory, &matrix, &v, &span),
            return err(warnings, errors),
            warnings,
            errors
        );
        matrix.push(v);
        // if an arm has witnesses to its usefulness then it is reachable
        arms_reachability.push(ReachableReport::new(
            witness_report.has_witnesses(),
            scrutinee,
        ));
    }
    let v = PatStack::from_pattern(Pattern::wild_pattern());
    let witness_report = check!(
        is_useful(engines, &factory, &matrix, &v, &span),
        return err(warnings, errors),
        warnings,
        errors
    );
    // if a wildcard case has no witnesses to its usefulness, then the match arms are exhaustive
    ok((witness_report, arms_reachability), warnings, errors)
}

/// Given a `Matrix` *P* and a `PatStack` *q*, computes a `WitnessReport` from
/// algorithm *U(P, q)*.
///
/// This recursive algorithm is basically an induction proof with 2 base cases.
/// The first base case is when *P* is the empty `Matrix`. In this case, we
/// return a witness report where the witnesses are wildcard patterns for every
/// element of *q*. The second base case is when *P* has at least one row but
/// does not have any columns. In this case, we return a witness report with no
/// witnesses. This case indicates exhaustivity. The induction case covers
/// everything else, and what we do for induction depends on what the first
/// element of *q* is. Depending on if the first element of *q* is a wildcard
/// pattern, or-pattern, or constructed pattern we do something different. Each
/// case returns a witness report that we propogate through the recursive steps.
fn is_useful(
    engines: Engines<'_>,
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (m, n) = check!(p.m_n(span), return err(warnings, errors), warnings, errors);
    match (m, n) {
        (0, 0) => ok(
            WitnessReport::Witnesses(PatStack::fill_wildcards(q.len())),
            warnings,
            errors,
        ),
        (_, 0) => ok(WitnessReport::NoWitnesses, warnings, errors),
        (_, _) => {
            let c = check!(
                q.first(span),
                return err(warnings, errors),
                warnings,
                errors
            );
            let witness_report = match c {
                Pattern::Wildcard => check!(
                    is_useful_wildcard(engines, factory, p, q, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Pattern::Or(pats) => check!(
                    is_useful_or(engines, factory, p, q, pats, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                c => check!(
                    is_useful_constructed(engines, factory, p, q, c, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
            };
            ok(witness_report, warnings, errors)
        }
    }
}

/// Computes a witness report from *U(P, q)* when *q* is a wildcard pattern.
///
/// Because *q* is a wildcard pattern, this means we are checking to see if the
/// wildcard pattern is useful given *P*. We can do this by investigating the
/// first column Σ of *P*. If Σ is a complete signature (that is if Σ contains
/// every constructor for the type of elements in Σ), then we can recursively
/// compute the witnesses for every element of Σ and aggregate them. If Σ is not
/// a complete signature, then we can compute the default `Matrix` for *P* (i.e.
/// a version of *P* that is agnostic to *c*) and recursively compute the
/// witnesses for if q is useful given the new default `Matrix`.
///
/// ---
///
/// 1. Compute Σ = {c₁, ... , cₙ}, which is the set of constructors that appear
///    as root constructors of the patterns of *P*'s first column.
/// 2. Determine if Σ is a complete signature.
/// 3. If it is a complete signature:
///     1. For every every *k* 0..*n*, compute the specialized `Matrix`
///        *S(cₖ, P)*
///     2. Compute the specialized `Matrix` *S(cₖ, q)*
///     3. Recursively compute U(S(cₖ, P), S(cₖ, q))
///     4. If the recursive call to (3.3) returns a non-empty witness report,
///        create a new pattern from *cₖ* and the witness report and a create a
///        new witness report from the elements not used to create the new
///        pattern
///     5. Aggregate a new patterns and new witness reports from every call of
///        (3.4)
///     6. Transform the aggregated patterns from (3.5) into a single pattern
///        and prepend it to the aggregated witness report
///     7. Return the witness report
/// 4. If it is not a complete signature:
///     1. Compute the default `Matrix` *D(P)*
///     2. Compute *q'* as \[q₂ ... qₙ*\].
///     3. Recursively compute *U(D(P), q')*.
///     4. If Σ is empty, create a pattern not present in Σ
///     5. Add this new pattern to the resulting witness report
///     6. Return the witness report
fn is_useful_wildcard(
    engines: Engines<'_>,
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // 1. Compute Σ = {c₁, ... , cₙ}, which is the set of constructors that appear
    //    as root constructors of the patterns of *P*'s first column.
    let sigma = check!(
        p.compute_sigma(span),
        return err(warnings, errors),
        warnings,
        errors
    );

    // 2. Determine if Σ is a complete signature.
    let is_complete_signature = check!(
        factory.is_complete_signature(engines, &sigma, span),
        return err(warnings, errors),
        warnings,
        errors
    );

    if is_complete_signature {
        // 3. If it is a complete signature:

        let mut witness_report = WitnessReport::NoWitnesses;
        let mut pat_stack = PatStack::empty();
        for c_k in sigma.iter() {
            //     3.1. For every every *k* 0..*n*, compute the specialized `Matrix`
            //        *S(cₖ, P)*
            let s_c_k_p = check!(
                compute_specialized_matrix(c_k, p, span),
                return err(warnings, errors),
                warnings,
                errors
            );

            //     3.2. Compute the specialized `Matrix` *S(cₖ, q)*
            let s_c_k_q = check!(
                compute_specialized_matrix(c_k, &Matrix::from_pat_stack(q.clone()), span),
                return err(warnings, errors),
                warnings,
                errors
            );
            let s_c_k_q = check!(
                s_c_k_q.unwrap_vector(span),
                return err(warnings, errors),
                warnings,
                errors
            );

            //     3.3. Recursively compute U(S(cₖ, P), S(cₖ, q))
            let wr = check!(
                is_useful(engines, factory, &s_c_k_p, &s_c_k_q, span),
                return err(warnings, errors),
                warnings,
                errors
            );

            //     3.4. If the recursive call to (3.3) returns a non-empty witness report,
            //        create a new pattern from *cₖ* and the witness report and a create a
            //        new witness report from the elements not used to create the new
            //        pattern
            //     3.5. Aggregate the new patterns and new witness reports from every call of
            //        (3.4)
            match (&witness_report, wr) {
                (WitnessReport::NoWitnesses, WitnessReport::NoWitnesses) => {}
                (WitnessReport::NoWitnesses, wr) => {
                    let (pat, wr) = check!(
                        WitnessReport::split_into_leading_constructor(wr, c_k, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if !pat_stack.contains(&pat) {
                        pat_stack.push(pat);
                    }
                    witness_report = wr;
                }
                (_, wr) => {
                    let (pat, _) = check!(
                        WitnessReport::split_into_leading_constructor(wr, c_k, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if !pat_stack.contains(&pat) {
                        pat_stack.push(pat);
                    }
                }
            }
        }

        //     3.6. Transform the aggregated patterns from (3.5) into a single pattern
        //        and prepend it to the aggregated witness report
        match &mut witness_report {
            WitnessReport::NoWitnesses => {}
            witness_report => {
                let pat_stack = check!(
                    Pattern::from_pat_stack(pat_stack, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                check!(
                    witness_report.add_witness(pat_stack, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
        }

        //     7. Return the witness report
        ok(witness_report, warnings, errors)
    } else {
        // 4. If it is not a complete signature:

        //     4.1. Compute the default `Matrix` *D(P)*
        let d_p = check!(
            compute_default_matrix(p, span),
            return err(warnings, errors),
            warnings,
            errors
        );

        //     4.2. Compute *q'* as \[q₂ ... qₙ*\].
        let (_, q_rest) = check!(
            q.split_first(span),
            return err(warnings, errors),
            warnings,
            errors
        );

        //     4.3. Recursively compute *U(D(P), q')*.
        let mut witness_report = check!(
            is_useful(engines, factory, &d_p, &q_rest, span),
            return err(warnings, errors),
            warnings,
            errors
        );

        //     4.4. If Σ is empty, create a pattern not present in Σ
        let witness_to_add = if sigma.is_empty() {
            Pattern::Wildcard
        } else {
            check!(
                factory.create_pattern_not_present(engines, sigma, span),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        //     4.5. Add this new pattern to the resulting witness report
        match &mut witness_report {
            WitnessReport::NoWitnesses => {}
            witness_report => check!(
                witness_report.add_witness(witness_to_add, span),
                return err(warnings, errors),
                warnings,
                errors
            ),
        }

        //     4.6. Return the witness report
        ok(witness_report, warnings, errors)
    }
}

/// Computes a witness report from *U(P, q)* when *q* is a constructed pattern
/// *c(r₁, ..., rₐ)*.
///
/// Given a specialized `Matrix` that specializes *P* to *c* and another
/// specialized `Matrix` that specializes *q* to *c*, recursively compute if the
/// latter `Matrix` is useful to the former.
///
/// ---
///
/// 1. Extract the specialized `Matrix` *S(c, P)*
/// 2. Extract the specialized `Matrix` *S(c, q)*
/// 3. Recursively compute *U(S(c, P), S(c, q))*
fn is_useful_constructed(
    engines: Engines<'_>,
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    c: Pattern,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // 1. Extract the specialized `Matrix` *S(c, P)*
    let s_c_p = check!(
        compute_specialized_matrix(&c, p, span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let (s_c_p_m, s_c_p_n) = check!(
        s_c_p.m_n(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    if s_c_p_m > 0 && s_c_p_n != (c.a() + q.len() - 1) {
        errors.push(CompileError::Internal(
            "S(c,P) matrix is misshappen",
            span.clone(),
        ));
        return err(warnings, errors);
    }

    // 2. Extract the specialized `Matrix` *S(c, q)*
    let s_c_q = check!(
        compute_specialized_matrix(&c, &Matrix::from_pat_stack(q.clone()), span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let s_c_q = check!(
        s_c_q.unwrap_vector(span),
        return err(warnings, errors),
        warnings,
        errors
    );

    // 3. Recursively compute *U(S(c, P), S(c, q))*
    is_useful(engines, factory, &s_c_p, &s_c_q, span)
}

/// Computes a witness report from *U(P, q)* when *q* is an or-pattern
/// *(r₁ | ... | rₐ)*.
///
/// Compute the witness report for each element of q and aggregate them
/// together.
///
/// ---
///
/// 1. For each *k* 0..*a* compute *q'* as \[*rₖ q₂ ... qₙ*\].
/// 2. Compute the witnesses from *U(P, q')*
/// 3. Aggregate the witnesses from every *U(P, q')*
fn is_useful_or(
    engines: Engines<'_>,
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    pats: PatStack,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (_, q_rest) = check!(
        q.split_first(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut p = p.clone();
    let mut witness_report = WitnessReport::Witnesses(PatStack::empty());
    for pat in pats.into_iter() {
        // 1. For each *k* 0..*a* compute *q'* as \[*rₖ q₂ ... qₙ*\].
        let mut v = PatStack::from_pattern(pat);
        v.append(&mut q_rest.clone());

        // 2. Compute the witnesses from *U(P, q')*
        let wr = check!(
            is_useful(engines, factory, &p, &v, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        p.push(v);

        // 3. Aggregate the witnesses from every *U(P, q')*
        witness_report = WitnessReport::join_witness_reports(witness_report, wr);
    }
    ok(witness_report, warnings, errors)
}

/// Given a `Matrix` *P*, constructs the default `Matrix` *D(P). This is done by
/// sequentially computing the rows of *D(P)*.
///
/// Intuition: A default `Matrix` is a transformation upon *P* that "shrinks"
/// the rows of *P* depending on if the row is able to generally match all
/// patterns in a default case.
fn compute_default_matrix(p: &Matrix, span: &Span) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut d_p = Matrix::empty();
    for p_i in p.rows().iter() {
        d_p.append(&mut check!(
            compute_default_matrix_row(p_i, span),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }
    ok(d_p, warnings, errors)
}

/// Given a `PatStack` *pⁱ* from `Matrix` *P*, compute the resulting row of the
/// default `Matrix` *D(P)*.
///
/// A row in the default `Matrix` "shrinks itself" or "eliminates itself"
/// depending on if its possible to make general claims the first element of the
/// row *pⁱ₁*. It is possible to make a general claim *pⁱ₁* when *pⁱ₁* is the
/// wildcard pattern (in which case it could match anything) and when *pⁱ₁* is
/// an or-pattern (in which case we can do recursion while pretending that the
/// or-pattern is itself a `Matrix`). A row "eliminates itself" when *pⁱ₁* is a
/// constructed pattern (in which case it could only make a specific constructed
/// pattern and we could not make any general claims about it).
///
/// ---
///
/// Rows are defined according to the first component of the row:
///
/// 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)*:
///     1. no row is produced
/// 2. *pⁱ₁* is a wildcard pattern:
///     1. the resulting row equals \[pⁱ₂ ... pⁱₙ*\]
/// 3. *pⁱ₁* is an or-pattern *(r₁ | ... | rₐ)*:
///     1. Construct a new `Matrix` *P'*, where given *k* 0..*a*, the rows of
///        *P'* are defined as \[*rₖ pⁱ₂ ... pⁱₙ*\] for every *k*.
///     2. The resulting rows are the rows obtained from calling the recursive
///        *D(P')*
fn compute_default_matrix_row(p_i: &PatStack, span: &Span) -> CompileResult<Vec<PatStack>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut rows: Vec<PatStack> = vec![];
    let (p_i_1, mut p_i_rest) = check!(
        p_i.split_first(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    match p_i_1 {
        Pattern::Wildcard => {
            // 2. *pⁱ₁* is a wildcard pattern:
            //     1. the resulting row equals \[pⁱ₂ ... pⁱₙ*\]
            let mut row = PatStack::empty();
            row.append(&mut p_i_rest);
            rows.push(row);
        }
        Pattern::Or(pats) => {
            // 3. *pⁱ₁* is an or-pattern *(r₁ | ... | rₐ)*:
            //     1. Construct a new `Matrix` *P'*, where given *k* 0..*a*, the rows of
            //        *P'* are defined as \[*rₖ pⁱ₂ ... pⁱₙ*\] for every *k*.
            let mut m = Matrix::empty();
            for pat in pats.iter() {
                let mut m_row = PatStack::from_pattern(pat.clone());
                m_row.append(&mut p_i_rest.clone());
                m.push(m_row);
            }
            //     2. The resulting rows are the rows obtained from calling the recursive
            //        *D(P')*
            let d_p = check!(
                compute_default_matrix(&m, span),
                return err(warnings, errors),
                warnings,
                errors
            );
            rows.append(&mut d_p.into_rows());
        }
        // 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)*:
        //     1. no row is produced
        _ => {}
    }
    ok(rows, warnings, errors)
}

/// Given a constructor *c* and a `Matrix` *P*, constructs the specialized
/// `Matrix` *S(c, P)*. This is done by sequentially computing the rows of
/// *S(c, P)*.
///
/// Intuition: A specialized `Matrix` is a transformation upon *P* that
/// "unwraps" the rows of *P* depending on if they are congruent with *c*.
fn compute_specialized_matrix(c: &Pattern, p: &Matrix, span: &Span) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut s_c_p = Matrix::empty();
    for p_i in p.rows().iter() {
        s_c_p.append(&mut check!(
            compute_specialized_matrix_row(c, p_i, span),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }
    let (m, _) = check!(
        s_c_p.m_n(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    if p.is_a_vector() && m > 1 {
        errors.push(CompileError::Internal(
            "S(c,p) must be a vector",
            span.clone(),
        ));
        return err(warnings, errors);
    }
    ok(s_c_p, warnings, errors)
}

/// Given a constructor *c* and a `PatStack` *pⁱ* from `Matrix` *P*, compute the
/// resulting row of the specialized `Matrix` *S(c, P)*.
///
/// Intuition: a row in the specialized `Matrix` "expands itself" or "eliminates
/// itself" depending on if its possible to furthur "drill down" into the
/// elements of *P* given a *c* that we are specializing for. It is possible to
/// "drill down" when the first element of a row of *P* *pⁱ₁* matches *c* (in
/// which case it is possible to "drill down" into the arguments for *pⁱ₁*),
/// when *pⁱ₁* is the wildcard case (in which case it is possible to "drill
/// down" into "fake" arguments for *pⁱ₁* as it does not matter if *c* matches
/// or not), and when *pⁱ₁* is an or-pattern (in which case we can do recursion
/// while pretending that the or-pattern is itself a `Matrix`). A row
/// "eliminates itself" when *pⁱ₁* does not match *c* (in which case it is not
/// possible to "drill down").
///
/// ---
///
/// Rows are defined according to the first component of the row:
///
/// 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* == *c'*:
///     1. the resulting row equals \[*r₁ ... rₐ pⁱ₂ ... pⁱₙ*\]
/// 2. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* != *c'*:
///     1. no row is produced
/// 3. *pⁱ₁* is a wildcard pattern and the number of sub-patterns in *c* is *a*:
///     1. the resulting row equals \[*_₁ ... _ₐ pⁱ₂ ... pⁱₙ*\]
/// 4. *pⁱ₁* is an or-pattern *(r₁ | ... | rₐ)*:
///     1. Construct a new `Matrix` *P'* where, given *k* 0..*a*, the rows of
///        *P'* are defined as \[*rₖ pⁱ₂ ... pⁱₙ*\] for every *k*
///     2. The resulting rows are the rows obtained from calling the recursive
///        *S(c, P')*
fn compute_specialized_matrix_row(
    c: &Pattern,
    p_i: &PatStack,
    span: &Span,
) -> CompileResult<Vec<PatStack>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut rows: Vec<PatStack> = vec![];
    let (p_i_1, mut p_i_rest) = check!(
        p_i.split_first(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    match p_i_1 {
        Pattern::Wildcard => {
            // 3. *pⁱ₁* is a wildcard pattern and the number of sub-patterns in *c* is *a*:
            //     3.1. the resulting row equals \[*_₁ ... _ₐ pⁱ₂ ... pⁱₙ*\]
            let mut row: PatStack = PatStack::fill_wildcards(c.a());
            row.append(&mut p_i_rest);
            rows.push(row);
        }
        Pattern::Or(pats) => {
            // 4. *pⁱ₁* is an or-pattern *(r₁ | ... | rₐ)*:
            //     4.1. Construct a new `Matrix` *P'* where, given *k* 0..*a*, the rows of
            //        *P'* are defined as \[*rₖ pⁱ₂ ... pⁱₙ*\] for every *k*
            let mut m = Matrix::empty();
            for pat in pats.iter() {
                let mut m_row = PatStack::from_pattern(pat.clone());
                m_row.append(&mut p_i_rest.clone());
                m.push(m_row);
            }

            //     4.2. The resulting rows are the rows obtained from calling the recursive
            //        *S(c, P')*
            let s_c_p = check!(
                compute_specialized_matrix(c, &m, span),
                return err(warnings, errors),
                warnings,
                errors
            );
            rows.append(&mut s_c_p.into_rows());
        }
        other => {
            if c.has_the_same_constructor(&other) {
                // 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* == *c'*:
                //     1.1. the resulting row equals \[*r₁ ... rₐ pⁱ₂ ... pⁱₙ*\]
                let mut row: PatStack = check!(
                    other.sub_patterns(span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                row.append(&mut p_i_rest);
                rows.push(row);
            }
            // 2. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* != *c'*:
            //     2.1. no row is produced
        }
    }
    ok(rows, warnings, errors)
}
