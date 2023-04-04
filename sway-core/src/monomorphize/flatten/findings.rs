use crate::{decl_engine::DeclId, language::ty::*, type_system::*};

pub(crate) struct Findings<'a> {
    from_fn: Vec<(DeclId<TyFunctionDecl>, &'a SubstList)>,
    from_struct: Vec<(DeclId<TyStructDecl>, &'a SubstList)>,
    from_enum: Vec<(DeclId<TyEnumDecl>, &'a SubstList)>,
    from_trait: Vec<(DeclId<TyTraitDecl>, &'a SubstList)>,
    from_impl_trait: Vec<(DeclId<TyImplTrait>, &'a SubstList)>,
}

impl<'a> Findings<'a> {
    pub(super) fn new() -> Findings<'a> {
        Findings {
            from_fn: vec![],
            from_struct: vec![],
            from_enum: vec![],
            from_trait: vec![],
            from_impl_trait: vec![],
        }
    }

    pub(super) fn from_fn_decl(
        decl_id: &'a DeclId<TyFunctionDecl>,
        subst_list: &'a SubstList,
    ) -> Findings<'a> {
        Findings {
            from_fn: vec![(*decl_id, subst_list)],
            from_struct: vec![],
            from_enum: vec![],
            from_trait: vec![],
            from_impl_trait: vec![],
        }
    }

    pub(super) fn from_struct_decl(
        decl_id: &'a DeclId<TyStructDecl>,
        subst_list: &'a SubstList,
    ) -> Findings<'a> {
        Findings {
            from_fn: vec![],
            from_struct: vec![(*decl_id, subst_list)],
            from_enum: vec![],
            from_trait: vec![],
            from_impl_trait: vec![],
        }
    }

    pub(super) fn from_enum_decl(
        decl_id: &'a DeclId<TyEnumDecl>,
        subst_list: &'a SubstList,
    ) -> Findings<'a> {
        Findings {
            from_fn: vec![],
            from_struct: vec![],
            from_enum: vec![(*decl_id, subst_list)],
            from_trait: vec![],
            from_impl_trait: vec![],
        }
    }

    pub(super) fn from_trait_decl(
        decl_id: &'a DeclId<TyTraitDecl>,
        subst_list: &'a SubstList,
    ) -> Findings<'a> {
        Findings {
            from_fn: vec![],
            from_struct: vec![],
            from_enum: vec![],
            from_trait: vec![(*decl_id, subst_list)],
            from_impl_trait: vec![],
        }
    }

    pub(super) fn from_impl_trait(
        decl_id: &'a DeclId<TyImplTrait>,
        subst_list: &'a SubstList,
    ) -> Findings<'a> {
        Findings {
            from_fn: vec![],
            from_struct: vec![],
            from_enum: vec![],
            from_trait: vec![],
            from_impl_trait: vec![(*decl_id, subst_list)],
        }
    }

    pub(super) fn add(self, other: Findings<'a>) -> Findings<'a> {
        let Findings {
            from_fn: mut lf,
            from_struct: mut ls,
            from_enum: mut le,
            from_trait: mut lt,
            from_impl_trait: mut lit,
        } = self;
        let Findings {
            from_fn: rf,
            from_struct: rs,
            from_enum: re,
            from_trait: rt,
            from_impl_trait: rit,
        } = other;
        lf.extend(rf);
        ls.extend(rs);
        le.extend(re);
        lt.extend(rt);
        lit.extend(rit);
        Findings {
            from_fn: lf,
            from_struct: ls,
            from_enum: le,
            from_trait: lt,
            from_impl_trait: lit,
        }
    }
}

impl<'a> FromIterator<Findings<'a>> for Findings<'a> {
    fn from_iter<T: IntoIterator<Item = Findings<'a>>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Findings::new(), |acc, elem| acc.add(elem))
    }
}
