use sway_types::Ident;

/// The name used within a module to refer to one of its submodules.
///
/// If an alias was given to the `mod`, this will be the alias. If not, this is the submodule's
/// library name.
pub type ModName = Ident;

pub trait HasModule<T>
where
    T: HasSubmodules<Self>,
    Self: Sized,
{
    /// Returns the module of this submodule.
    fn module(&self) -> &T;
}

pub trait HasSubmodules<E>
where
    E: HasModule<Self>,
    Self: Sized,
{
    /// Returns the submodules of this module.
    fn submodules(&self) -> &[(ModName, E)];

    /// An iterator yielding all submodules recursively, depth-first.
    fn submodules_recursive(&self) -> SubmodulesRecursive<'_, Self, E> {
        SubmodulesRecursive {
            _module_type: std::marker::PhantomData,
            submods: self.submodules().iter(),
            current: None,
        }
    }
}

type NamedSubmodule<E> = (ModName, E);
type SubmoduleItem<'module, T, E> = (
    &'module NamedSubmodule<E>,
    Box<SubmodulesRecursive<'module, T, E>>,
);

/// Iterator type for iterating over submodules.
///
/// Used rather than `impl Iterator` to enable recursive submodule iteration.
pub struct SubmodulesRecursive<'module, T, E> {
    _module_type: std::marker::PhantomData<T>,
    submods: std::slice::Iter<'module, NamedSubmodule<E>>,
    current: Option<SubmoduleItem<'module, T, E>>,
}

impl<'module, T, E> Iterator for SubmodulesRecursive<'module, T, E>
where
    T: HasSubmodules<E> + 'module,
    E: HasModule<T>,
{
    type Item = &'module (ModName, E);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.current = match self.current.take() {
                None => match self.submods.next() {
                    None => return None,
                    Some(submod) => {
                        Some((submod, Box::new(submod.1.module().submodules_recursive())))
                    }
                },
                Some((submod, mut submods)) => match submods.next() {
                    Some(next) => {
                        self.current = Some((submod, submods));
                        return Some(next);
                    }
                    None => return Some(submod),
                },
            }
        }
    }
}
