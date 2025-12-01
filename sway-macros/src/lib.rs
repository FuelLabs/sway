use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use sway_types::style::to_snake_case;
use syn::{
    Attribute, Ident, Item, LitBool, Token, Type, TypeGenerics, parse_macro_input
};

enum CallStrategy {
    DoNotCall,
    SimpleCall,
    IterCall,
}

struct ParsedAttributes {
    call_visit: CallStrategy,
    call_visitor: bool,
    skip: bool,
    optional: bool,
}

fn parse_attributes(attrs: &[Attribute]) -> ParsedAttributes {
    let mut call_visit = CallStrategy::DoNotCall;
    let mut call_visitor = true;
    let mut skip = false;
    let mut optional = false;
    
    for att in attrs.iter() {
        match &att.meta {
            syn::Meta::List(meta_list) => {
                let a = meta_list.path.segments.first().unwrap();
                if a.ident == "visit" {
                    for token in meta_list.tokens.to_token_stream() {
                        if token.to_string() == "," {
                            continue;
                        }

                        if token.to_string() == " " {
                            continue;
                        }

                        if token.to_string() == "skip" {
                            skip = true;
                        } else  if token.to_string() == "optional" {
                            optional = true;
                        } else if token.to_string() == "call_visit" {
                            call_visit = CallStrategy::SimpleCall;
                        } else if token.to_string() == "do_not_call_visit" {
                            call_visit = CallStrategy::DoNotCall;
                        } else if token.to_string() == "call_visitor" {
                            call_visitor = true;
                        } else if token.to_string() == "do_not_call_visitor" {
                            call_visitor = false;
                        } else if token.to_string() == "iter_visit" {
                            call_visit = CallStrategy::IterCall;
                        } else {
                            panic!("Unknown attribute: {}", token.to_string())
                        }
                    }
                }
            },
            _ => {}
        }
    }

    ParsedAttributes {
        call_visit,
        call_visitor,
        skip,
        optional,
    }
}

#[proc_macro_derive(Visit, attributes(visit))]
pub fn derive_visit(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    let mut q = quote! {};

    let mut anon_identifiers = "qwertyuiopasdfghjklzxcvbnm"
        .chars()
        .map(|x| format_ident!("{x}"));

    match input {
        Item::Enum(e) => {
            let mut arms = quote! {};

            for variant in e.variants.iter() {
                let mut arm_fields_pattern = quote! {};
                let mut arm_fields_into_owned = quote! {};
                let mut arm_body = quote! {};

                let fields_len = variant.fields.len();
                let mut has_name = false;

                for f in variant.fields.iter() {
                    let field_ident =
                        get_variant_field_ident(&mut anon_identifiers, &mut has_name, f);
                    arm_fields_pattern.extend(quote! {#field_ident,});

                    if has_name {
                        arm_fields_into_owned
                            .extend(quote! {#field_ident: #field_ident.into_owned(),});
                    } else {
                        arm_fields_into_owned.extend(quote! {#field_ident.into_owned(),});
                    }

                    let field_type_as_snake = to_snake_case(&get_type_as_string(&f.ty));
                    let visit_fn = format_ident!("visit_{}", field_type_as_snake);
                    arm_body.extend(quote! {
                        let #field_ident = visitor.#visit_fn(#field_ident);
                        has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                    });
                }

                let fields_pattern = match (has_name, fields_len) {
                    (true, _) => quote! { {#arm_fields_pattern} },
                    (false, 0) => quote! {},
                    (false, _) => quote! { (#arm_fields_pattern) },
                };

                let arm_fields_into_owned = match (has_name, fields_len) {
                    (true, _) => quote! {{#arm_fields_into_owned}},
                    (false, 0) => quote! {},
                    (false, _) => quote! {(#arm_fields_into_owned)},
                };

                let variant_name = format_ident!("{}", variant.ident);
                arms.extend(quote! {
                    Self::#variant_name #fields_pattern => {
                        #[allow(unused_variables)]
                        let mut has_changes: bool = false;
                        #arm_body

                        if has_changes {
                            std::borrow::Cow::Owned(Self::#variant_name #arm_fields_into_owned)
                        } else {
                            std::borrow::Cow::Borrowed(self)
                        }
                    }
                });
            }

            let enum_name = format_ident!("{}", e.ident);
            q.extend(quote!{
                impl #enum_name {
                    pub fn visit<V: crate::semantic_analysis::Visitor>(&self, visitor: &mut V) -> std::borrow::Cow<Self> {
                        match self {
                            #arms
                        }
                    }
                }
            });
        }
        Item::Struct(s) => {
            let mut fields_pattern = quote! {};
            let mut fields_into_owned = quote! {};
            let mut body = quote!{};

            for field in s.fields.iter() {
                let attrs = parse_attributes(&field.attrs);

                let field_ident = field.ident.as_ref().unwrap();
                fields_pattern.extend(quote!{
                    #field_ident,
                });
                fields_into_owned.extend(quote!{
                    #field_ident: #field_ident.into_owned(),
                });

                body.extend(quote! {
                    #[allow(unused_mut)]
                    let mut #field_ident = std::borrow::Cow::Borrowed(#field_ident);
                });

                if !attrs.skip {
                    if attrs.call_visitor {
                        let field_type_as_snake = to_snake_case(&get_type_as_string(&field.ty));
                        let visit_fn = format_ident!("visit_{}", field_type_as_snake);
                        body.extend(quote! {
                            visitor.#visit_fn(&mut #field_ident);
                            has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                        });
                    }

                    match attrs.call_visit {
                        CallStrategy::DoNotCall => {},
                        CallStrategy::SimpleCall => {
                            let ty = field.ty.clone();
                            body.extend(quote! {
                                <#ty>::visit(&mut #field_ident, visitor);
                                has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                            });
                        },
                        CallStrategy::IterCall => {
                            let ty = get_vec_inner_type(&field.ty);
                            body.extend(quote! {
                                for idx in 0..#field_ident.len() {
                                    let mut item = std::borrow::Cow::Borrowed(&#field_ident[idx]);
                                    <#ty>::visit(&mut item, visitor);
                                    if let std::borrow::Cow::Owned(item) = item {
                                        #field_ident.to_mut()[idx] = item
                                    }
                                }
                                has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                            });
                        },
                    }
                }
            }

            let struct_name = format_ident!("{}", s.ident);
            q.extend(quote!{
                impl #struct_name {
                    pub fn visit<V: crate::semantic_analysis::Visitor>(s: &mut std::borrow::Cow<Self>, visitor: &mut V) {
                        let Self {
                            #fields_pattern
                        } = s.as_ref();

                        let mut has_changes: bool = false;
                        #body

                        if has_changes {
                            *s = std::borrow::Cow::Owned(Self { #fields_into_owned })
                        }
                    }
                }
            });
        },
        _ => panic!("Visit derive only work on enums and structs"),
    };

    q.into()
}

fn get_vec_inner_type(vec: &Type) -> &Type {
    match vec {
        Type::Path(type_path) => {
            let vec = type_path.path.segments.first().unwrap();
            if vec.ident.to_string() != "Vec" {
                panic!("visit only knows how to iter Vec");
            }

            match &vec.arguments {
                syn::PathArguments::None => todo!(),
                syn::PathArguments::AngleBracketed(args) => {
                    match &args.args[0] {
                        syn::GenericArgument::Type(t) => t,
                        _ => todo!(),
                    }
                },
                syn::PathArguments::Parenthesized(_) => todo!(),
            }
        },
        _ => todo!(),
    }
}

#[derive(Default)]
struct VisitorGenerator {
    consts: Vec<(Ident, LitBool)>,
    types: Vec<Type>,
}

impl syn::parse::Parse for VisitorGenerator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut g = VisitorGenerator::default();

        while !input.is_empty() {
            if let Ok(_) = input.parse::<Token![const]>() {
                let ident = input.parse::<Ident>().unwrap();
                let _ = input.parse::<Token![:]>().unwrap();
                let _ = input.parse::<Type>().unwrap();
                let _ = input.parse::<Token![=]>().unwrap();
                let init = input.parse::<LitBool>().unwrap();
                let _ = input.parse::<Token![,]>().unwrap();
                g.consts.push((ident, init));
                continue;
            }

            if let Ok(t) = input.parse::<Type>() {
                let _ = input.parse::<Token![,]>().unwrap();
                g.types.push(t);
                continue;
            }

            panic!("Unsupported item. Failed at: {}", input.to_string());
        }

        Ok(g)
    }
}

#[proc_macro]
pub fn generate_visitor(input: TokenStream) -> TokenStream {
    let g = parse_macro_input!(input as VisitorGenerator);

    let mut fns = quote! {};
    let mut consts = quote! {};

    for t in g.types {
        let visit_fn = to_snake_case(&get_type_as_string(&t));
        let visit_fn = format_ident!("visit_{}", visit_fn);
        fns.extend(quote! {
            fn #visit_fn<'a>(&mut self, item: &'a #t) -> std::borrow::Cow<'a, #t> {
                std::borrow::Cow::Borrowed(item)
            }
        });
    }

    for (name, value) in g.consts {
        consts.extend(quote! {
            const #name: bool = #value;
        });
    }

    quote! {
        pub trait Visitor {
            #consts
            #fns
        }
    }
    .into()
}

fn get_variant_field_ident(
    anon_identifiers: &mut impl Iterator<Item = syn::Ident>,
    has_name: &mut bool,
    f: &syn::Field,
) -> syn::Ident {
    let ident = if let Some(ident) = &f.ident {
        *has_name = true;
        if ident == "r#else" {
            format_ident!("r#else")
        } else {
            format_ident!("{}", ident)
        }
    } else {
        *has_name = false;
        anon_identifiers.next().unwrap()
    };
    ident
}

fn get_type_as_string(t: &Type) -> String {
    match t {
        Type::Array(type_array) => {
            let elem = get_type_as_string(&type_array.elem);
            let len = type_array.len.to_token_stream().to_string();
            format!("_{elem}_{len}")
        }
        Type::BareFn(_type_bare_fn) => todo!("BareFn"),
        Type::Group(_type_group) => todo!("Group"),
        Type::ImplTrait(_type_impl_trait) => todo!("ImplTrait"),
        Type::Infer(_type_infer) => todo!("Infer"),
        Type::Macro(_type_macro) => todo!("Macro"),
        Type::Never(_type_never) => todo!("Never"),
        Type::Paren(_type_paren) => todo!("Paren"),
        Type::Path(type_path) => {
            let mut s = String::new();

            if let Some(segment) = type_path.path.segments.last() {
                s.push_str(segment.ident.to_string().as_str());
                match &segment.arguments {
                    syn::PathArguments::None => {}
                    syn::PathArguments::AngleBracketed(a) => {
                        for arg in a.args.iter() {
                            match arg {
                                syn::GenericArgument::Lifetime(_lifetime) => todo!(),
                                syn::GenericArgument::Type(t) => {
                                    s.push_str(&get_type_as_string(t));
                                }
                                syn::GenericArgument::Const(_expr) => todo!(),
                                syn::GenericArgument::AssocType(_assoc_type) => todo!(),
                                syn::GenericArgument::AssocConst(_assoc_const) => todo!(),
                                syn::GenericArgument::Constraint(_constraint) => todo!(),
                                _ => todo!(),
                            }
                        }
                    }
                    syn::PathArguments::Parenthesized(_parenthesized_generic_arguments) => todo!(),
                }
            }

            s
        }
        Type::Ptr(_type_ptr) => todo!("Ptr"),
        Type::Reference(_type_reference) => todo!("Reference"),
        Type::Slice(_type_slice) => todo!("Slice"),
        Type::TraitObject(_type_trait_object) => todo!("TraitObject"),
        Type::Tuple(type_tuple) => {
            let mut s = String::new();
            for elem in type_tuple.elems.iter() {
                s.push_str("_");
                s.push_str(&get_type_as_string(elem));
            }
            s
        }
        Type::Verbatim(_token_stream) => todo!("Verbatim"),
        _ => todo!(),
    }
}
