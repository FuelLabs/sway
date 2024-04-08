use crate::{engine_threading::*, type_system::priv_prelude::*};

pub trait SubstTypes: Sized + Clone {
    #[must_use]
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self>;

    #[must_use]
    fn subst(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        if !type_mapping.is_empty() {
            self.subst_inner(type_mapping, engines)
        } else {
            None
        }
    }

    fn subst_mut(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        if let Some(new) = self.subst(type_mapping, engines) {
            *self = new;
        }
    }
}

impl<T: SubstTypes> SubstTypes for Box<T> {
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        let value = (&**self).subst_inner(type_mapping, engines)?;
        Some(Box::new(value))
    }
}

impl<T: SubstTypes> SubstTypes for Option<T> {
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        self.as_ref().map(|x| x.subst_inner(type_mapping, engines))
    }
}

impl<T: SubstTypes> SubstTypes for Vec<T> {
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        let mut iter = self.iter();
        let mut i = 0;

        while let Some(item) = iter.next() {
            if let Some(changed) = item.subst(type_mapping, engines) {
                let mut new_vec = Vec::with_capacity(self.len());
                new_vec.extend(self.iter().take(i).cloned());
                new_vec.push(changed);

                while let Some(item) = iter.next() {
                    let new_item = match item.subst(type_mapping, engines) {
                        Some(new_item) => new_item,
                        None => item.clone(),
                    };
                    new_vec.push(new_item);
                }

                return Some(new_vec);
            }

            i += 1;
        }

        None
    }
}

impl<A: Clone, B: SubstTypes> SubstTypes for (A, B) {
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        let new = self.1.subst(type_mapping, engines)?;
        Some((self.0.clone(), new))
    }
}

/// Helps with the implementation of SubstTypes.
/// Each expression is goes through `SubstTypes` and if all return `None`, the generated function
/// also returns `None`, and everything can aborted.
/// If one of the expressions return `Some(...)`, everything else is cloned and returned.
///
/// Example:
///
/// ```rust,ignore
/// impl SubstTypes for TypeParameter {
///     fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
///         let (type_id, trait_constraints) =
///             subs! {self.type_id, self.trait_constraints}(type_mapping, engines)?;
///         Some(Self {
///             type_id,
///             trait_constraints,
///             ..self.clone()
///         })
///     }
///  }
/// ```
#[macro_export]
macro_rules! subs {
    (unwrap_or_else; $a:ident; $($items:expr,)*; ; $($idx_rest:tt)* ) => { ($($items,)*) };
    (unwrap_or_else; $a:ident; $($items:expr,)*; $field:expr, $($field_rest:expr,)*; $idx:tt $($idx_rest:tt)*) => {
        subs!{unwrap_or_else;
            $a;
            $($items,)* $a.$idx.unwrap_or_else(|| $field.clone()),;
            $($field_rest,)*;
            $($idx_rest)*
        }
    };
    ($($field:expr),*) => {
        |type_mapping: &TypeSubstMap, engines: &Engines| {
            let mut all_none = true;
            let a = ($(
                {
                    let r = $field.subst(type_mapping, engines);
                    all_none &= r.is_none();
                    r
                },
            )*);

            if all_none {
                None
            } else {
                Some(
                    subs!{unwrap_or_else; a; ; $($field,)*; 0 1 2 3 4 5 6 7 8 9 10 }
                )
            }
        }
    };
}
