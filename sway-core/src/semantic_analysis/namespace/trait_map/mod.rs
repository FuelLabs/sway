mod key;
pub(crate) mod map;
mod suffix;

use key::*;
use suffix::*;

use crate::language::{ty::TyImplItem, CallPath};

/// Trait name, containing type arguments for generic traits.
type TraitCallPath = CallPath<TraitSuffix>;

/// Ordered map of name to [TyImplItem](ty::TyImplItem)
type TraitItems = im::OrdMap<String, TyImplItem>;

/// Map of trait name and type to [TraitItems].
type TraitImpls = im::OrdMap<TraitKey, TraitItems>;

/*

TraitMap: smart wrapper
- TraitImpls

TraitImpls: map of key to items
- im::OrdMap<TraitKey, TraitItems>

TraitKey: smart wrapper for the trait impl key
- call_path: TraitCallPath
- implementing_for: TypeId

TraitCallPath: trait call path with type arguments
- CallPath<TraitSuffix>

TraitSuffix: trait name and type arguments
- name: Ident
- args: Vec<TypeArgument>

*/
