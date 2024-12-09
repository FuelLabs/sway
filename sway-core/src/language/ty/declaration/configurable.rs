use crate::{
    decl_engine::{DeclId, DeclMapping, DeclRef, ReplaceDecls},
    engine_threading::*,
    has_changes,
    language::{parsed::ConfigurableDeclaration, ty::*, CallPath, Visibility},
    semantic_analysis::TypeCheckContext,
    transform,
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyConfigurableDecl {
    pub call_path: CallPath,
    pub value: Option<TyExpression>,
    pub visibility: Visibility,
    pub attributes: transform::Attributes,
    pub return_type: TypeId,
    pub type_ascription: GenericArgument,
    pub span: Span,
    // Only encoding v1 has a decode_fn
    pub decode_fn: Option<DeclRef<DeclId<TyFunctionDecl>>>,
}

impl TyConfigurableDecl {
    // A configurable is indirect if its type encoded buffer size
    // cannot be known at compilation time
    pub fn is_indirect(&self, engines: &Engines) -> bool {
        let type_info = engines.te().get(self.type_ascription.type_id);
        matches!(
            type_info.abi_encode_size_hint(engines),
            AbiEncodeSizeHint::PotentiallyInfinite | AbiEncodeSizeHint::CustomImpl
        )
    }
}

impl TyDeclParsedType for TyConfigurableDecl {
    type ParsedType = ConfigurableDeclaration;
}

impl DebugWithEngines for TyConfigurableDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, _engines: &Engines) -> fmt::Result {
        write!(f, "{}", self.call_path)
    }
}

impl EqWithEngines for TyConfigurableDecl {}
impl PartialEqWithEngines for TyConfigurableDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.call_path == other.call_path
            && self.value.eq(&other.value, ctx)
            && self.visibility == other.visibility
            && self.type_ascription.eq(&other.type_ascription, ctx)
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), ctx)
    }
}

impl HashWithEngines for TyConfigurableDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let type_engine = engines.te();
        let TyConfigurableDecl {
            call_path,
            value,
            visibility,
            return_type,
            type_ascription,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
            decode_fn: _, // this is defined entirely by the type ascription
        } = self;
        call_path.hash(state);
        value.hash(state, engines);
        visibility.hash(state);
        type_engine.get(*return_type).hash(state, engines);
        type_ascription.hash(state, engines);
    }
}

impl Named for TyConfigurableDecl {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl Spanned for TyConfigurableDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl SubstTypes for TyConfigurableDecl {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.return_type.subst(ctx);
            self.type_ascription.subst(ctx);
            self.value.subst(ctx);
        }
    }
}

impl ReplaceDecls for TyConfigurableDecl {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        if let Some(expr) = &mut self.value {
            expr.replace_decls(decl_mapping, handler, ctx)
        } else {
            Ok(false)
        }
    }
}
