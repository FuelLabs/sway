use std::slice::Iter;
use std::vec::IntoIter;

use sway_types::Ident;

use crate::error::err;
use crate::error::ok;
use crate::CompileResult;
use crate::Literal;
use crate::MatchCondition;
use crate::Scrutinee;
use crate::TypeInfo;

enum WitnessReport {
    NoWitnesses,
    Witnesses(PatStack),
}

impl WitnessReport {
    fn join_witness_reports(a: WitnessReport, b: WitnessReport) -> Self {
        match (a, b) {
            (WitnessReport::NoWitnesses, WitnessReport::NoWitnesses) => WitnessReport::NoWitnesses,
            (WitnessReport::NoWitnesses, WitnessReport::Witnesses(wits)) => {
                WitnessReport::Witnesses(wits)
            }
            (WitnessReport::Witnesses(wits), WitnessReport::NoWitnesses) => {
                WitnessReport::Witnesses(wits)
            }
            (WitnessReport::Witnesses(wits1), WitnessReport::Witnesses(mut wits2)) => {
                let mut wits = wits1;
                wits.append(&mut wits2);
                WitnessReport::Witnesses(wits)
            }
        }
    }

    fn resolve_with_constructor(
        witness_report: WitnessReport,
        c: &Pattern,
        n: usize,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match witness_report {
            WitnessReport::NoWitnesses => unimplemented!(),
            WitnessReport::Witnesses(witnesses) => {
                let (rs, mut ps) = witnesses.split_at(witnesses.len() - n + 1);
                let mut pat_stack = PatStack::empty();
                pat_stack.push(check!(
                    Pattern::from_constructor_and_arguments(c, rs),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                pat_stack.append(&mut ps);
                ok(WitnessReport::Witnesses(pat_stack), vec![], vec![])
            }
        }
    }

    fn add_witness(&mut self, witness: Pattern) -> CompileResult<()> {
        match self {
            WitnessReport::NoWitnesses => unimplemented!(),
            WitnessReport::Witnesses(witnesses) => {
                witnesses.prepend(witness);
                ok((), vec![], vec![])
            }
        }
    }

    fn has_witnesses(&self) -> bool {
        match self {
            WitnessReport::NoWitnesses => false,
            WitnessReport::Witnesses(witnesses) => true, // !witnesses.is_empty()
        }
    }
}

#[derive(Clone, Debug)]
enum Pattern {
    Wildcard,
    Literal(Literal),
    Struct(StructPattern),
    Tuple(PatStack),
}

impl Pattern {
    fn from_match_condition(match_condition: MatchCondition) -> Self {
        match match_condition {
            MatchCondition::CatchAll(_) => Pattern::Wildcard,
            MatchCondition::Scrutinee(scrutinee) => Pattern::from_scrutinee(scrutinee),
        }
    }

    fn from_scrutinee(scrutinee: Scrutinee) -> Self {
        match scrutinee {
            Scrutinee::Variable { .. } => Pattern::Wildcard,
            Scrutinee::Literal { value, .. } => Pattern::Literal(value),
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => {
                let pats = fields
                    .into_iter()
                    .map(|x| match x.scrutinee {
                        Some(scrutinee) => Pattern::from_scrutinee(scrutinee),
                        None => Pattern::Wildcard,
                    })
                    .collect::<Vec<_>>();
                Pattern::Struct(StructPattern {
                    struct_name,
                    fields: PatStack { pats },
                })
            }
            Scrutinee::Tuple { elems, .. } => Pattern::Tuple(PatStack {
                pats: elems
                    .into_iter()
                    .map(Pattern::from_scrutinee)
                    .collect::<Vec<_>>(),
            }),
            _ => unreachable!(),
        }
    }

    fn from_constructor_and_arguments(c: &Pattern, args: PatStack) -> CompileResult<Self> {
        let warnings = vec![];
        let errors = vec![];
        match c {
            Pattern::Tuple(elems) => {
                if elems.len() != args.len() {
                    unimplemented!()
                }
                ok(Pattern::Tuple(args), warnings, errors)
            }
            _ => unimplemented!(),
        }
    }

    fn wild_pattern() -> Self {
        Pattern::Wildcard
    }

    fn a(&self) -> usize {
        match self {
            Pattern::Wildcard => 0,
            Pattern::Literal(_) => 1,
            Pattern::Struct(StructPattern { fields, .. }) => fields.len(),
            Pattern::Tuple(elems) => elems.len(),
        }
    }

    fn has_the_same_constructor(&self, other: &Pattern) -> bool {
        match (self, other) {
            (Pattern::Wildcard, Pattern::Wildcard) => true,
            (Pattern::Literal(lit1), Pattern::Literal(lit2)) => match (lit1, lit2) {
                (Literal::U8(x), Literal::U8(y)) => x == y,
                (Literal::U16(x), Literal::U16(y)) => x == y,
                (Literal::U32(x), Literal::U32(y)) => x == y,
                (Literal::U64(x), Literal::U64(y)) => x == y,
                (Literal::B256(x), Literal::B256(y)) => x == y,
                (Literal::Boolean(x), Literal::Boolean(y)) => x == y,
                (Literal::Byte(x), Literal::Byte(y)) => x == y,
                (Literal::Numeric(x), Literal::Numeric(y)) => x == y,
                (Literal::String(x), Literal::String(y)) => x == y,
                _ => false,
            },
            (
                Pattern::Struct(StructPattern {
                    struct_name: struct_name1,
                    fields: fields1,
                }),
                Pattern::Struct(StructPattern {
                    struct_name: struct_name2,
                    fields: fields2,
                }),
            ) => struct_name1 == struct_name2 && fields1.len() == fields2.len(),
            (Pattern::Tuple(elems1), Pattern::Tuple(elems2)) => elems1.len() == elems2.len(),
            _ => false,
        }
    }

    fn sub_patterns(&self) -> PatStack {
        match self {
            Pattern::Struct(StructPattern { fields, .. }) => fields.to_owned(),
            Pattern::Tuple(elems) => elems.to_owned(),
            _ => PatStack::empty(),
        }
    }
}

#[derive(Clone, Debug)]
struct StructPattern {
    struct_name: Ident,
    fields: PatStack,
}

#[derive(Clone, Debug)]
struct PatStack {
    pats: Vec<Pattern>,
}

impl PatStack {
    fn empty() -> Self {
        PatStack { pats: vec![] }
    }

    fn from_pattern(pattern: Pattern) -> Self {
        PatStack {
            pats: vec![pattern],
        }
    }

    fn compute_patterns_not_present(type_info: &TypeInfo, patterns: PatStack) -> Self {
        unimplemented!()
    }

    fn first(&self) -> CompileResult<Pattern> {
        match self.pats.first() {
            Some(first) => ok(first.to_owned(), vec![], vec![]),
            None => unimplemented!(),
        }
    }

    fn split_first(&self) -> CompileResult<(Pattern, PatStack)> {
        match self.pats.split_first() {
            Some((first, pat_stack_contents)) => {
                let pat_stack = PatStack {
                    pats: pat_stack_contents.to_vec(),
                };
                ok((first.to_owned(), pat_stack), vec![], vec![])
            }
            None => unimplemented!(),
        }
    }

    fn split_at(&self, n: usize) -> (PatStack, PatStack) {
        let (a, b) = self.pats.split_at(n);
        let x = PatStack { pats: a.to_vec() };
        let y = PatStack { pats: b.to_vec() };
        (x, y)
    }

    fn push(&mut self, other: Pattern) {
        self.pats.push(other)
    }

    fn append(&mut self, others: &mut PatStack) {
        self.pats.append(&mut others.pats);
    }

    fn prepend(&mut self, other: Pattern) {
        self.pats.insert(0, other);
    }

    fn len(&self) -> usize {
        self.pats.len()
    }

    fn is_empty(&self) -> bool {
        self.pats.is_empty()
    }

    fn iter(&self) -> Iter<'_, Pattern> {
        self.pats.iter()
    }

    fn into_iter(self) -> IntoIter<Pattern> {
        self.pats.into_iter()
    }

    fn is_complete_signature(&self) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        if self.pats.is_empty() {
            return ok(false, warnings, errors);
        }
        let (first, rest) = check!(
            self.split_first(),
            return err(warnings, errors),
            warnings,
            errors
        );
        match first {
            // its assumed that no one is every going to list every single literal
            Pattern::Literal(_) => ok(false, warnings, errors),
            Pattern::Tuple(elems) => {
                for pat in rest.iter() {
                    if !pat.has_the_same_constructor(&Pattern::Tuple(elems.clone())) {
                        return ok(false, warnings, errors);
                    }
                }
                ok(true, warnings, errors)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug)]
struct Matrix {
    rows: Vec<PatStack>,
}

impl Matrix {
    fn empty() -> Self {
        Matrix { rows: vec![] }
    }

    fn from_pat_stack(pat_stack: &PatStack) -> Self {
        Matrix {
            rows: vec![pat_stack.to_owned()],
        }
    }

    fn push(&mut self, row: PatStack) {
        self.rows.push(row);
    }

    fn append(&mut self, rows: &mut Vec<PatStack>) {
        self.rows.append(rows);
    }

    fn rows(&self) -> &Vec<PatStack> {
        &self.rows
    }

    fn m_n(&self) -> CompileResult<(usize, usize)> {
        let warnings = vec![];
        let errors = vec![];
        let first = match self.rows.first() {
            Some(first) => first,
            None => return ok((0, 0), warnings, errors),
        };
        let n = first.len();
        for row in self.rows.iter().skip(1) {
            let l = row.len();
            if l != n {
                unimplemented!()
            }
        }
        ok((self.rows.len(), n), warnings, errors)
    }

    fn is_a_vector(&self) -> bool {
        self.rows.len() == 1
    }

    fn unwrap_vector(&self) -> CompileResult<PatStack> {
        if self.rows.len() > 1 {
            unimplemented!()
        }
        match self.rows.first() {
            Some(first) => ok(first.clone(), vec![], vec![]),
            None => ok(PatStack::empty(), vec![], vec![]),
        }
    }

    fn compute_sigma(&self) -> CompileResult<PatStack> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut pat_stack = PatStack::empty();
        for row in self.rows.iter() {
            let first = check!(row.first(), return err(warnings, errors), warnings, errors);
            match first {
                Pattern::Wildcard => {}
                other => pat_stack.push(other),
            }
        }
        ok(pat_stack, warnings, errors)
    }
}

/// Algorithm modeled after this paper:
/// http://moscova.inria.fr/%7Emaranget/papers/warn/warn004.html
/// and resembles the one here:
/// https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/thir/pattern/usefulness/index.html
pub(crate) fn check_match_expression_usefulness(
    type_info: TypeInfo,
    arms: Vec<MatchCondition>,
) -> CompileResult<(bool, Vec<(MatchCondition, bool)>)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut matrix = Matrix::empty();
    let mut arms_reachability = vec![];
    match arms.split_first() {
        Some((first_arm, arms_rest)) => {
            matrix.push(PatStack::from_pattern(Pattern::from_match_condition(
                first_arm.clone(),
            )));
            arms_reachability.push((first_arm.clone(), true));
            for arm in arms_rest.iter() {
                let pattern = Pattern::from_match_condition(arm.clone());
                let v = PatStack::from_pattern(pattern);
                let witness_report = check!(
                    is_useful(&type_info, &matrix, &v),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                matrix.push(v);
                // if an arm has witnesses to its usefulness then it is reachable
                arms_reachability.push((arm.clone(), witness_report.has_witnesses()));
            }
        }
        None => unimplemented!(),
    }
    let v = PatStack::from_pattern(Pattern::wild_pattern());
    let witness_report = check!(
        is_useful(&type_info, &matrix, &v),
        return err(warnings, errors),
        warnings,
        errors
    );
    // if a wildcard case has no witnesses to its usefulness, then the match arms are exhaustive
    ok(
        (!witness_report.has_witnesses(), arms_reachability),
        warnings,
        errors,
    )
}

fn is_useful(type_info: &TypeInfo, p: &Matrix, q: &PatStack) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (m, n) = check!(p.m_n(), return err(warnings, errors), warnings, errors);
    /*
    if n != q.len() {
        println!("p: {:?}", p);
        println!("q: {:?}", q);
        unimplemented!()
    }
    */
    match (m, n) {
        (0, 0) => ok(
            WitnessReport::Witnesses(PatStack::empty()),
            warnings,
            errors,
        ),
        (_, 0) => ok(WitnessReport::NoWitnesses, warnings, errors),
        (_, _) => {
            let (c, q_rest) = check!(
                q.split_first(),
                return err(warnings, errors),
                warnings,
                errors
            );
            match c {
                Pattern::Wildcard => {
                    let sigma = check!(
                        p.compute_sigma(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let is_complete_signature = check!(
                        sigma.is_complete_signature(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if is_complete_signature {
                        let mut joined_witness_report = WitnessReport::NoWitnesses;
                        for c_k in sigma.iter() {
                            let s_c_k_p = check!(
                                compute_specialized_matrix(c_k, p),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let s_c_k_q = check!(
                                compute_specialized_matrix(c_k, &Matrix::from_pat_stack(q)),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let s_c_k_q = check!(
                                s_c_k_q.unwrap_vector(),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let witness_report = check!(
                                is_useful(type_info, &s_c_k_p, &s_c_k_q),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let witness_report = check!(
                                WitnessReport::resolve_with_constructor(witness_report, c_k, n),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            joined_witness_report = WitnessReport::join_witness_reports(
                                joined_witness_report,
                                witness_report,
                            )
                        }
                        ok(joined_witness_report, warnings, errors)
                    } else {
                        let d_p = check!(
                            compute_default_matrix(p),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let mut witness_report = check!(
                            is_useful(type_info, &d_p, &q_rest),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        if sigma.len() == 0 {
                            check!(
                                witness_report.add_witness(Pattern::Wildcard),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                        } else {
                            let constructors_not_present =
                                PatStack::compute_patterns_not_present(type_info, sigma);
                            for constructor in constructors_not_present.into_iter() {
                                check!(
                                    witness_report.add_witness(constructor),
                                    return err(warnings, errors),
                                    warnings,
                                    errors
                                );
                            }
                        }
                        ok(witness_report, warnings, errors)
                    }
                }
                c => {
                    let s_c_p = check!(
                        compute_specialized_matrix(&c, p),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let s_c_q = check!(
                        compute_specialized_matrix(&c, &Matrix::from_pat_stack(q)),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let s_c_q = check!(
                        s_c_q.unwrap_vector(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    is_useful(type_info, &s_c_p, &s_c_q)
                }
            }
        }
    }
}

fn compute_default_matrix(p: &Matrix) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut d_p = Matrix::empty();
    for p_i in p.rows().iter() {
        let (p_i_1, p_i_rest) = check!(
            p_i.split_first(),
            return err(warnings, errors),
            warnings,
            errors
        );
        if let Pattern::Wildcard = p_i_1 {
            d_p.push(p_i_rest);
        }
    }
    ok(d_p, warnings, errors)
}

fn compute_specialized_matrix(q: &Pattern, p: &Matrix) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut s_c_p = Matrix::empty();
    for p_i in p.rows().iter() {
        let (p_i_1, mut p_i_rest) = check!(
            p_i.split_first(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut rows = compute_specialized_matrix_row(q, &p_i_1, &mut p_i_rest);
        s_c_p.append(&mut rows);
    }
    let (m, _n) = check!(s_c_p.m_n(), return err(warnings, errors), warnings, errors);
    if p.is_a_vector() && m > 1 {
        unimplemented!()
    }
    ok(s_c_p, warnings, errors)
}

fn compute_specialized_matrix_row(
    q: &Pattern,
    p_i_1: &Pattern,
    p_i_rest: &mut PatStack,
) -> Vec<PatStack> {
    let mut rows: Vec<PatStack> = vec![];
    match p_i_1 {
        Pattern::Wildcard => {
            let mut row: PatStack = PatStack::empty();
            for _ in 0..q.a() {
                row.push(Pattern::Wildcard);
            }
            row.append(p_i_rest);
            rows.push(row);
        }
        other => {
            if q.has_the_same_constructor(other) {
                let mut row: PatStack = PatStack::empty();
                row.append(&mut other.sub_patterns());
                row.append(p_i_rest);
                rows.push(row);
            }
        }
    }
    rows
}
