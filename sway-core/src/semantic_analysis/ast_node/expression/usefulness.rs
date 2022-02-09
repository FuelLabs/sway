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
enum ArmType {
    FakeExtraWildcard,
    RealArm,
}

#[derive(Clone, Debug)]
enum ReachabilityStatus {
    Unknown,
    Known(Reachability),
}

#[derive(Clone, Debug)]
enum Reachability {
    Reachable,
    Unreachable,
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
                (Literal::U8(_), Literal::U8(_))
                | (Literal::U16(_), Literal::U16(_))
                | (Literal::U32(_), Literal::U32(_))
                | (Literal::U64(_), Literal::U64(_))
                | (Literal::B256(_), Literal::B256(_))
                | (Literal::Boolean(_), Literal::Boolean(_))
                | (Literal::Byte(_), Literal::Byte(_))
                | (Literal::Numeric(_), Literal::Numeric(_))
                | (Literal::String(_), Literal::String(_)) => true,
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
                        .map(|(field1, field2)| field1.has_the_same_constructor(&field2))
                        .all(|x| x == true)
            }
            (Pattern::Tuple(elems1), Pattern::Tuple(elems2)) => {
                elems1.len() == elems2.len()
                    && elems1
                        .iter()
                        .zip(elems2.iter())
                        .map(|(elems1, elems2)| elems1.has_the_same_constructor(&elems2))
                        .all(|x| x == true)
            }
            _ => false,
        }
    }

    fn sub_patterns(&self) -> PatStack {
        match self {
            Pattern::Wildcard => PatStack::empty(),
            Pattern::Literal(lit) => PatStack {
                pats: vec![Pattern::Literal(lit.to_owned())],
            },
            Pattern::Struct(StructPattern { fields, .. }) => fields.to_owned(),
            Pattern::Tuple(elems) => elems.to_owned(),
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
    patterns: Vec<PatStack>,
}

impl Matrix {
    fn empty() -> Self {
        Matrix { patterns: vec![] }
    }

    fn push(&mut self, other: PatStack) {
        self.patterns.push(other);
    }

    fn append(&mut self, others: &mut Vec<PatStack>) {
        self.patterns.append(others);
    }

    fn rows(&self) -> &Vec<PatStack> {
        &self.patterns
    }
}

/// Algorithm modeled after this documentation:
/// https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/thir/pattern/usefulness/index.html
/// and this paper:
/// http://moscova.inria.fr/%7Emaranget/papers/warn/index.html
pub(crate) fn check_match_expression_usefulness(
    _parent_type: TypeInfo,
    arms: Vec<MatchCondition>,
    _span: Span,
) -> CompileResult<()> {
    let arms_as_patterns = arms
        .into_iter()
        .map(Pattern::from_match_condition)
        .collect::<Vec<_>>();
    let mut matrix = Matrix::empty();
    let arms_usefulness = arms_as_patterns
        .into_iter()
        .map(|pattern| {
            let v = PatStack::from_pattern(pattern);
            is_useful(&matrix, &v, ArmType::RealArm);
            matrix.push(v);
            /*
            let is_reachable = match pattern.reachability.clone() {
                ReachabilityStatus::Unknown => unimplemented!(),
                ReachabilityStatus::Known(is_reachable) => is_reachable,
            };
            */
            unimplemented!()
        })
        .collect::<Vec<_>>();
    let v = PatStack::from_pattern(Pattern::wild_pattern());
    let usefulness = is_useful(&matrix, &v, ArmType::FakeExtraWildcard);
    unimplemented!()
}

fn is_useful(matrix: &Matrix, v: &PatStack, arm_type: ArmType) {
    unimplemented!()
}

fn compute_specialized_matrix(q: &Pattern, P: &Matrix) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut s_c_p = Matrix::empty();
    for p_i in P.rows().iter() {
        let (p_i_1, mut p_i_rest) = check!(
            p_i.split_first(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut rows = check!(
            compute_specialized_matrix_row(q, &p_i_1, &mut p_i_rest),
            return err(warnings, errors),
            warnings,
            errors
        );
        s_c_p.append(&mut rows);
    }
    ok(s_c_p, warnings, errors)
}

fn compute_specialized_matrix_row(
    q: &Pattern,
    p_i_1: &Pattern,
    p_i_rest: &mut PatStack,
) -> CompileResult<Vec<PatStack>> {
    let warnings = vec![];
    let errors = vec![];
    let mut rows: Vec<PatStack> = vec![];
    let a = q.a();
    match p_i_1 {
        Pattern::Wildcard => {
            let mut row: PatStack = PatStack::empty();
            for _ in 0..a {
                row.push(Pattern::Wildcard);
            }
            row.append(p_i_rest);
            rows.push(row);
        }
        other => {
            if q.has_the_same_constructor(other) {
                let mut row: PatStack = PatStack::empty();
                row.append(&mut other.sub_patterns());
            }
        }
    }
    ok(rows, warnings, errors)
}
