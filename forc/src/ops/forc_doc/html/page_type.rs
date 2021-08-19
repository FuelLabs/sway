use core_lang::{
    EnumDeclaration, EnumVariant, FunctionDeclaration, StructDeclaration, StructField,
    TraitDeclaration, TraitFn,
};
use maud::{html, Markup};

const STRUCTS: &'static str = "structs";
const ENUMS: &'static str = "enums";
const TRAITS: &'static str = "traits";
const FUNCTIONS: &'static str = "functions";
pub(crate) const SWAY_TYPES: [&'static str; 4] = [STRUCTS, ENUMS, TRAITS, FUNCTIONS];

pub(crate) enum PageType<'a> {
    Struct(&'a str, &'a Vec<StructField<'a>>),
    Enum(&'a str, &'a Vec<EnumVariant<'a>>),
    Trait(&'a str, &'a Vec<TraitFn<'a>>),
    Function(&'a FunctionDeclaration<'a>),
}

impl<'a> PageType<'a> {
    pub(crate) fn get_name(&'a self) -> &'a str {
        match self {
            &PageType::Struct(name, _) => name,
            &PageType::Enum(name, _) => name,
            &PageType::Trait(name, _) => name,
            &PageType::Function(func_dec) => func_dec.name.primary_name,
        }
    }

    pub(crate) fn build_details(&'a self) -> Option<Markup> {
        match self {
            &PageType::Struct(_, fields) => Some(html! {
                div class="item-table" {
                    @for field in fields {
                        div class="item-row" {
                            div class="item-column-left" {
                                p class="struct" {
                                    (field.name.primary_name)
                                }
                            }

                            div class="item-column-right" {
                                p {
                                    ":"({
                                        let field_type = format!("{:?}", field.r#type);
                                        field_type
                                    })
                                }
                            }
                        }
                    }
                }
            }),
            &PageType::Enum(_, variants) => Some(html! {
                div class="item-table" {
                    @for variant in variants {
                        div class="item-row" {
                            div class="item-column-left" {
                                p class="struct" {
                                    (variant.name.primary_name)
                                }
                            }

                            div class="item-column-right" {
                                p {
                                    ":"({
                                        let variant_type = format!("{:?}", variant.r#type);
                                        variant_type
                                    })
                                }
                            }
                        }
                    }
                }
            }),
            &PageType::Trait(_, functions) => Some(html! {
                div class="item-table" {
                    @for function in functions {
                        div class="item-row" {
                            div class="item-column-left" {
                                p class="struct" {
                                    (function.name.primary_name)"()"
                                }
                            }

                            div class="item-column-right" {
                                p {
                                    ":"({
                                        let function_body = format!("-> {:?}", function.return_type);
                                        function_body
                                    })
                                }
                            }
                        }
                    }
                }
            }),
            &PageType::Function(function) => Some(html! {
                div class="item-table" {
                    div class="item-row" {
                        div class="item-column-left" {
                            p class="struct" {
                                (function.name.primary_name)"()"
                            }
                        }

                        div class="item-column-right" {
                            p {
                                ":"({
                                    let function_body = format!("-> {:?}", function.return_type);
                                    function_body
                                })
                            }
                        }
                    }

                }
            }),
            _ => None,
        }
    }

    pub(crate) fn get_type_key(&'a self) -> &'static str {
        match self {
            &PageType::Struct(_, _) => STRUCTS,
            &PageType::Enum(_, _) => ENUMS,
            &PageType::Trait(_, _) => TRAITS,
            &PageType::Function(_) => FUNCTIONS,
        }
    }

    pub(crate) fn get_type(&'a self) -> &str {
        match self {
            &PageType::Struct(_, _) => "Struct",
            &PageType::Enum(_, _) => "Enum",
            &PageType::Trait(_, _) => "Trait",
            &PageType::Function(_) => "Function",
        }
    }
}

impl<'a> From<&'a StructDeclaration<'a>> for PageType<'a> {
    fn from(struct_dec: &'a StructDeclaration<'a>) -> Self {
        let name = struct_dec.name.primary_name;
        let fields = &struct_dec.fields;

        PageType::Struct(name, fields)
    }
}

impl<'a> From<&'a EnumDeclaration<'a>> for PageType<'a> {
    fn from(enum_dec: &'a EnumDeclaration<'a>) -> Self {
        let name = enum_dec.name.primary_name;
        let variants = &enum_dec.variants;

        PageType::Enum(name, variants)
    }
}

impl<'a> From<&'a TraitDeclaration<'a>> for PageType<'a> {
    fn from(trait_dec: &'a TraitDeclaration<'a>) -> Self {
        let name = trait_dec.name.primary_name;
        let methods = &trait_dec.interface_surface;

        PageType::Trait(name, methods)
    }
}

impl<'a> From<&'a FunctionDeclaration<'a>> for PageType<'a> {
    fn from(func_dec: &'a FunctionDeclaration<'a>) -> Self {
        PageType::Function(func_dec)
    }
}
