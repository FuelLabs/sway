use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Span;

use crate::{language::ty, type_system::TypeId, Engines};

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
/// https://doc.rust-lang.org/1.75.0/nightly-rustc/rustc_mir_build/thir/pattern/usefulness/index.html
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
/// Usefulness is used in the exhaustivity algorithm.
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
/// the witness for pattern `1` would be the value "1" as the pattern `1`
/// would catch the concrete hypothetical matched value "1" and no other
/// previous cases would have caught it. The witness for pattern `_` is an
/// or-pattern of all of the remaining integers they wouldn't be caught by `0`
/// and `1`, so "2 | .. | MAX".
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
/// the pattern `1` (noted with an arrow) would not have any witnesses
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
/// This algorithm checks if a match expression is exhaustive and if its match
/// arms are reachable by applying the above definitions of usefulness and
/// witnesses. This algorithm sequentially creates a [WitnessReport] for every
/// match arm by calling *U(P, q)*, where *P* is the [Matrix] of patterns seen
/// so far and *q* is the current pattern under investigation for its
/// reachability. A match arm is reachable if its `WitnessReport` is non-empty.
/// Once all existing match arms have been analyzed, the match expression is
/// analyzed for its exhaustivity. *U(P, q)* is called again to create another
/// `WitnessReport`, this time where *P* is the `Matrix` of all patterns and `q`
/// is an imaginary additional wildcard pattern. The match expression is
/// exhaustive if the imaginary additional wildcard pattern has an empty
/// `WitnessReport`.
pub(crate) fn check_match_expression_usefulness(
    handler: &Handler,
    engines: &Engines,
    type_id: TypeId,
    scrutinees: Vec<ty::TyScrutinee>,
    span: Span,
) -> Result<(WitnessReport, Vec<ReachableReport>), ErrorEmitted> {
    let mut matrix = Matrix::empty();
    let mut arms_reachability = vec![];

    // If the provided type does not have a valid constructor and there are no
    // branches in the match expression (i.e. no scrutinees to check), then
    // every scrutinee (i.e. 0 scrutinees) are useful! We return early in this
    // case.
    if !engines
        .te()
        .get(type_id)
        .has_valid_constructor(engines.de())
        && scrutinees.is_empty()
    {
        let witness_report = WitnessReport::NoWitnesses;
        let arms_reachability = vec![];
        return Ok((witness_report, arms_reachability));
    }

    let factory = ConstructorFactory::new(engines, type_id);
    for scrutinee in scrutinees.into_iter() {
        let pat = Pattern::from_scrutinee(scrutinee.clone());
        let v = PatStack::from_pattern(pat);
        let witness_report = is_useful(handler, engines, &factory, &matrix, &v, &span)?;
        matrix.push(v);
        // if an arm has witnesses to its usefulness then it is reachable
        arms_reachability.push(ReachableReport::new(
            witness_report.has_witnesses(),
            scrutinee,
        ));
    }
    let v = PatStack::from_pattern(Pattern::wild_pattern());
    let witness_report = is_useful(handler, engines, &factory, &matrix, &v, &span)?;
    // if a wildcard case has no witnesses to its usefulness, then the match arms are exhaustive
    Ok((witness_report, arms_reachability))
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
/// case returns a witness report that we propagate through the recursive steps.
fn is_useful(
    handler: &Handler,
    engines: &Engines,
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> Result<WitnessReport, ErrorEmitted> {
    let (m, n) = p.m_n(handler, span)?;
    match (m, n) {
        (0, 0) => Ok(WitnessReport::Witnesses(PatStack::fill_wildcards(q.len()))),
        (_, 0) => Ok(WitnessReport::NoWitnesses),
        (_, _) => {
            let c = q.first(handler, span)?;
            let witness_report = match c {
                Pattern::Wildcard => is_useful_wildcard(handler, engines, factory, p, q, span)?,
                Pattern::Or(pats) => is_useful_or(handler, engines, factory, p, q, pats, span)?,
                c => is_useful_constructed(handler, engines, factory, p, q, c, span)?,
            };
            Ok(witness_report)
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
    handler: &Handler,
    engines: &Engines,
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> Result<WitnessReport, ErrorEmitted> {
    // 1. Compute Σ = {c₁, ... , cₙ}, which is the set of constructors that appear
    //    as root constructors of the patterns of *P*'s first column.
    let sigma = p.compute_sigma(handler, span)?;

    // 2. Determine if Σ is a complete signature.
    let is_complete_signature = factory.is_complete_signature(handler, engines, &sigma, span)?;

    if is_complete_signature {
        // 3. If it is a complete signature:

        let mut witness_report = WitnessReport::NoWitnesses;
        let mut pat_stack = PatStack::empty();
        for c_k in sigma.iter() {
            //     3.1. For every every *k* 0..*n*, compute the specialized `Matrix`
            //        *S(cₖ, P)*
            let s_c_k_p = compute_specialized_matrix(handler, c_k, p, q, span)?;

            //     3.2. Compute the specialized `Matrix` *S(cₖ, q)*
            let s_c_k_q = compute_specialized_matrix(
                handler,
                c_k,
                &Matrix::from_pat_stack(q.clone()),
                q,
                span,
            )?;

            // *S(cₖ, q)* may have multiple rows in the case of a or pattern
            // in that case we define: *U(P,((r1∣r2) q2...qn)) = U(P,(r1 q2...qn)) ∨ U(P,(r2 q2...qn))*

            let mut wr = WitnessReport::NoWitnesses;

            for s_c_k_q in s_c_k_q.rows() {
                //     3.3. Recursively compute U(S(cₖ, P), S(cₖ, q))
                let new_wr = is_useful(handler, engines, factory, &s_c_k_p, s_c_k_q, span)?;
                wr = WitnessReport::join_witness_reports(wr, new_wr);
            }

            //     3.4. If the recursive call to (3.3) returns a non-empty witness report,
            //        create a new pattern from *cₖ* and the witness report and a create a
            //        new witness report from the elements not used to create the new
            //        pattern
            //     3.5. Aggregate the new patterns and new witness reports from every call of
            //        (3.4)
            match (&witness_report, wr) {
                (WitnessReport::NoWitnesses, WitnessReport::NoWitnesses) => {}
                (WitnessReport::Witnesses(_), WitnessReport::NoWitnesses) => {}
                (WitnessReport::NoWitnesses, wr @ WitnessReport::Witnesses(_)) => {
                    let (pat, wr) =
                        WitnessReport::split_into_leading_constructor(handler, wr, c_k, span)?;
                    if !pat_stack.contains(&pat) {
                        pat_stack.push(pat);
                    }
                    witness_report = wr;
                }
                (_, wr) => {
                    let (pat, wr) =
                        WitnessReport::split_into_leading_constructor(handler, wr, c_k, span)?;
                    if !pat_stack.contains(&pat) {
                        pat_stack.push(pat);
                    }
                    witness_report = WitnessReport::join_witness_reports(witness_report, wr);
                }
            }
        }

        //     3.6. Transform the aggregated patterns from (3.5) into a single pattern
        //        and prepend it to the aggregated witness report
        match &mut witness_report {
            WitnessReport::NoWitnesses => {}
            witness_report => {
                let pat_stack = Pattern::from_pat_stack(handler, pat_stack, span)?;
                witness_report.add_witness(handler, pat_stack, span)?
            }
        }

        //     7. Return the witness report
        Ok(witness_report)
    } else {
        // 4. If it is not a complete signature:

        //     4.1. Compute the default `Matrix` *D(P)*
        let d_p = compute_default_matrix(handler, p, q, span)?;

        //     4.2. Compute *q'* as \[q₂ ... qₙ*\].
        let (_, q_rest) = q.split_first(handler, span)?;

        //     4.3. Recursively compute *U(D(P), q')*.
        let mut witness_report = is_useful(handler, engines, factory, &d_p, &q_rest, span)?;

        //     4.4. If Σ is empty, create a pattern not present in Σ
        let witness_to_add = if sigma.is_empty() {
            Pattern::Wildcard
        } else {
            factory.create_pattern_not_present(handler, engines, sigma, span)?
        };

        //     4.5. Add this new pattern to the resulting witness report
        match &mut witness_report {
            WitnessReport::NoWitnesses => {}
            witness_report => witness_report.add_witness(handler, witness_to_add, span)?,
        }

        //     4.6. Return the witness report
        Ok(witness_report)
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
    handler: &Handler,
    engines: &Engines,
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    c: Pattern,
    span: &Span,
) -> Result<WitnessReport, ErrorEmitted> {
    // 1. Extract the specialized `Matrix` *S(c, P)*
    let s_c_p = compute_specialized_matrix(handler, &c, p, q, span)?;

    // 2. Extract the specialized `Matrix` *S(c, q)*
    let s_c_q =
        compute_specialized_matrix(handler, &c, &Matrix::from_pat_stack(q.clone()), q, span)?;

    // *S(c, q)* may have multiple rows in the case of a or pattern
    // in that case we define: *U(P,((r1∣r2) q2...qn)) = U(P,(r1 q2...qn)) ∨ U(P,(r2 q2...qn))*
    let mut witness_report = WitnessReport::NoWitnesses;
    for s_c_q in s_c_q.rows() {
        // 3. Recursively compute *U(S(c, P), S(c, q))*
        let wr = is_useful(handler, engines, factory, &s_c_p, s_c_q, span)?;

        witness_report = WitnessReport::join_witness_reports(witness_report, wr);
    }
    Ok(witness_report)
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
    handler: &Handler,
    engines: &Engines,
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    pats: PatStack,
    span: &Span,
) -> Result<WitnessReport, ErrorEmitted> {
    let (_, q_rest) = q.split_first(handler, span)?;
    let mut p = p.clone();
    let mut witness_report = WitnessReport::NoWitnesses;
    for pat in pats.into_iter() {
        // 1. For each *k* 0..*a* compute *q'* as \[*rₖ q₂ ... qₙ*\].
        let mut v = PatStack::from_pattern(pat);
        v.append(&mut q_rest.clone());

        // 2. Compute the witnesses from *U(P, q')*
        let wr = is_useful(handler, engines, factory, &p, &v, span)?;
        p.push(v);

        // 3. Aggregate the witnesses from every *U(P, q')*
        witness_report = WitnessReport::join_witness_reports(witness_report, wr);
    }
    Ok(witness_report)
}

/// Given a `Matrix` *P*, constructs the default `Matrix` *D(P). This is done by
/// sequentially computing the rows of *D(P)*.
///
/// Intuition: A default `Matrix` is a transformation upon *P* that "shrinks"
/// the rows of *P* depending on if the row is able to generally match all
/// patterns in a default case.
fn compute_default_matrix(
    handler: &Handler,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> Result<Matrix, ErrorEmitted> {
    let mut d_p = Matrix::empty();
    for p_i in p.rows().iter() {
        d_p.append(&mut compute_default_matrix_row(handler, p_i, q, span)?);
    }
    let (m, n) = d_p.m_n(handler, span)?;
    if m > 0 && n != (q.len() - 1) {
        return Err(handler.emit_err(CompileError::Internal(
            "D(P) matrix is misshapen",
            span.clone(),
        )));
    }
    Ok(d_p)
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
fn compute_default_matrix_row(
    handler: &Handler,
    p_i: &PatStack,
    q: &PatStack,
    span: &Span,
) -> Result<Vec<PatStack>, ErrorEmitted> {
    let mut rows: Vec<PatStack> = vec![];
    let (p_i_1, mut p_i_rest) = p_i.split_first(handler, span)?;
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
            let d_p = compute_default_matrix(handler, &m, q, span)?;
            rows.append(&mut d_p.into_rows());
        }
        // 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)*:
        //     1. no row is produced
        _ => {}
    }
    Ok(rows)
}

/// Given a constructor *c* and a `Matrix` *P*, constructs the specialized
/// `Matrix` *S(c, P)*. This is done by sequentially computing the rows of
/// *S(c, P)*.
///
/// Intuition: A specialized `Matrix` is a transformation upon *P* that
/// "unwraps" the rows of *P* depending on if they are congruent with *c*.
fn compute_specialized_matrix(
    handler: &Handler,
    c: &Pattern,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> Result<Matrix, ErrorEmitted> {
    let mut s_c_p = Matrix::empty();

    if let Pattern::Or(cpats) = c {
        for cpat in cpats.iter() {
            let mut rows = compute_specialized_matrix(handler, cpat, p, q, span)?.into_rows();

            s_c_p.append(&mut rows);
        }
        return Ok(s_c_p);
    }

    for p_i in p.rows().iter() {
        s_c_p.append(&mut compute_specialized_matrix_row(
            handler, c, p_i, q, span,
        )?);
    }
    let (m, n) = s_c_p.m_n(handler, span)?;
    if m > 0 && n != (c.a() + q.len() - 1) {
        return Err(handler.emit_err(CompileError::Internal(
            "S(c,P) matrix is misshapen",
            span.clone(),
        )));
    }
    Ok(s_c_p)
}

/// Given a constructor *c* and a `PatStack` *pⁱ* from `Matrix` *P*, compute the
/// resulting row of the specialized `Matrix` *S(c, P)*.
///
/// Intuition: a row in the specialized [Matrix] "expands itself" or "eliminates
/// itself" depending on if its possible to further "drill down" into the
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
    handler: &Handler,
    c: &Pattern,
    p_i: &PatStack,
    q: &PatStack,
    span: &Span,
) -> Result<Vec<PatStack>, ErrorEmitted> {
    let mut rows: Vec<PatStack> = vec![];
    let (p_i_1, mut p_i_rest) = p_i.split_first(handler, span)?;
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
            let s_c_p = compute_specialized_matrix(handler, c, &m, q, span)?;
            rows.append(&mut s_c_p.into_rows());
        }
        other => {
            if c.has_the_same_constructor(&other) {
                // 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* == *c'*:
                //     1.1. the resulting row equals \[*r₁ ... rₐ pⁱ₂ ... pⁱₙ*\]
                let mut row: PatStack = other.sub_patterns(handler, span)?;
                row.append(&mut p_i_rest);
                rows.push(row);
            }
            // 2. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* != *c'*:
            //     2.1. no row is produced
        }
    }
    Ok(rows)
}
