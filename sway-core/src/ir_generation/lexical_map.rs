// Nested mappings between symbol strings.  Allows shadowing and/or nested scopes for local
// symbols.
//
// NOTE: ALL symbols should be represented in this data structure to be sure that we
// don't accidentally ignore (i.e., neglect to shadow with) a new binding.
//
// A further complication is although we have enter_scope() and leave_scope() to potentially add
// and remove shadowing symbols, the re-use of symbol names can't be allowed, so all names are
// reserved even when they're not 'currently' valid.

use std::collections::{HashMap, HashSet};

pub(super) struct LexicalMap {
    symbol_map: Vec<HashMap<String, String>>,
    reserved_symbols: HashSet<String>,
}

impl LexicalMap {
    pub(super) fn from_iter<I: IntoIterator<Item = String>>(names: I) -> Self {
        let (root_symbol_map, reserved_symbols): (HashMap<String, String>, HashSet<String>) = names
            .into_iter()
            .fold((HashMap::new(), HashSet::new()), |(mut m, mut r), name| {
                m.insert(name.clone(), name.clone());
                r.insert(name);
                (m, r)
            });

        LexicalMap {
            symbol_map: vec![root_symbol_map],
            reserved_symbols,
        }
    }

    pub(super) fn enter_scope(&mut self) -> &mut Self {
        self.symbol_map.push(HashMap::new());
        self
    }

    pub(super) fn leave_scope(&mut self) -> &mut Self {
        assert!(self.symbol_map.len() > 1);
        self.symbol_map.pop();
        self
    }

    pub(super) fn get(&self, symbol: &str) -> Option<&String> {
        // Only get 'valid' symbols which are currently in scope.
        self.symbol_map
            .iter()
            .rev()
            .find_map(|scope| scope.get(symbol))
    }

    /// Insert `new_symbol` into this lexical scope. If it has ever existed then the
    /// original will be shadowed and the shadower is returned.
    pub(super) fn insert(&mut self, new_symbol: String) -> String {
        fn get_new_local_symbol(reserved: &HashSet<String>, candidate: String) -> String {
            if reserved.contains(&candidate) {
                // Try again with adjusted candidate.
                get_new_local_symbol(reserved, format!("{candidate}_"))
            } else {
                candidate
            }
        }
        let local_symbol = get_new_local_symbol(&self.reserved_symbols, new_symbol.clone());
        self.symbol_map
            .last_mut()
            .expect("LexicalMap should always have at least the root scope.")
            .insert(new_symbol, local_symbol.clone());
        self.reserved_symbols.insert(local_symbol.clone());
        local_symbol
    }

    /// Generate and reserve a unique 'anonymous' symbol. It is in the form `__anon_X` where `X` is a
    /// unique number.
    pub(super) fn insert_anon(&mut self) -> String {
        self.insert_unique_named("anon")
    }

    /// Generate and reserve a unique named symbol. It is in the form `__<name>_X` where `X` is a
    /// unique number.
    pub(super) fn insert_unique_named(&mut self, name: &str) -> String {
        let anon_symbol = (0..)
            .map(|n| format!("__{name}_{n}"))
            .find(|candidate| !self.reserved_symbols.contains(candidate))
            .unwrap();
        self.symbol_map
            .last_mut()
            .expect("LexicalMap should always have at least the root scope.")
            .insert(anon_symbol.clone(), anon_symbol.clone());
        self.reserved_symbols.insert(anon_symbol.clone());
        anon_symbol
    }
}
