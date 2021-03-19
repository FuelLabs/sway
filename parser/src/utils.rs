use crate::{semantics, Ident, TypeInfo, TypedFunctionDeclaration};
use std::collections::HashMap;

use pest::Span;

use crate::{parse_tree::CallPath, TypedDeclaration};
pub(crate) fn join_spans<'sc>(s1: Span<'sc>, s2: Span<'sc>) -> Span<'sc> {
    let s1_positions = s1.split();
    let s2_positions = s2.split();
    s1_positions.0.span(&s2_positions.1)
}

pub(crate) fn find_in_namespace<'sc, 'manifest, 'compiler>(
    name: CallPath<'sc>,
    namespace: &'compiler HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    methods_namespace: &'compiler HashMap<TypeInfo<'sc>, Vec<TypedFunctionDeclaration<'sc>>>,
    imported_namespace: &'compiler HashMap<
        &'manifest str,
        HashMap<Ident<'sc>, HashMap<Ident<'sc>, semantics::TypedDeclaration<'sc>>>,
    >,
    imported_method_namespace: &'compiler HashMap<
        &'manifest str,
        HashMap<Ident<'sc>, HashMap<TypeInfo<'sc>, Vec<semantics::TypedFunctionDeclaration<'sc>>>>,
    >,
) -> Option<&'compiler TypedDeclaration<'sc>> {
    dbg!(&name);
    dbg!(&imported_namespace.get(&(name.prefixes[0].primary_name.clone())));
    dbg!(&imported_namespace);
    // see if the call path's first prefix has any matches
    match name.prefixes.len() {
        // if there is no prefix, then the local namespace is what we want
        0 => find_in_namespace_inner(name, namespace, methods_namespace),

        1 => todo!("some sort of err -- this kind of importing isn't supported yet"),
        // if there prefix, then we want to search that imported namespace
        2 => {
            let (head, next) = (name.prefixes[0].clone(), name.prefixes[1].clone());
            let methods_namespace = imported_method_namespace
                .get(head.primary_name)?
                .get(&next)?;
            let namespace = imported_namespace.get(head.primary_name)?.get(&next)?;

            find_in_namespace_inner(
                CallPath {
                    prefixes: vec![],
                    suffix: name.suffix.clone(),
                },
                namespace,
                methods_namespace,
            )
        }
        _ => todo!("Err: rework module system"),
    }
}

fn find_in_namespace_inner<'sc, 'compiler>(
    name: CallPath<'sc>,
    namespace: &'compiler HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    methods_namespace: &'compiler HashMap<TypeInfo<'sc>, Vec<TypedFunctionDeclaration<'sc>>>,
) -> Option<&'compiler TypedDeclaration<'sc>> {
    println!("yo ");
    dbg!(&name.suffix);
    dbg!(namespace.get(&name.suffix))
    // TODO method lookup
}
