use std::fmt::{Debug, Formatter, Result};
use std::sync::Mutex;

use sway_ir::Type;

use crate::decl_engine::DeclRefFunction;
use crate::language::parsed::MethodName;
use crate::semantic_analysis::TypeCheckContext;
use crate::{Engines, TypeBinding, TypeId, TypeInfo};

pub trait Observer {
    fn on_trace(&self, _msg: &str) {}

    fn on_before_method_resolution(
        &self,
        _ctx: &TypeCheckContext<'_>,
        _method_name: &TypeBinding<MethodName>,
        _args_types: &[TypeId],
    ) {
    }

    fn on_after_method_resolution(
        &self,
        _ctx: &TypeCheckContext<'_>,
        _method_name: &TypeBinding<MethodName>,
        _args_types: &[TypeId],
        _new_ref: DeclRefFunction,
        _new_type_id: TypeId,
    ) {
    }

    fn on_after_ir_type_resolution(
        &self,
        _engines: &Engines,
        _ctx: &sway_ir::Context,
        _type_info: &TypeInfo,
        _ir_type: &Type,
    ) {
    }
}

#[derive(Default)]
pub struct ObservabilityEngine {
    observer: Mutex<Option<Box<dyn Observer>>>,
    trace: Mutex<bool>,
}

unsafe impl Send for ObservabilityEngine {}
unsafe impl Sync for ObservabilityEngine {}

impl Debug for ObservabilityEngine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("ObservabilityEngine")
            .field("trace", &self.trace)
            .finish()
    }
}

impl ObservabilityEngine {
    pub fn set_observer(&self, observer: Box<dyn Observer>) {
        let mut obs = self.observer.lock().unwrap();
        *obs = Some(observer);
    }

    pub fn raise_on_before_method_resolution(
        &self,
        ctx: &TypeCheckContext,
        method_name: &TypeBinding<MethodName>,
        arguments_types: &[TypeId],
    ) {
        if let Some(obs) = self.observer.lock().unwrap().as_mut() {
            obs.on_before_method_resolution(ctx, method_name, arguments_types);
        }
    }

    pub fn raise_on_after_method_resolution(
        &self,
        ctx: &TypeCheckContext,
        method_name: &TypeBinding<MethodName>,
        arguments_types: &[TypeId],
        ref_function: DeclRefFunction,
        tid: TypeId,
    ) {
        if let Some(obs) = self.observer.lock().unwrap().as_mut() {
            obs.on_after_method_resolution(ctx, method_name, arguments_types, ref_function, tid);
        }
    }

    pub fn raise_on_after_ir_type_resolution(
        &self,
        engines: &Engines,
        ctx: &sway_ir::Context,
        type_info: &TypeInfo,
        ir_type: &Type,
    ) {
        if let Some(obs) = self.observer.lock().unwrap().as_mut() {
            obs.on_after_ir_type_resolution(engines, ctx, type_info, ir_type);
        }
    }

    pub(crate) fn trace(&self, get_txt: impl FnOnce() -> String) {
        let trace = self.trace.lock().unwrap();
        if *trace {
            if let Some(obs) = self.observer.lock().unwrap().as_mut() {
                obs.on_trace(&get_txt());
            }
        }
    }

    pub fn enable_trace(&self, enable: bool) {
        let mut trace = self.trace.lock().unwrap();
        *trace = enable
    }
}
