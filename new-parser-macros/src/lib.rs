use {
    proc_macro::TokenStream,
    proc_macro2::TokenStream as TokenStream2,
    syn::{
        Token, parse_macro_input, Expr, Ident,
        punctuated::Punctuated,
    },
    quote::{quote, format_ident},
};

#[proc_macro]
pub fn or(tokens: TokenStream) -> TokenStream {
    let exprs: Punctuated<Expr, Token![,]> = parse_macro_input!(tokens with Punctuated::parse_terminated);
    let exprs = exprs.into_iter().rev().collect();
    let expr = build_or_matches(exprs, Vec::new());
    let ret = quote! {
        from_fn(move |input| { #expr })
    };
    ret.into()
}

fn build_or_matches(mut exprs: Vec<Expr>, mut errors: Vec<Ident>) -> TokenStream2 {
    match exprs.pop() {
        Some(expr) => {
            let error_name = format_ident!("error{}", errors.len());
            errors.push(error_name.clone());
            let on_error = build_or_matches(exprs, errors);
            quote! {
                match #expr.parse(input) {
                    Ok(stuff) => Ok(stuff),
                    Err(Err(error)) => Err(Err(error)),
                    Err(Ok(#error_name)) => #on_error,
                }
            }
        },
        None => {
            quote! {
                Err(Ok((#(#errors,)*)))
            }
        },
    }
}

#[proc_macro]
pub fn then(tokens: TokenStream) -> TokenStream {
    let exprs: Punctuated<Expr, Token![,]> = parse_macro_input!(tokens with Punctuated::parse_terminated);
    let exprs = exprs.into_iter().rev().collect();
    let expr = build_then_matches(exprs, Vec::new());
    let ret = quote! {
        from_fn(move |input| {
            let mut total_len = 0;
            #expr
        })
    };
    ret.into()
}

fn build_then_matches(mut exprs: Vec<Expr>, mut values: Vec<Ident>) -> TokenStream2 {
    match exprs.pop() {
        Some(expr) => {
            let value_name = format_ident!("value{}", values.len());
            values.push(value_name.clone());
            let on_ok = build_then_matches(exprs, values);
            quote! {
                match #expr.parse(input) {
                    Ok((#value_name, len)) => {
                        let input = input.slice(len..);
                        total_len += len;
                        #on_ok
                    },
                    Err(error) => Err(error),
                }
            }
        },
        None => {
            quote! {
                Ok(((#(#values,)*), total_len))
            }
        },
    }
}
