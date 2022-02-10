use std::slice::Iter;

use sway_types::Ident;
use sway_types::Span;

use crate::error::err;
use crate::error::ok;
use crate::CompileResult;
use crate::Literal;
use crate::MatchCondition;
use crate::Scrutinee;
use crate::TypeInfo;

#[derive(Clone, Debug)]
enum Usefulness {
    Useful,
    NotUseful,
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
                struct_name1 == struct_name2
                    && fields1.len() == fields2.len()
                    && fields1
                        .iter()
                        .zip(fields2.iter())
                        .map(|(field1, field2)| field1.has_the_same_constructor(field2))
                        .all(|x| x)
            }
            (Pattern::Tuple(elems1), Pattern::Tuple(elems2)) => {
                elems1.len() == elems2.len()
                    && elems1
                        .iter()
                        .zip(elems2.iter())
                        .map(|(elems1, elems2)| elems1.has_the_same_constructor(elems2))
                        .all(|x| x)
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

    fn m_n(&self) -> (usize, usize) {
        let mut n = 0;
        for row in self.rows.iter() {
            let l = row.len();
            if l > n {
                n = l
            }
        }
        (self.rows.len(), n)
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
    for arm in arms.into_iter() {
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
        let arm_is_reachable = match arm_is_useful {
            Usefulness::Useful => true,
            Usefulness::NotUseful => false,
        };
        arms_reachability.push((arm, arm_is_reachable));
    }
    let v = PatStack::from_pattern(Pattern::wild_pattern());
    let wildcard_is_useful = check!(
        is_useful(&matrix, &v, ArmType::FakeExtraWildcard),
        return err(warnings, errors),
        warnings,
        errors
    );
    // if a wildcard case is not useful, then the match arms are exhaustive
    let is_exhaustive = match wildcard_is_useful {
        Usefulness::NotUseful => true,
        Usefulness::Useful => false,
    };
    ok((is_exhaustive, arms_reachability), warnings, errors)
}

fn is_useful(p: &Matrix, q: &PatStack, _arm_type: ArmType) -> CompileResult<Usefulness> {
    //println!("{:?}", p);
    //println!("{:?}", q);
    let mut warnings = vec![];
    let mut errors = vec![];
    match p.m_n() {
        (0, 0) => ok(Usefulness::Useful, warnings, errors),
        (_, 0) => ok(Usefulness::NotUseful, warnings, errors),
        (_, _) => {
            let (q_1, q_rest) = check!(
                q.split_first(),
                return err(warnings, errors),
                warnings,
                errors
            );
            match q_1 {
                Pattern::Wildcard => {
                    let d_p = check!(
                        compute_default_matrix(p),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    is_useful(&d_p, &q_rest, _arm_type)
                }
                q_1 => {
                    let s_c_p = check!(
                        compute_specialized_matrix(&q_1, p),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    println!("{:#?}", s_c_p);
                    let s_c_q = check!(
                        compute_specialized_matrix(&q_1, &Matrix::from_pat_stack(q)),
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
        let row = match p_i_1 {
            Pattern::Wildcard => p_i_rest,
            _ => PatStack::empty(),
        };
        d_p.push(row);
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
