use std::slice::Iter;

use sway_types::Ident;

use crate::error::err;
use crate::error::ok;
use crate::CompileResult;
use crate::Literal;
use crate::MatchCondition;
use crate::Scrutinee;

#[derive(Clone, Debug)]
enum Usefulness {
    Useful,
    NotUseful,
}

impl Usefulness {
    fn to_bool(&self) -> bool {
        match self {
            Usefulness::Useful => true,
            Usefulness::NotUseful => false,
        }
    }
}

#[derive(Clone, Debug)]
enum ArmType {
    FakeExtraWildcard,
    RealArm,
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
            ) => {
                struct_name1 == struct_name2 && fields1.len() == fields2.len()
                /*
                && fields1
                    .iter()
                    .zip(fields2.iter())
                    .map(|(field1, field2)| field1.has_the_same_constructor(field2))
                    .all(|x| x)
                */
            }
            (Pattern::Tuple(elems1), Pattern::Tuple(elems2)) => {
                elems1.len() == elems2.len()
                /*
                && elems1
                    .iter()
                    .zip(elems2.iter())
                    .map(|(elems1, elems2)| elems1.has_the_same_constructor(elems2))
                    .all(|x| x)
                */
            }
            _ => false,
        }
    }

    fn sub_patterns(&self) -> PatStack {
        match self {
            /*
            Pattern::Literal(lit) => PatStack {
                pats: vec![Pattern::Literal(lit.to_owned())],
            },
            */
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

    fn push(&mut self, other: Pattern) {
        self.pats.push(other)
    }

    fn append(&mut self, others: &mut PatStack) {
        self.pats.append(&mut others.pats);
    }

    fn len(&self) -> usize {
        self.pats.len()
    }

    fn iter(&self) -> Iter<'_, Pattern> {
        self.pats.iter()
    }

    fn is_complete_signature(&self) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut filtered_pats = vec![];
        for pat in self.pats.iter() {
            match pat {
                Pattern::Wildcard => {}
                other => filtered_pats.push(other.clone()),
            }
        }
        if filtered_pats.is_empty() {
            return ok(false, warnings, errors);
        }
        let filtered_pat_stack = PatStack {
            pats: filtered_pats,
        };
        let (first, rest) = check!(
            filtered_pat_stack.split_first(),
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
            pat_stack.push(check!(
                row.first(),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
        ok(pat_stack, warnings, errors)
    }
}

/// Algorithm modeled after this paper:
/// http://moscova.inria.fr/%7Emaranget/papers/warn/warn004.html
/// and resembles the one here:
/// https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/thir/pattern/usefulness/index.html
pub(crate) fn check_match_expression_usefulness(
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
                let arm_is_useful = check!(
                    is_useful(&matrix, &v, ArmType::RealArm),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                matrix.push(v);
                // if an arm is useful then it is reachable
                arms_reachability.push((arm.clone(), arm_is_useful.to_bool()));
            }
        }
        None => unimplemented!(),
    }
    let v = PatStack::from_pattern(Pattern::wild_pattern());
    let wildcard_is_useful = check!(
        is_useful(&matrix, &v, ArmType::FakeExtraWildcard),
        return err(warnings, errors),
        warnings,
        errors
    );
    // if a wildcard case is not useful, then the match arms are exhaustive
    ok(
        (!wildcard_is_useful.to_bool(), arms_reachability),
        warnings,
        errors,
    )
}

fn is_useful(p: &Matrix, q: &PatStack, _arm_type: ArmType) -> CompileResult<Usefulness> {
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
        (0, _) => ok(Usefulness::Useful, warnings, errors),
        (_, 0) => ok(Usefulness::NotUseful, warnings, errors),
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
                        let mut usefulness = false;
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
                            let s_c_q_vector = check!(
                                s_c_k_q.unwrap_vector(),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let res = check!(
                                is_useful(&s_c_k_p, &s_c_q_vector, _arm_type.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            usefulness = usefulness || res.to_bool();
                        }
                        if usefulness {
                            ok(Usefulness::Useful, warnings, errors)
                        } else {
                            ok(Usefulness::NotUseful, warnings, errors)
                        }
                    } else {
                        let d_p = check!(
                            compute_default_matrix(p),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        is_useful(&d_p, &q_rest, _arm_type)
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
                    let s_c_q_vector = check!(
                        s_c_q.unwrap_vector(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    is_useful(&s_c_p, &s_c_q_vector, _arm_type)
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
