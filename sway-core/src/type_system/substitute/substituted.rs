use std::{collections::HashMap, hash::Hash};

/// A wrapper type to indicate at the type-level an object that has undergone
/// type substitution.
#[derive(Debug)]
pub struct Substituted<T> {
    /// The element that has undergone substitution.
    inner: T,

    /// An inner marker that denotes if this type underwent transformational
    /// substitution.
    ///
    /// This would be: `self -> self'`. As opposed to `self -> self`.
    marker: bool,
}

impl<T> Substituted<T> {
    pub(super) fn new(inner: T, marker: bool) -> Substituted<T> {
        Substituted { inner, marker }
    }

    pub(crate) fn inner(&self) -> &T {
        &self.inner
    }

    pub(crate) fn into_inner(self) -> T {
        self.inner
    }

    pub(super) fn marker(&self) -> bool {
        self.marker
    }

    pub(crate) fn bypass(_inner: T) -> Substituted<T> {
        todo!()
    }

    pub(crate) fn as_ref<'a>(&'a self) -> Substituted<&'a T> {
        Substituted::new(self.inner(), self.marker())
    }

    pub(crate) fn map<F, U>(self, f: F) -> Substituted<U>
    where
        F: FnOnce(T) -> U,
    {
        let Substituted { inner, marker } = self;
        Substituted::new(f(inner), marker)
    }

    pub(crate) fn and_then<F, U>(self, f: F) -> Substituted<U>
    where
        F: FnOnce(T) -> Substituted<U>,
    {
        let Substituted { inner, marker } = self;
        let next = f(inner);
        let marker = marker || next.marker();
        Substituted::new(next.into_inner(), marker)
    }
}

impl<A> Substituted<A> {
    pub(super) fn and<B>(self, next: Substituted<B>) -> Substituted<(A, B)> {
        let marker = self.marker() || next.marker();
        let then = (self.into_inner(), next.into_inner());
        Substituted::new(then, marker)
    }
}

impl<A, B> Substituted<(A, B)> {
    pub(super) fn and_two<C>(self, next: Substituted<C>) -> Substituted<(A, B, C)> {
        let marker = self.marker() || next.marker();
        let (a, b) = self.into_inner();
        let then = (a, b, next.into_inner());
        Substituted::new(then, marker)
    }
}

impl<A, B, C> Substituted<(A, B, C)> {
    pub(super) fn and_three<D>(self, next: Substituted<D>) -> Substituted<(A, B, C, D)> {
        let marker = self.marker() || next.marker();
        let (a, b, c) = self.into_inner();
        let then = (a, b, c, next.into_inner());
        Substituted::new(then, marker)
    }
}

impl<A, B, C, D> Substituted<(A, B, C, D)> {
    pub(super) fn and_four<E>(self, next: Substituted<E>) -> Substituted<(A, B, C, D, E)> {
        let marker = self.marker() || next.marker();
        let (a, b, c, d) = self.into_inner();
        let then = (a, b, c, d, next.into_inner());
        Substituted::new(then, marker)
    }
}

impl<A, B, C, D, E> Substituted<(A, B, C, D, E)> {
    pub(super) fn and_five<F>(self, next: Substituted<F>) -> Substituted<(A, B, C, D, E, F)> {
        let marker = self.marker() || next.marker();
        let (a, b, c, d, e) = self.into_inner();
        let then = (a, b, c, d, e, next.into_inner());
        Substituted::new(then, marker)
    }
}

impl<T> Substituted<Substituted<T>> {
    pub(super) fn flatten(self) -> Substituted<T> {
        let marker = self.marker();
        let inner = self.into_inner();
        let marker = marker || inner.marker();
        Substituted::new(inner.into_inner(), marker)
    }
}

impl<T> Copy for Substituted<T> where T: Clone + Copy {}
impl<T> Clone for Substituted<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: self.marker,
        }
    }
}

// https://stackoverflow.com/a/30220832
pub struct SubstitutedIterator<'a, T> {
    inner: &'a [T],
    index: usize,
    marker: bool,
}

impl<'a, T> Iterator for SubstitutedIterator<'a, T> {
    type Item = Substituted<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        let elem = self.inner.get(self.index)?;
        self.index += 1;
        Some(Substituted::new(elem, self.marker))
    }
}

// https://doc.rust-lang.org/stable/std/iter/#iterating-by-reference
impl<'a, T> IntoIterator for Substituted<&'a Vec<T>> {
    type Item = Substituted<&'a T>;
    type IntoIter = SubstitutedIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SubstitutedIterator {
            inner: self.inner(),
            index: 0,
            marker: self.marker,
        }
    }
}

impl<T> FromIterator<Substituted<T>> for Substituted<Vec<T>> {
    fn from_iter<I: IntoIterator<Item = Substituted<T>>>(iter: I) -> Self {
        let mut marker = false;
        let inner = iter
            .into_iter()
            .map(|elem| {
                marker = marker || elem.marker();
                elem.into_inner()
            })
            .collect();
        Substituted::new(inner, marker)
    }
}

impl<K, V> FromIterator<Substituted<(K, V)>> for Substituted<HashMap<K, V>>
where
    K: Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item = Substituted<(K, V)>>>(iter: I) -> Self {
        let mut marker = false;
        let inner = iter
            .into_iter()
            .map(|elem| {
                marker = marker || elem.marker();
                elem.into_inner()
            })
            .collect();
        Substituted::new(inner, marker)
    }
}

impl<'a, T> Substituted<Vec<T>> {
    pub(crate) fn iter(&'a self) -> impl Iterator<Item = Substituted<&'a T>> {
        self.as_ref().into_iter()
    }
}

impl<'a, T> Substituted<&'a Vec<T>> {
    pub(crate) fn iter(self) -> impl Iterator<Item = Substituted<&'a T>> {
        self.into_iter()
    }
}

pub(crate) trait SubstitutedAndMap<F, A, O> // where
//     F: FnOnce(A) -> O,
{
    fn and_map(self, f: F) -> Substituted<O>;
}

impl<F, A, B, O> SubstitutedAndMap<F, (A, B), O> for Substituted<(A, B)>
where
    F: FnOnce(A, B) -> O,
{
    fn and_map(self, f: F) -> Substituted<O> {
        let Substituted { inner, marker } = self;
        let (a, b): (A, B) = inner;
        let o: O = f(a, b);
        Substituted::new(o, marker)
    }
}

impl<F, A, B, C, O> SubstitutedAndMap<F, (A, B, C), O> for Substituted<(A, B, C)>
where
    F: FnOnce(A, B, C) -> O,
{
    fn and_map(self, f: F) -> Substituted<O> {
        let Substituted { inner, marker } = self;
        let (a, b, c): (A, B, C) = inner;
        let o: O = f(a, b, c);
        Substituted::new(o, marker)
    }
}

impl<F, A, B, C, D, O> SubstitutedAndMap<F, (A, B, C, D), O> for Substituted<(A, B, C, D)>
where
    F: FnOnce(A, B, C, D) -> O,
{
    fn and_map(self, f: F) -> Substituted<O> {
        let Substituted { inner, marker } = self;
        let (a, b, c, d): (A, B, C, D) = inner;
        let o: O = f(a, b, c, d);
        Substituted::new(o, marker)
    }
}

impl<F, A, B, C, D, E, G, O> SubstitutedAndMap<F, (A, B, C, D, E, G), O>
    for Substituted<(A, B, C, D, E, G)>
where
    F: FnOnce(A, B, C, D, E, G) -> O,
{
    fn and_map(self, f: F) -> Substituted<O> {
        let Substituted { inner, marker } = self;
        let (a, b, c, d, e, g): (A, B, C, D, E, G) = inner;
        let o: O = f(a, b, c, d, e, g);
        Substituted::new(o, marker)
    }
}
