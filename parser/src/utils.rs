use crate::parse_tree::ImportType;
use crate::{semantics, Ident, TypeInfo, TypedFunctionDeclaration};
use either::Either;
use std::collections::HashMap;

use pest::Span;

use crate::{parse_tree::CallPath, TypedDeclaration};
pub(crate) fn join_spans<'sc>(s1: Span<'sc>, s2: Span<'sc>) -> Span<'sc> {
    let s1_positions = s1.split();
    let s2_positions = s2.split();
    s1_positions.0.span(&s2_positions.1)
}

pub(crate) fn find_in_namespace<'sc, 'manifest, 'compiler>(
    path: Vec<Ident<'sc>>,
    name: ImportType<'sc>,
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
) -> Option<Either<&'compiler TypedDeclaration<'sc>, Vec<(Ident<'sc>, TypedDeclaration<'sc>)>>> {
    // see if the call path's first prefix has any matches
    match path.len() {
        // if there is no prefix, then the local namespace is what we want
        0 => find_in_namespace_inner(name, namespace, methods_namespace),

        1 => todo!("some sort of err -- this kind of importing isn't supported yet"),
        // if there prefix, then we want to search that imported namespace
        2 => {
            let (head, next) = (path[0].clone(), path[1].clone());
            println!("1");
            let methods_namespace = imported_method_namespace
                .get(dbg!(head.primary_name))?
                .get(dbg!(&next))?;
            println!("2");
            let namespace = imported_namespace.get(head.primary_name)?.get(&next)?;
            println!("3");

            find_in_namespace_inner(name, namespace, methods_namespace)
        }
        _ => todo!("Err: rework module system"),
    }
}

fn find_in_namespace_inner<'sc, 'compiler>(
    name: ImportType<'sc>,
    namespace: &'compiler HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    methods_namespace: &'compiler HashMap<TypeInfo<'sc>, Vec<TypedFunctionDeclaration<'sc>>>,
) -> Option<Either<&'compiler TypedDeclaration<'sc>, Vec<(Ident<'sc>, TypedDeclaration<'sc>)>>> {
    match name {
        ImportType::Item(ref s) => namespace.get(s).map(Either::Left),
        ImportType::Star => Some(Either::Right(
            namespace
                .iter()
                .map(|(x, y)| (x.clone(), y.clone()))
                .collect(),
        )),
    }
    // TODO method lookup
}
