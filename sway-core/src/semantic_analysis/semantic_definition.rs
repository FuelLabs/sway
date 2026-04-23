use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use hashbrown::HashMap;
use sway_error::{error::CompileError, handler::Handler};
use sway_types::{integer_bits::IntegerBits, Span};

use crate::{
    engine_threading::{PartialEqWithEngines, PartialEqWithEnginesContext},
    unify::unifier::{Unifier, UnifyKind},
    Engines, IncludeSelf, TreatNumericAs, TypeEngine, TypeId, TypeInfo, UnifyCheck,
};

pub enum TypeEvent {
    // When a type reachs its non changeable state. It is 100% concrete as it cannot change anymore
    OnNonChangeableState,
}

#[derive(Default)]
pub struct SemanticDefinitionEngine {
    defs: Vec<Arc<SemanticDefinition>>,
}

impl SemanticDefinitionEngine {
    pub fn new_def(&mut self) -> SemanticDefinitionId {
        let def = Arc::new(SemanticDefinition {
            inner: Mutex::new(SemanticDefinitionInner {
                events: HashMap::new(),
                callbacks: Arc::new(Mutex::new(CallbackRegistry { callbacks: vec![] })),
                unifications: vec![],
            }),
        });
        let id = SemanticDefinitionId(self.defs.len());
        self.defs.push(def);
        id
    }

    pub fn get(&self, id: SemanticDefinitionId) -> &Arc<SemanticDefinition> {
        self.defs.get(id.0).expect("invalid SemanticDefinitionId")
    }
}

pub struct SemanticDefinitionSolver<'a> {
    engines: &'a Engines,
    handler: &'a Handler,
    def: &'a SemanticDefinition,
    replacements: HashMap<TypeId, TypeId>,
    tid_map: HashMap<TypeId, TypeId>,
}

enum UnificationChange {
    LeftChanged,
    RightChanged,
}

fn unify(engines: &Engines, left_tid: TypeId, right_tid: TypeId) -> Option<UnificationChange> {
    let left = engines.te().get(left_tid);
    let right = engines.te().get(right_tid);
    match (left.as_ref(), right.as_ref()) {
        // (TypeInfo::Unknown, TypeInfo::Unknown) => None,
        (
            TypeInfo::UnsignedInteger(IntegerBits::Eight),
            TypeInfo::UnsignedInteger(IntegerBits::Eight),
        ) => None,
        (TypeInfo::UnsignedInteger(IntegerBits::Eight), TypeInfo::Numeric) => {
            engines
                .te()
                .replace(engines, right_tid, TypeInfo::clone(&left));
            Some(UnificationChange::RightChanged)
        }
        (TypeInfo::Unknown, right) => {
            engines.te().replace(engines, left_tid, right.clone());
            Some(UnificationChange::LeftChanged)
        }
        _ => todo!("{:?} {:?}", engines.help_out(left), engines.help_out(right)),
    }
}

/// Is changeable every type that has:
/// UnknownGeneric, TypeInfo::Placeholder, TypeInfo::TraitType, TypeInfo::Numeric and TypeInfo::Unknown
// TODO: SemanticDefinition should not suppor TypeInfo::Custom { .. }
// as all types should have already be resolved
// TODO: TypeInfo::TypeParam?
fn is_changeable(engines: &Engines, tid: TypeId) -> bool {
    tid.extract_inner_types(engines, IncludeSelf::Yes)
        .into_iter()
        .any(|tid| {
            let type_info = engines.te().get(tid);
            matches!(
                type_info.as_ref(),
                TypeInfo::UnknownGeneric { .. }
                    | TypeInfo::Placeholder(..)
                    | TypeInfo::TraitType { .. }
                    | TypeInfo::Numeric
                    | TypeInfo::Unknown
            )
        })
}

impl<'a> SemanticDefinitionSolver<'a> {
    pub fn push_replacement(&mut self, tid: TypeId, new_tid: TypeId) -> &mut Self {
        self.replacements.insert(tid, new_tid);
        self
    }

    pub fn solve(self) -> SolveResult {
        let SemanticDefinitionSolver {
            engines,
            handler,
            def,
            mut replacements,
            tid_map,
        } = self;

        // Adjust the semantic definition usind tid_map
        let mut def = {
            let def = def.inner.lock().unwrap();
            SemanticDefinitionInner::clone(&def)
        };

        for (k, v) in tid_map {
            if let Some(events) = def.events.remove(&k) {
                def.events.insert(v, events);
            }

            def.unifications.retain_mut(|(l, r)| {
                if *l == k {
                    *l = v;
                }

                if *r == k {
                    *r = v;
                }

                true
            });
        }

        // Start solving
        let mut not_changeable_anymore_worklist = vec![];

        let mut steps = 10;
        let mut worklist = replacements.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
        while let Some(tid) = worklist.pop() {
            // call events on all concretized types
            for not_changeable_tid in not_changeable_anymore_worklist.drain(..) {
                if let Some(actions) = def.events.get(&not_changeable_tid).cloned() {
                    for action in actions.iter() {
                        match action {
                            InnerTypeEvent::OnNonChangeableState { callback_id } => {
                                eprintln!(
                                    "    calling OnNonChangeableState callback for {:?}({:?})",
                                    not_changeable_tid,
                                    engines.help_out(not_changeable_tid)
                                );
                                let mut registry = def.callbacks.lock().unwrap();
                                let cb = &mut registry.callbacks[*callback_id];
                                match cb {
                                    TypedCallbacks::TypeNonChangeableState { f } => {
                                        (f)(engines, handler, not_changeable_tid);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // try to solve for tid

            steps -= 1;
            if steps <= 0 {
                break;
            }
            eprintln!(
                "current: {tid:?}({:?}); worklist: {worklist:?}",
                engines.help_out(tid)
            );
            let tid_is_changeable = is_changeable(engines, tid);
            eprintln!("    is_changeable: {:?}", tid_is_changeable);

            if let Some(replace_tid) = replacements.remove(&tid) {
                eprintln!(
                    "    replacement: \n        {tid:?}({:?}) <- {replace_tid:?}({:?})",
                    engines.help_out(tid),
                    engines.help_out(replace_tid)
                );

                let replace_type_info = engines.te().get(replace_tid);
                let replace_tid_is_changeable = is_changeable(engines, replace_tid);
                engines
                    .te()
                    .replace(engines, tid, replace_type_info.as_ref().clone());

                if tid_is_changeable && !replace_tid_is_changeable {
                    eprintln!("        {:?} is not changeable anymore", tid);
                    not_changeable_anymore_worklist.push(tid);
                }
            }

            for (left, right) in def
                .unifications
                .iter()
                .filter(|(a, b)| *a == tid || *b == tid)
            {
                eprintln!(
                    "    unification:\n        {left:?}({:?}) with {right:?}({:?})",
                    engines.help_out(left),
                    engines.help_out(right)
                );

                let left_is_changeable_before = is_changeable(engines, *left);
                let right_is_changeable_before = is_changeable(engines, *right);

                match unify(engines, *left, *right) {
                    Some(UnificationChange::LeftChanged) => {
                        eprintln!(
                            "        left changed to {:?}({:?})",
                            left,
                            engines.help_out(left)
                        );
                        worklist.push(*left);
                    }
                    Some(UnificationChange::RightChanged) => {
                        eprintln!(
                            "        right changed to {:?}({:?})",
                            right,
                            engines.help_out(right)
                        );
                        worklist.push(*right);
                    }
                    None => {
                        eprintln!("        no work needed");
                    }
                }

                let left_is_changeable_after = is_changeable(engines, *left);
                let right_is_changeable_after = is_changeable(engines, *right);

                if left_is_changeable_before && !left_is_changeable_after {
                    not_changeable_anymore_worklist.push(*left);
                    eprintln!("        {:?} is not changeable anymore", left);
                }

                if right_is_changeable_before && !right_is_changeable_after {
                    not_changeable_anymore_worklist.push(*right);
                    eprintln!("        {:?} is not changeable anymore", right);
                }
            }
        }

        // check solving state
        eprintln!("checking solving state");
        let missing_replacements = replacements.len() > 0;
        let mut still_changeable_types = BTreeSet::new();
        for (l, r) in def.unifications.iter() {
            if is_changeable(engines, *l) {
                still_changeable_types.insert(l);
            }

            if is_changeable(engines, *r) {
                still_changeable_types.insert(r);
            }
        }

        for t in still_changeable_types.iter() {
            eprintln!("    {:?}({:?}) still changeable", t, engines.help_out(t));
        }

        let r = if !missing_replacements && still_changeable_types.is_empty() {
            SolveResult::Solved
        } else {
            SolveResult::Incomplete
        };

        eprintln!("    {r:?}");

        r
    }

    fn check_solved() -> SolveResult {
        SolveResult::Solved
    }

    /// Modify the `SemanticDefinition` in place (do not change anything inside the SemanticDefinitionEngine) to
    /// accomodate `get_method_safe_for_unify` and others mechanisms that change TypeId inside decls
    fn push_tid_map(&mut self, tid_map: HashMap<TypeId, TypeId>) {
        self.tid_map = tid_map;
    }
}

#[derive(Debug)]
pub enum SolveResult {
    Solved,
    Incomplete,
}

#[derive(Clone, Debug)]
enum InnerTypeEvent {
    OnNonChangeableState { callback_id: usize },
}

enum TypedCallbacks {
    TypeNonChangeableState {
        f: Box<dyn FnMut(&Engines, &Handler, TypeId)>,
    },
}

struct CallbackRegistry {
    callbacks: Vec<TypedCallbacks>,
}

#[derive(Clone)]
pub struct SemanticDefinitionInner {
    unifications: Vec<(TypeId, TypeId)>,
    events: HashMap<TypeId, Vec<InnerTypeEvent>>,
    callbacks: Arc<Mutex<CallbackRegistry>>,
}

impl std::fmt::Debug for SemanticDefinitionInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticDefinitionInner")
            .field("unifications", &self.unifications)
            .field("events", &self.events)
            .finish()
    }
}

#[derive(Debug)]
pub struct SemanticDefinition {
    inner: Mutex<SemanticDefinitionInner>,
}

impl SemanticDefinition {
    fn solver<'a>(
        &'a self,
        engines: &'a Engines,
        handler: &'a Handler,
    ) -> SemanticDefinitionSolver<'a> {
        SemanticDefinitionSolver {
            engines,
            handler,
            def: self.clone(),
            replacements: HashMap::default(),
            tid_map: HashMap::new(),
        }
    }

    pub fn when_type_reaches_non_changeable_state(
        &self,
        tid: TypeId,
        e: TypeEvent,
        callback: impl 'static + FnMut(&Engines, &Handler, TypeId),
    ) {
        let mut inner = self.inner.lock().unwrap();

        let callback_id = {
            let mut registry = inner.callbacks.lock().unwrap();
            registry
                .callbacks
                .push(TypedCallbacks::TypeNonChangeableState {
                    f: Box::new(callback),
                });
            registry.callbacks.len() - 1
        };

        let v = inner.events.entry(tid).or_default();
        match e {
            TypeEvent::OnNonChangeableState => {
                v.push(InnerTypeEvent::OnNonChangeableState { callback_id });
            }
        }
    }

    pub fn push_type_unify(&self, a: TypeId, b: TypeId) {
        let mut inner = self.inner.lock().unwrap();
        inner.unifications.push((a, b));
    }
}

#[derive(Clone, Copy)]
pub struct SemanticDefinitionId(usize);

#[test]
fn semantic_definition_solve() {
    let engines = Engines::default();
    let handler = Handler::default();

    let mut sde = SemanticDefinitionEngine::default();

    // this will be done on TyFunctionDecl::type_check
    let sdid = sde.new_def();

    // let a = 0x100;
    // Vec::<u8>::new().push(a);

    // this will be on type_check_literal for the RHS
    let sd = sde.get(sdid);
    let value = 0x100 as u64;
    let tid0 = engines.te().insert(&engines, TypeInfo::Numeric, None);
    sd.when_type_reaches_non_changeable_state(
        tid0,
        TypeEvent::OnNonChangeableState,
        move |engines, handler, tid| {
            let final_type_info = engines.te().get(tid);
            let max_value = match final_type_info.as_ref() {
                TypeInfo::UnsignedInteger(IntegerBits::Eight) => u8::MAX as u64,
                TypeInfo::UnsignedInteger(IntegerBits::Sixteen) => u16::MAX as u64,
                TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo) => u32::MAX as u64,
                TypeInfo::UnsignedInteger(IntegerBits::SixtyFour) => u64::MAX as u64,
                x => {
                    handler.emit_err(CompileError::InternalOwned(
                        format!("Invalid final type `{x:?}`"),
                        Span::dummy(),
                    ));
                    return;
                }
            };

            if value > max_value {
                handler.emit_err(CompileError::Internal("Do not fit", Span::dummy()));
            }
        },
    );

    // this will be on type_check_variable_declaration for the LHS
    let tid1 = engines.te().insert(&engines, TypeInfo::Unknown, None);
    sd.push_type_unify(tid1, tid0);

    // Simulate get_method_safe_for_unify
    engines.te().start_capturing_duplicates();
    engines.te().duplicate(&engines, tid0);
    let tid1_new = engines.te().duplicate(&engines, tid1);
    let tid_map = engines.te().end_capturing_duplicates().unwrap();

    // This will be done when monomorphizing
    let tid2 = engines.te().insert(
        &engines,
        TypeInfo::UnsignedInteger(IntegerBits::Eight),
        None,
    );
    let mut solver = sd.solver(&engines, &handler);
    solver.push_tid_map(tid_map);
    solver.push_replacement(tid1_new, tid2);
    let r = solver.solve();
    assert!(matches!(r, SolveResult::Solved));

    dbg!(handler.consume());
}
