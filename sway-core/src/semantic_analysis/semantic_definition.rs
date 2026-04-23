use std::sync::{Arc, Mutex};

use hashbrown::HashMap;
use sway_error::{error::CompileError, handler::Handler};
use sway_types::{Span, integer_bits::IntegerBits};

use crate::{Engines, TreatNumericAs, TypeEngine, TypeId, TypeInfo, UnifyCheck, unify::unifier::{Unifier, UnifyKind}};

pub enum TypeEvent {
    // When a type reachs its non changeable state. It is 100% concrete as it cannot change anymore
    OnNonChangeableState
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
                callbacks: Arc::new(Mutex::new(CallbackRegistry {
                    callbacks: vec![],
                })),
                unify: vec![],
            })
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

enum UnificationChange{
    LeftChanged,
    RightChanged,
}

fn unify(engines: &Engines, left_tid: TypeId, right_tid: TypeId) -> Option<UnificationChange> {
    let left = engines.te().get(left_tid);
    let right = engines.te().get(right_tid);
    match (left.as_ref(), right.as_ref()) {
        // (TypeInfo::Unknown, TypeInfo::Unknown) => None,
        (TypeInfo::UnsignedInteger(IntegerBits::Eight), TypeInfo::UnsignedInteger(IntegerBits::Eight)) => {
            None
        }
        (TypeInfo::UnsignedInteger(IntegerBits::Eight), TypeInfo::Numeric) => {
            engines.te().replace(engines, right_tid, TypeInfo::clone(&left));
            Some(UnificationChange::RightChanged)
        },
        (TypeInfo::Unknown, right) => {
            engines.te().replace(engines, left_tid, right.clone());
            Some(UnificationChange::LeftChanged)
        },
        _ => todo!("{:?} {:?}", engines.help_out(left), engines.help_out(right)),
    }
}

impl<'a> SemanticDefinitionSolver<'a> {
    pub fn replace_type(&mut self, tid: TypeId, new_tid: TypeId) -> &mut Self {
        self.replacements.insert(tid, new_tid);
        self
    }

    pub fn solve(self) {
        let SemanticDefinitionSolver {
            engines,
            handler,
            def,
            replacements,
            tid_map,
        } = self;

        // Adjust the semantic definition usind tid_map
        let mut inner = self.def.inner.lock().unwrap();
        let def = SemanticDefinitionInner::clone(&inner);
        for (k, v) in tid_map {
            if let Some(events) = inner.events.remove(&k) {
                inner.events.insert(k, events);
            }

            inner.unify.retain_mut(|(l, r)| {
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
        let mut steps = 10;
        let mut worklist = replacements.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
        while let Some(tid) = worklist.pop() {
            steps -= 1;
            if steps <= 0 {
                break;
            }
            eprintln!("{tid:?} {:?}; q: {worklist:?}", engines.help_out(tid));
            let type_info_is_concrete = tid.is_concrete(engines, TreatNumericAs::Abstract);

            if let Some(replace_tid) = replacements.get(&tid) {
                eprintln!("    replace: {tid:?}({:?}) with {replace_tid:?}({:?})", engines.help_out(tid), engines.help_out(replace_tid));
                
                if !tid.eq(replace_tid) {
                    let replace_type_info = engines.te().get(*replace_tid);
                    engines.te().replace(engines, tid, replace_type_info.as_ref().clone());
                }                
            }

            for (left, right) in inner.unify.iter().filter(|(a, b)| *a == tid || *b == tid) {
                eprintln!("    unify: {left:?}({:?}) with {right:?}({:?})", engines.help_out(left), engines.help_out(right));
                match unify(engines, *left, *right) {
                    Some(UnificationChange::LeftChanged) => {
                        eprintln!("        left changed");
                        worklist.push(*left);
                    },
                    Some(UnificationChange::RightChanged) => {
                        eprintln!("        right changed");
                        worklist.push(*right);
                    },
                    None => {
                        eprintln!("        no work needed");
                    },
                }
            }

            if let Some(actions) = inner.events.get(&tid).cloned() {
                for action in actions.iter() {
                    match action {
                        InnerTypeEvent::OnNonChangeableState { callback_id } => {
                            let type_info_is_concrete_after = tid.is_concrete(engines, TreatNumericAs::Abstract);
                            if !type_info_is_concrete && type_info_is_concrete_after {
                                eprintln!("    calling OnNonChangeableState callback");
                                let mut registry = inner.callbacks.lock().unwrap();
                                let cb = &mut registry.callbacks[*callback_id];
                                cb(engines, handler);
                            }
                        },
                    }
                }
            }
        }
    }
    
    /// Modify the `SemanticDefinition` in place (do not change anything inside the SemanticDefinitionEngine) to 
    /// accomodate `get_method_safe_for_unify` and others mechanisms that change TypeId inside decls
    fn use_tid_map(&mut self, tid_map: HashMap<TypeId, TypeId>) {
        self.tid_map = tid_map;
    }
}

#[derive(Clone, Debug)]
enum InnerTypeEvent {
    OnNonChangeableState {
        callback_id: usize,
    }
}

struct CallbackRegistry {
    callbacks: Vec<Box<dyn FnMut(&Engines, &Handler)>>,
}

#[derive(Clone)]
pub struct SemanticDefinitionInner {
    unify: Vec<(TypeId, TypeId)>,
    events: HashMap<TypeId, Vec<InnerTypeEvent>>,
    callbacks: Arc<Mutex<CallbackRegistry>>,
}

impl std::fmt::Debug for SemanticDefinitionInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticDefinitionInner")
            .field("unify", &self.unify)
            .field("tys", &self.events)
            .finish()
    }
}

#[derive(Debug)]
pub struct SemanticDefinition {
    inner: Mutex<SemanticDefinitionInner>,    
}

impl SemanticDefinition {
    fn solver<'a>(&'a self, engines: &'a Engines, handler: &'a Handler) -> SemanticDefinitionSolver<'a> {
        SemanticDefinitionSolver {
            engines,
            handler,
            def: self.clone(),
            replacements: HashMap::default(),
            tid_map: HashMap::new(),
        }
    }

    pub fn push_type_callback(&self, tid: TypeId, e: TypeEvent, callback: impl 'static + FnMut(&Engines, &Handler)) {
        let mut inner = self.inner.lock().unwrap();

        let callback_id = {
            let mut registry = inner.callbacks.lock().unwrap();
            registry.callbacks.push(Box::new(callback));
            registry.callbacks.len() - 1
        };

        let v = inner.events.entry(tid).or_default();
        match e {
            TypeEvent::OnNonChangeableState => {
                v.push(InnerTypeEvent::OnNonChangeableState { callback_id });
            },
        }
    }

    pub fn push_type_unify(&self, a: TypeId, b: TypeId) {
        let mut inner = self.inner.lock().unwrap();
        inner.unify.push((a, b));
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
    sd.push_type_callback(tid0, TypeEvent::OnNonChangeableState, move |engines, handler| {
        let final_type_info = engines.te().get(tid0);
        let max_value = match final_type_info.as_ref() {
            TypeInfo::UnsignedInteger(IntegerBits::Eight) => u8::MAX as u64,
            TypeInfo::UnsignedInteger(IntegerBits::Sixteen) => u16::MAX as u64,
            TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo) => u32::MAX as u64,
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour) => u64::MAX as u64,
            _ => {
                handler.emit_err(CompileError::Internal("Invalid final type", Span::dummy()));
                todo!();
            },
        };

        if value > max_value {
            handler.emit_err(CompileError::Internal("Do not fit", Span::dummy()));
        }
    });

    // this will be on type_check_variable_declaration for the LHS
    let tid1 = engines.te().insert(&engines, TypeInfo::Unknown, None);     
    sd.push_type_unify(tid1, tid0);

    dbg!(&sd);

    // Simulate get_method_safe_for_unify
    engines.te().start_capturing_duplicates();
    engines.te().duplicate(&engines, tid0);
    let tid1_new = engines.te().duplicate(&engines, tid1);
    let tid_map = engines.te().end_capturing_duplicates().unwrap();

    dbg!(&tid_map);

    // This will be done when monomorphizing
    let tid2 = engines.te().insert(&engines, TypeInfo::UnsignedInteger(IntegerBits::Eight), None);
    let mut solver = sd.solver(&engines, &handler);
    solver.use_tid_map(tid_map);
    solver.replace_type(tid1_new, tid2);
    let r = solver.solve();

    dbg!(handler.consume());
}