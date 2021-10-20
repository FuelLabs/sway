use proc_macro2::{Literal, TokenStream};
use quote::quote;

/// Functions used by the Abigen to expand functions defined in an ABI spec.

/// Expands a doc string into an attribute token stream.
pub fn expand_doc(s: &str) -> TokenStream {
    let doc = Literal::string(s);
    quote! {
        #[doc = #doc]
    }
}
