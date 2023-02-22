//! Example: https://aloso.github.io/2021/03/09/creating-an-iterator

use std::slice::Iter;

use crate::type_system::*;

pub struct TypeParametersIter<'a> {
    self_type: &'a Option<TypeParameter>,
    have_shown_self_type: bool,
    iter: Iter<'a, TypeParameter>,
}

impl<'a> TypeParametersIter<'a> {
    pub(super) fn new(
        self_type: &'a Option<TypeParameter>,
        have_shown_self_type: bool,
        iter: Iter<'a, TypeParameter>,
    ) -> TypeParametersIter<'a> {
        TypeParametersIter {
            self_type,
            have_shown_self_type,
            iter,
        }
    }
}

impl<'a> Iterator for TypeParametersIter<'a> {
    type Item = &'a TypeParameter;

    fn next(&mut self) -> Option<Self::Item> {
        match self.self_type.as_ref() {
            Some(self_type) if !self.have_shown_self_type => {
                self.have_shown_self_type = true;
                Some(self_type)
            }
            Some(_) => self.iter.next(),
            None => self.iter.next(),
        }
    }
}
