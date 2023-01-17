use std::collections::hash_map::HashMap;
use sway_types::Ident;

extern crate quote;
extern crate syn;

use proc_macro2::{Literal, Punct, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::fmt::Write;
use syn::{parse_quote, parse_str, ItemFn, ItemMod};

#[test]
fn test2() {
    let true_keyword: ItemMod = parse_quote! {
        /// A value of type [`bool`] representing logical **true**.
        ///
        /// Logically `true` is not equal to [`false`].
        ///
        /// ## Control structures that check for **true**
        ///
        /// Several of Sway's control structures will check for a `bool` condition evaluating to **true**.
        ///
        ///   * The condition in an [`if`] expression must be of type `bool`.
        ///     Whenever that condition evaluates to **true**, the `if` expression takes
        ///     on the value of the first block. If however, the condition evaluates
        ///     to `false`, the expression takes on value of the `else` block if there is one.
        ///
        ///   * [`while`] is another control flow construct expecting a `bool`-typed condition.
        ///     As long as the condition evaluates to **true**, the `while` loop will continually
        ///     evaluate its associated block.
        ///
        ///   * [`match`] arms can have guard clauses on them.
        mod true_keyword {}
    };

    let false_keyword: ItemMod = parse_quote! {
        /// A value of type [`bool`] representing logical **false**.
        ///
        /// `false` is the logical opposite of [`true`].
        ///
        /// See the documentation for [`true`] for more information.
        mod false_keyword {}
    };

    let mut keyword_docs = HashMap::new();
    let keywords = vec![true_keyword, false_keyword];
    keywords.iter().for_each(|keyword| {
        let ident = keyword.ident.clone().to_string();
        // remove "_keyword" suffix to get the keyword name
        let name = ident.trim_end_matches("_keyword").to_owned();
        let mut documentation = String::new();
        keyword.attrs.iter().for_each(|attr| {
            let tokens = attr.tokens.to_token_stream();
            let lit = extract_lit(tokens);
            write!(documentation, "{}\n", lit).unwrap();
        });
        keyword_docs.insert(name, documentation);
    });

    keyword_docs.iter().for_each(|(k, v)| {
        println!("{}: {}", k, v);
    });
}

/// Extracts the literal from a token stream and returns it as a string.
fn extract_lit(tokens: TokenStream) -> String {
    let mut res = "".to_string();
    for token in tokens.into_iter() {
        if let TokenTree::Literal(l) = token {
            let mut s = l.to_string();
            s = s.replace("r\"", "///"); // replace the "r\"" with /// at the beginning
            s.pop(); // remove the " at the end
            res.push_str(&s);
        }
    }
    res
}
