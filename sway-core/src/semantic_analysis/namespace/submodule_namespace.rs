use super::{namespace::Namespace, PathBuf};

/// A namespace session type representing the type-checking of a submodule.
///
/// This type allows for re-using the parent's `Namespace` in order to provide access to the
/// `root` and `init` throughout type-checking of the submodule, but with an updated `mod_path` to
/// represent the submodule's path. When dropped, the `SubmoduleNamespace` reset's the
/// `Namespace`'s `mod_path` to the parent module path so that type-checking of the parent may
/// continue.
pub struct SubmoduleNamespace<'a> {
    pub(crate) namespace: &'a mut Namespace,
    pub(crate) parent_mod_path: PathBuf,
}

impl<'a> std::ops::Deref for SubmoduleNamespace<'a> {
    type Target = Namespace;
    fn deref(&self) -> &Self::Target {
        self.namespace
    }
}

impl<'a> std::ops::DerefMut for SubmoduleNamespace<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.namespace
    }
}

impl<'a> Drop for SubmoduleNamespace<'a> {
    fn drop(&mut self) {
        // Replace the submodule path with the original module path.
        // This ensures that the namespace's module path is reset when ownership over it is
        // relinquished from the SubmoduleNamespace.
        self.namespace.mod_path = std::mem::take(&mut self.parent_mod_path);
    }
}
