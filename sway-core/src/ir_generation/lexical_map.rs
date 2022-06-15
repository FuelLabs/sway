// Nested mappings between symbol strings.  Allows shadowing and/or nested scopes for local
// symbols.
//
// NOTE: ALL symbols should be represented in this data structure to be sure that we
// don't accidentally ignore (i.e., neglect to shadow with) a new binding.
//
// A further complication is although we have enter_scope() and leave_scope() to potentially add
// and remove shadowing symbols, the re-use of symbol names can't be allowed, so all names are
// reserved even when they're not 'currently' valid.

use std::collections::HashMap;

pub(super) struct LexicalMap {
    symbol_map: Vec<HashMap<String, String>>,
    reserved_sybols: Vec<String>,
}

impl LexicalMap {
    pub(super) fn from_iter<I: IntoIterator<Item = String>>(names: I) -> Self {
        let (root_symbol_map, reserved_sybols): (HashMap<String, String>, Vec<String>) = names
            .into_iter()
            .fold((HashMap::new(), Vec::new()), |(mut m, mut r), name| {
                m.insert(name.clone(), name.clone());
                r.push(name);
                (m, r)
            });

        LexicalMap {
            symbol_map: vec![root_symbol_map],
            reserved_sybols,
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

    pub(super) fn insert(&mut self, new_symbol: String) -> String {
        // Insert this new symbol into this lexical scope.  If it has ever existed then the
        // original will be shadowed and the shadower is returned.
        fn get_new_local_symbol(reserved: &[String], candidate: String) -> String {
            match reserved.iter().find(|&reserved| reserved == &candidate) {
                None => candidate,
                Some(_) => {
                    // Try again with adjusted candidate.
                    get_new_local_symbol(reserved, format!("{candidate}_"))
                }
            }
        }
        let local_symbol = get_new_local_symbol(&self.reserved_sybols, new_symbol.clone());
        self.symbol_map
            .last_mut()
            .expect("LexicalMap should always have at least the root scope.")
            .insert(new_symbol, local_symbol.clone());
        self.reserved_sybols.push(local_symbol.clone());
        local_symbol
    }
}
