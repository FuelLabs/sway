use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident,
    Meta, NestedMeta, Variant,
};

#[proc_macro_derive(DebugWithContext, attributes(in_context))]
pub fn derive_debug_with_context(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let type_name = ident.to_string();
    let body = match data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let (field_names, fmt_fields) = fmt_fields_named(&type_name, fields_named);
                quote! {
                    let #ident { #(#field_names,)* } = self;
                    #fmt_fields
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                let (field_names, fmt_fields) = fmt_fields_unnamed(&type_name, fields_unnamed);
                quote! {
                    let #ident(#(#field_names,)*) = self;
                    #fmt_fields
                }
            }
            Fields::Unit => {
                quote! {
                    formatter.write_str(#type_name)
                }
            }
        },
        Data::Enum(data_enum) => {
            let branches = {
                data_enum.variants.iter().map(|variant| {
                    let Variant {
                        ident: variant_ident,
                        fields,
                        ..
                    } = variant;
                    let type_variant_name = format!("{}::{}", type_name, variant_ident);
                    match fields {
                        Fields::Named(fields_named) => {
                            let (field_names, fmt_fields) =
                                fmt_fields_named(&type_variant_name, fields_named);
                            quote! {
                                #ident::#variant_ident { #(#field_names,)* } => {
                                    #fmt_fields
                                },
                            }
                        }
                        Fields::Unnamed(fields_unnamed) => {
                            let (field_names, fmt_fields) =
                                fmt_fields_unnamed(&type_variant_name, fields_unnamed);
                            quote! {
                                #ident::#variant_ident(#(#field_names,)*) => {
                                    #fmt_fields
                                },
                            }
                        }
                        Fields::Unit => {
                            quote! {
                                #ident::#variant_ident => {
                                    formatter.write_str(#type_variant_name)
                                },
                            }
                        }
                    }
                })
            };
            quote! {
                match self {
                    #(#branches)*
                }
            }
        }
        Data::Union(_) => {
            panic!("#[derive(DebugWithContext)] cannot be used on unions");
        }
    };
    let output = quote! {
        impl #impl_generics DebugWithContext for #ident #ty_generics
        #where_clause
        {
            fn fmt_with_context<'a, 'c>(
                &'a self,
                formatter: &mut std::fmt::Formatter,
                context: &'c Context,
            ) -> std::fmt::Result {
                #body
            }
        }
    };
    output.into()
}

fn fmt_fields_named<'i>(
    name: &str,
    fields_named: &'i FieldsNamed,
) -> (Vec<&'i Ident>, proc_macro2::TokenStream) {
    let field_names = {
        fields_named
            .named
            .iter()
            .map(|field| field.ident.as_ref().unwrap())
            .collect::<Vec<_>>()
    };
    let fmt_fields = {
        fields_named
            .named
            .iter()
            .zip(field_names.iter())
            .map(|(field, name)| {
                let name_str = name.to_string();
                let expr = pass_through_context(name, &field.attrs);
                quote! {
                    debug_struct = debug_struct.field(#name_str, &#expr);
                }
            })
    };
    let token_tree = quote! {
        let mut debug_struct = &mut formatter.debug_struct(#name);
        #(#fmt_fields)*
        debug_struct.finish()
    };
    (field_names, token_tree)
}

fn fmt_fields_unnamed(
    name: &str,
    fields_unnamed: &FieldsUnnamed,
) -> (Vec<Ident>, proc_macro2::TokenStream) {
    let field_names = {
        (0..fields_unnamed.unnamed.len())
            .map(|i| format_ident!("field_{}", i))
            .collect::<Vec<_>>()
    };
    let fmt_fields = {
        fields_unnamed
            .unnamed
            .iter()
            .zip(field_names.iter())
            .map(|(field, name)| {
                let expr = pass_through_context(name, &field.attrs);
                quote! {
                    debug_tuple = debug_tuple.field(&#expr);
                }
            })
    };
    let token_tree = quote! {
        let mut debug_tuple = &mut formatter.debug_tuple(#name);
        #(#fmt_fields)*
        debug_tuple.finish()
    };
    (field_names, token_tree)
}

fn pass_through_context(field_name: &Ident, attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let context_field_opt = {
        attrs
            .iter()
            .filter_map(|attr| {
                let attr_name = attr.path.get_ident()?;
                if attr_name != "in_context" {
                    return None;
                }
                let context_field = {
                    try_parse_context_field_from_attr(attr)
                        .expect("malformed #[in_context(..)] attribute")
                };
                Some(context_field)
            })
            .dedup()
            .at_most_one()
            .expect("multiple #[in_context(..)] attributes on field")
    };
    match context_field_opt {
        None => {
            quote! {
                #field_name.with_context(context)
            }
        }
        Some(context_field) => {
            quote! {
                context.#context_field[*#field_name].with_context(context)
            }
        }
    }
}

fn try_parse_context_field_from_attr(attr: &Attribute) -> Option<Ident> {
    let meta = attr.parse_meta().ok()?;
    let meta_list = match meta {
        Meta::List(meta_list) => meta_list,
        _ => return None,
    };
    if meta_list.nested.len() != 1 {
        return None;
    }
    let nested_meta = meta_list.nested.first()?;
    let inner_meta = match nested_meta {
        NestedMeta::Meta(inner_meta) => inner_meta,
        _ => return None,
    };
    let path = match inner_meta {
        Meta::Path(path) => path,
        _ => return None,
    };
    let context_field_name = path.get_ident()?.clone();
    Some(context_field_name)
}
