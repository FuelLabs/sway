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
                tys: HashMap::new(),
                callbacks: vec![],
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
    types: HashMap<TypeId, TypeId>,
}

enum UnificationChange{
    LeftChanged,
    RightChanged,
}

fn unify(engines: &Engines, tida: &TypeId, tidb: &TypeId) -> Option<UnificationChange> {
    let a = engines.te().get(*tida);
    let b = engines.te().get(*tidb);
    match (a.as_ref(), b.as_ref()) {
        // (TypeInfo::Unknown, TypeInfo::Unknown) => None,
        (TypeInfo::UnsignedInteger(IntegerBits::Eight), TypeInfo::UnsignedInteger(IntegerBits::Eight)) => {
            None
        }
        (TypeInfo::UnsignedInteger(IntegerBits::Eight), TypeInfo::Numeric) => {
            engines.te().replace(engines, *tidb, TypeInfo::clone(&a));
            Some(UnificationChange::RightChanged)
        },
        // (a, TypeInfo::Unknown) => {
        //     engines.te().replace(engines, *tidb, a.clone());
        //     Some(UnificationChange::RightChanged)
        // },
        _ => todo!("{:?} {:?}", engines.help_out(a), engines.help_out(b)),
    }
}

impl<'a> SemanticDefinitionSolver<'a> {
    pub fn solve_type(&mut self, tid: TypeId, new_tid: TypeId) -> &mut Self {
        self.types.insert(tid, new_tid);
        self
    }

    pub fn solve(self) {
        let SemanticDefinitionSolver {
            engines,
            handler,
            def,
            types,
        } = self;

        let mut inner = def.inner.lock().unwrap();

        let mut steps = 10;
        let mut q = types.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
        while let Some(tid) = q.pop() {
            steps -= 1;
            if steps <= 0 {
                break;
            }
            eprintln!("{tid:?} {:?}; q: {q:?}", engines.help_out(tid));
            let type_info_is_concrete = tid.is_concrete(engines, TreatNumericAs::Abstract);

            if let Some(replace_tid) = types.get(&tid) {
                eprintln!("    replace: {tid:?}({:?}) with {replace_tid:?}({:?})", engines.help_out(tid), engines.help_out(replace_tid));
                
                if !tid.eq(replace_tid) {
                    let replace_type_info = engines.te().get(*replace_tid);
                    engines.te().replace(engines, tid, replace_type_info.as_ref().clone());
                }                
            }

            for (a, b) in inner.unify.iter().filter(|(a, b)| *a == tid || *b == tid) {
                eprintln!("    unify: {a:?}({:?}) with {b:?}({:?})", engines.help_out(a), engines.help_out(b));
                match unify(engines, a, b) {
                    Some(UnificationChange::LeftChanged) => {
                        eprintln!("        left changed");
                        q.push(*a);
                    },
                    Some(UnificationChange::RightChanged) => {
                        eprintln!("        right changed");
                        q.push(*b);
                    },
                    None => {},
                }
            }

            if let Some(actions) = inner.tys.get(&tid).cloned() {
                for action in actions.iter() {
                    match action {
                        InnerTypeEvent::OnNonChangeableState { callback_id } => {
                            let type_info_is_concrete_after = tid.is_concrete(engines, TreatNumericAs::Abstract);
                            if !type_info_is_concrete && type_info_is_concrete_after {
                                eprintln!("    calling OnNonChangeableState callback");
                                let cb = &mut inner.callbacks[*callback_id];
                                cb(engines, handler);
                            }
                        },
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum InnerTypeEvent {
    OnNonChangeableState {
        callback_id: usize,
    }
}

pub struct SemanticDefinitionInner {
    unify: Vec<(TypeId, TypeId)>,
    tys: HashMap<TypeId, Vec<InnerTypeEvent>>,
    callbacks: Vec<Box<dyn FnMut(&Engines, &Handler)>>,
}

impl std::fmt::Debug for SemanticDefinitionInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticDefinitionInner")
            .field("unify", &self.unify)
            .field("tys", &self.tys)
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
            def: self,
            types: HashMap::default(),
        }
    }

    pub fn type_callback(&self, tid: TypeId, e: TypeEvent, callback: impl 'static + FnMut(&Engines, &Handler)) {
        let mut inner = self.inner.lock().unwrap();

        let callback_id = inner.callbacks.len();
        inner.callbacks.push(Box::new(callback));

        let v = inner.tys.entry(tid).or_default();
        match e {
            TypeEvent::OnNonChangeableState => {
                v.push(InnerTypeEvent::OnNonChangeableState { callback_id });
            },
        }
    }

    pub fn type_unify(&self, a: TypeId, b: TypeId) {
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
    
    let sd = sde.get(sdid);

    // let a = 0x100;
    // this will be on type_check_literal for the RHS
    let value = 0x100 as u64;
    let tid0 = engines.te().insert(&engines, TypeInfo::Numeric, None);     
    sd.type_callback(tid0, TypeEvent::OnNonChangeableState, move |engines, handler| {
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
    sd.type_unify(tid1, tid0);

    dbg!(&sd);

    // This will be done when monomorphizing
    let tid2 = engines.te().insert(&engines, TypeInfo::UnsignedInteger(IntegerBits::Eight), None);
    let mut solver = sd.solver(&engines, &handler);
    solver.solve_type(tid1, tid2);
    solver.solve();

    dbg!(handler.consume());
}