use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use sway_types::style::to_snake_case;
use syn::{parse_macro_input, Attribute, Ident, Item, LitBool, Token, Type};

const DEBUG: bool = false;

#[derive(Debug)]
enum CallStrategy {
    DoNotCall,
    SimpleCall,
}

#[derive(Debug)]
struct ParsedAttributes {
    call_visit: CallStrategy,
    call_visitor: bool,
    skip: bool,
    optional: bool,
    leaf: bool,
}

fn parse_attributes(attrs: &[Attribute]) -> ParsedAttributes {
    let mut call_visit = CallStrategy::SimpleCall;
    let mut call_visitor = false;
    let mut skip = false;
    let mut optional = false;
    let mut leaf = false;

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
                        } else if token.to_string() == "optional" {
                            optional = true;
                        } else if token.to_string() == "call_visit" {
                            call_visit = CallStrategy::SimpleCall;
                        } else if token.to_string() == "do_not_call_visit" {
                            call_visit = CallStrategy::DoNotCall;
                        } else if token.to_string() == "call_visitor" {
                            call_visitor = true;
                        } else if token.to_string() == "do_not_call_visitor" {
                            call_visitor = false;
                        } else if token.to_string() == "leaf" {
                            leaf = true;
                        } else {
                            panic!("Unknown attribute: {}", token.to_string())
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ParsedAttributes {
        call_visit,
        call_visitor,
        skip,
        optional,
        leaf,
    }
}

#[proc_macro_derive(Visit, attributes(visit))]
pub fn derive_visit(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);

    let mut q = quote! {};

    let anon_identifiers = "qwertyuiopasdfghjklzxcvbnm".repeat(100);
    let mut anon_identifiers = anon_identifiers.chars().map(|x| format_ident!("{x}"));

    match input {
        Item::Enum(e) => {
            let attrs = parse_attributes(&e.attrs);
            let enum_name = format_ident!("{}", e.ident);

            if attrs.leaf {
                let field_type_as_snake = to_snake_case(&e.ident.to_string());
                let visit_fn = format_ident!("visit_{}", field_type_as_snake);
                let dbg_tokens = if DEBUG {
                    quote! {dbg!(std::any::type_name::<Self>());}
                } else {
                    quote! {}
                };
                q.extend(quote!{
                    impl #enum_name {
                        pub fn visit<V: crate::semantic_analysis::Visitor>(cow: &mut std::borrow::Cow<Self>, visitor: &mut V) {
                            #dbg_tokens
                            visitor.#visit_fn(cow);
                        }
                    }
                });
            } else {
                let mut arms = quote! {};

                for variant in e.variants.iter() {
                    let attrs = parse_attributes(&variant.attrs);
                    let skip_variant = attrs.skip;

                    let variant_name = format_ident!("{}", variant.ident);

                    let mut arm_fields_pattern = quote! {};
                    let mut arm_fields_into_owned = quote! {};
                    let mut arm_body = quote! {};

                    if !skip_variant {
                        let fields_len = variant.fields.len();
                        let mut has_name = false;

                        for variant_field in variant.fields.iter() {
                            let attrs = parse_attributes(&variant_field.attrs);

                            let field_ident = get_variant_field_ident(
                                &mut anon_identifiers,
                                &mut has_name,
                                variant_field,
                            );
                            arm_fields_pattern.extend(quote! {#field_ident,});

                            if has_name {
                                arm_fields_into_owned
                                    .extend(quote! {#field_ident: #field_ident.into_owned(),});
                            } else {
                                arm_fields_into_owned.extend(quote! {#field_ident.into_owned(),});
                            }

                            arm_body.extend(quote! {
                                #[allow(unused_mut)]
                                let mut #field_ident = std::borrow::Cow::Borrowed(#field_ident);
                            });

                            if !attrs.skip {
                                if attrs.call_visitor {
                                    let full_type = get_known_type(&variant_field.ty);
                                    match full_type.as_str() {
                                        "Vec" => {
                                            let inner_ty =
                                                get_first_generic_argument(&variant_field.ty).unwrap();
                                            let sanitized_type_name =
                                                sanitize_type_name(&get_type_as_string(inner_ty));
                                            let field_type_as_snake =
                                                to_snake_case(&sanitized_type_name);
                                            let visit_fn =
                                                format_ident!("visit_{}", field_type_as_snake);
                                            arm_body.extend(quote! {
                                                for idx in 0..#field_ident.len() {
                                                    let mut item = std::borrow::Cow::Borrowed(&#field_ident[idx]);
                                                    visitor.#visit_fn(&mut item);
                                                    if let std::borrow::Cow::Owned(item) = item {
                                                        #field_ident.to_mut()[idx] = item;
                                                    }
                                                }
                                                has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                                            });
                                        }
                                        "Option" => {
                                            let inner_ty =
                                                get_first_generic_argument(&variant_field.ty).unwrap();
                                            let sanitized_type_name =
                                                sanitize_type_name(&get_type_as_string(inner_ty));
                                            let field_type_as_snake =
                                                to_snake_case(&sanitized_type_name);
                                            let visit_fn =
                                                format_ident!("visit_{}", field_type_as_snake);
                                            arm_body.extend(quote! {
                                                if let Some(v) = #field_ident.as_ref() {
                                                    let mut v = std::borrow::Cow::Borrowed(v);
                                                    visitor.#visit_fn(&mut v);
                                                    if let std::borrow::Cow::Owned(v) = v {
                                                        #field_ident = std::borrow::Cow::Owned(Some(v));
                                                    }
                                                }
                                                has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                                            });
                                        }
                                        "Option_Box" => {
                                            let inner_ty =
                                                get_first_generic_argument(&variant_field.ty).unwrap();
                                            let inner_ty =
                                                get_first_generic_argument(&inner_ty).unwrap();

                                            let sanitized_type_name =
                                                sanitize_type_name(&get_type_as_string(inner_ty));
                                            let field_type_as_snake =
                                                to_snake_case(&sanitized_type_name);
                                            let visit_fn =
                                                format_ident!("visit_{}", field_type_as_snake);

                                            arm_body.extend(quote! {
                                                if let Some(v) = #field_ident.as_ref() {
                                                    let mut v = std::borrow::Cow::Borrowed(Box::as_ref(&v));
                                                    visitor.#visit_fn(&mut v);
                                                    if let std::borrow::Cow::Owned(v) = v {
                                                        #field_ident = std::borrow::Cow::Owned(Some(Box::new(v)));
                                                    }
                                                }
                                                has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                                            });
                                        }
                                        "Box" => {
                                            let inner_ty = get_first_generic_argument(&variant_field.ty).unwrap();

                                            let sanitized_type_name =
                                                sanitize_type_name(&get_type_as_string(inner_ty));
                                            let field_type_as_snake =
                                                to_snake_case(&sanitized_type_name);
                                            let visit_fn =
                                                format_ident!("visit_{}", field_type_as_snake);

                                            arm_body.extend(quote! {
                                                let mut v = std::borrow::Cow::Borrowed(Box::as_ref(&#field_ident));
                                                visitor.#visit_fn(&mut v);
                                                if let std::borrow::Cow::Owned(v) = v {
                                                    #field_ident = std::borrow::Cow::Owned(Box::new(v));
                                                }
                                                has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                                            });
                                        }
                                        "Box_Option" => {
                                            todo!("Box_Option")
                                        }
                                        _ => {
                                            let sanitized_type_name =
                                                sanitize_type_name(&get_type_as_string(&variant_field.ty));
                                            let field_type_as_snake =
                                                to_snake_case(&sanitized_type_name);
                                            let visit_fn =
                                                format_ident!("visit_{}", field_type_as_snake);
                                            arm_body.extend(quote! {
                                                visitor.#visit_fn(&mut #field_ident);
                                                has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                                            });
                                        }
                                    }
                                }

                                arm_body.extend(generate_call_visit(
                                    &attrs,
                                    &variant_field.ty,
                                    &field_ident,
                                ));
                            }
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

                        let dbg_tokens = if DEBUG {
                            quote! {dbg!(format!("{}::{}",std::any::type_name::<Self>(), stringify!(#variant_name)));}
                        } else {
                            quote! {}
                        };
                        arms.extend(quote! {
                            Self::#variant_name #fields_pattern => {
                                #dbg_tokens

                                #[allow(unused_variables)]
                                let mut has_changes: bool = false;

                                #arm_body

                                if has_changes {
                                    *cow = std::borrow::Cow::Owned(Self::#variant_name #arm_fields_into_owned);
                                }
                            }
                        });
                    } else {
                        let mut has_name = false;
                        for f in variant.fields.iter() {
                            has_name = f.ident.is_some()
                        }

                        let fields_pattern = if has_name {
                            quote! { {..} }
                        } else {
                            quote! { (..) }
                        };

                        let dbg_tokens = if DEBUG {
                            quote! {dbg!(format!("{}::{}",std::any::type_name::<Self>(), stringify!(#variant_name)));}
                        } else {
                            quote! {}
                        };
                        arms.extend(quote! {
                            Self::#variant_name #fields_pattern => {#dbg_tokens}
                        });
                    }
                }

                q.extend(quote!{
                    impl #enum_name {
                        pub fn visit<V: crate::semantic_analysis::Visitor>(cow: &mut std::borrow::Cow<Self>, visitor: &mut V) {
                            match cow.as_ref() {
                                #arms
                            }
                        }
                    }
                });
            }
        }
        Item::Struct(s) => {
            let attrs = parse_attributes(&s.attrs);
            let struct_name = format_ident!("{}", s.ident);

            if attrs.leaf {
                let field_type_as_snake = to_snake_case(&s.ident.to_string());
                let visit_fn = format_ident!("visit_{}", field_type_as_snake);
                let dbg_tokens = if DEBUG {
                    quote! {dbg!(std::any::type_name::<Self>());}
                } else {
                    quote! {}
                };
                q.extend(quote!{
                    impl #struct_name {
                        pub fn visit<V: crate::semantic_analysis::Visitor>(cow: &mut std::borrow::Cow<Self>, visitor: &mut V) {
                            #dbg_tokens
                            visitor.#visit_fn(cow);
                        }
                    }
                });
            } else {
                let mut fields_pattern = quote! {};
                let mut fields_into_owned = quote! {};
                let mut body = quote! {};

                for field in s.fields.iter() {
                    let attrs = parse_attributes(&field.attrs);

                    let field_ident = field.ident.as_ref().unwrap();
                    fields_pattern.extend(quote! {
                        #field_ident,
                    });
                    fields_into_owned.extend(quote! {
                        #field_ident: #field_ident.into_owned(),
                    });

                    body.extend(quote! {
                        #[allow(unused_mut)]
                        let mut #field_ident = std::borrow::Cow::Borrowed(#field_ident);
                    });

                    if !attrs.skip {
                        if attrs.call_visitor {
                            let sanitized_type_name =
                                sanitize_type_name(&get_type_as_string(&field.ty));
                            let field_type_as_snake = to_snake_case(&sanitized_type_name);
                            let visit_fn = format_ident!("visit_{}", field_type_as_snake);
                            body.extend(quote! {
                                visitor.#visit_fn(&mut #field_ident);
                                has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                            });
                        }

                        body.extend(generate_call_visit(&attrs, &field.ty, field_ident));
                    }
                }

                let dbg_tokens = if DEBUG {
                    quote! {dbg!(std::any::type_name::<Self>());}
                } else {
                    quote! {}
                };
                q.extend(quote!{
                    impl #struct_name {
                        pub fn visit<V: crate::semantic_analysis::Visitor>(cow: &mut std::borrow::Cow<Self>, visitor: &mut V) {
                            #dbg_tokens

                            let Self {
                                #fields_pattern
                            } = cow.as_ref();

                            let mut has_changes: bool = false;
                            #body

                            if has_changes {
                                *cow = std::borrow::Cow::Owned(Self { #fields_into_owned })
                            }
                        }
                    }
                });
            }
        }
        _ => panic!("Visit derive only work on enums and structs"),
    };

    q.into()
}

fn get_known_type(ty: &syn::Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let type_segment = type_path.path.segments.last().unwrap();
            let mut full_type = type_segment.ident.to_string();

            if full_type.as_str() == "Option" {
                let inner_ty = get_first_generic_argument(&ty).unwrap();
                match inner_ty {
                    Type::Path(type_path) => {
                        let inner_segment = type_path.path.segments.last().unwrap();
                        if inner_segment.ident.to_string() == "Box" {
                            full_type = "Option_Box".to_string()
                        }
                    }
                    _ => {}
                }
            } else if full_type.as_str() == "Box" {
                let inner_ty = get_first_generic_argument(&ty).unwrap();
                match inner_ty {
                    Type::Path(type_path) => {
                        let first_segment = type_path.path.segments.first().unwrap();
                        if first_segment.ident.to_string() == "Option" {
                            full_type = "Box_Option".to_string()
                        }
                    }
                    _ => {}
                }
            }
        
            full_type
        }
        _ => {
            todo!()
        }
    }
}

fn generate_call_visit(
    attrs: &ParsedAttributes,
    ty: &syn::Type,
    field_ident: &Ident,
) -> proc_macro2::TokenStream {
    match attrs.call_visit {
        CallStrategy::DoNotCall => quote! {},
        CallStrategy::SimpleCall => {
            let full_type = get_known_type(ty);
            match full_type.as_str() {
                "Vec" => {
                    let inner_ty = get_first_generic_argument(&ty);
                    quote! {
                        for idx in 0..#field_ident.len() {
                            let mut item = std::borrow::Cow::Borrowed(&#field_ident[idx]);
                            <#inner_ty>::visit(&mut item, visitor);
                            if let std::borrow::Cow::Owned(item) = item {
                                #field_ident.to_mut()[idx] = item;
                            }
                        }
                        has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                    }
                }
                "Option" => {
                    let inner_ty = get_first_generic_argument(&ty);
                    quote! {
                        if let Some(v) = #field_ident.as_ref() {
                            let mut item = std::borrow::Cow::Borrowed(v);
                            <#inner_ty>::visit(&mut item, visitor);
                            if let std::borrow::Cow::Owned(item) = item {
                                *#field_ident.to_mut() = Some(item);
                            }
                        }
                        has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                    }
                }
                "Option_Box" => {
                    let inner_ty = get_first_generic_argument(&ty).unwrap();
                    let inner_ty = get_first_generic_argument(&inner_ty).unwrap();
                    quote! {
                        if let Some(v) = #field_ident.as_ref() {
                            let mut item = std::borrow::Cow::Borrowed(Box::as_ref(&v));
                            <#inner_ty>::visit(&mut item, visitor);
                            if let std::borrow::Cow::Owned(item) = item {
                                *#field_ident.to_mut() = Some(Box::new(item));
                            }
                        }
                        has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                    }
                }
                "Box" => {
                    let inner_ty = get_first_generic_argument(&ty).unwrap();
                    quote! {
                        let mut item = std::borrow::Cow::Borrowed(Box::as_ref(&#field_ident));
                        <#inner_ty>::visit(&mut item, visitor);
                        if let std::borrow::Cow::Owned(item) = item {
                            *#field_ident.to_mut() = Box::new(item);
                        }
                        has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                    }
                }
                "Box_Option" => {
                    todo!("Box_Option")
                }
                _ => {
                    quote! {
                        <#ty>::visit(&mut #field_ident, visitor);
                        has_changes |= matches!(#field_ident, std::borrow::Cow::Owned(_));
                    }
                }
            }
        }
    }
}

fn get_first_generic_argument(vec: &Type) -> Option<&Type> {
    match vec {
        Type::Path(type_path) => {
            let t = type_path.path.segments.last().unwrap();
            match &t.arguments {
                syn::PathArguments::None => None,
                syn::PathArguments::AngleBracketed(args) => match &args.args[0] {
                    syn::GenericArgument::Type(t) => Some(t),
                    _ => todo!("2"),
                },
                syn::PathArguments::Parenthesized(_) => todo!("3"),
            }
        }
        _ => todo!("4"),
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
        let sanitized_type = sanitize_type_name(&get_type_as_string(&t));
        let visit_fn = to_snake_case(&sanitized_type);
        let visit_fn = format_ident!("visit_{}", visit_fn);
        fns.extend(quote! {
            fn #visit_fn(&mut self, item: &mut std::borrow::Cow<#t>) {
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

fn sanitize_type_name(s: &str) -> String {
    s.replace("[", "_slice_")
        .replace("]", "_")
        .replace("<", "_")
        .replace(">", "_")
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
                                syn::GenericArgument::Lifetime(_lifetime) => todo!("Lifetime"),
                                syn::GenericArgument::Type(t) => {
                                    s.push_str(&get_type_as_string(t));
                                }
                                syn::GenericArgument::Const(_expr) => todo!("Const"),
                                syn::GenericArgument::AssocType(_assoc_type) => todo!("AssocType"),
                                syn::GenericArgument::AssocConst(_assoc_const) => {
                                    todo!("AssocConst")
                                }
                                syn::GenericArgument::Constraint(_constraint) => {
                                    todo!("Constraint")
                                }
                                _ => todo!("Z"),
                            }
                        }
                    }
                    syn::PathArguments::Parenthesized(_parenthesized_generic_arguments) => {
                        todo!("Parenthesized")
                    }
                }
            }

            s
        }
        Type::Ptr(_type_ptr) => todo!("Ptr"),
        Type::Reference(_type_reference) => todo!("Reference"),
        Type::Slice(type_slice) => {
            format!("[{}]", get_type_as_string(&type_slice.elem))
        }
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
        _ => todo!("W"),
    }
}
