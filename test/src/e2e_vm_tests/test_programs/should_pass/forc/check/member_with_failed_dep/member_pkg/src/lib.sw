library;

// This package itself is valid. Its type check runs after the type check of
// `dep_pkg` has failed, leaving abandoned open decl slots in the shared decl
// engine. Those slots must not poison the type check of this package - in
// particular, they must not trigger the internal compiler error that guards
// against decl slots left open after a successfully finished type checking.

pub fn call_make() -> u64 {
    dep_pkg::make()
}
