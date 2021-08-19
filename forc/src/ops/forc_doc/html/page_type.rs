use core_lang::{EnumDeclaration, EnumVariant, StructDeclaration, StructField};
use maud::{html, Markup};

const STRUCTS: &'static str = "structs";
const ENUMS: &'static str = "enums";
pub(crate) const SWAY_TYPES: [&'static str; 2] = [STRUCTS, ENUMS];

pub(crate) enum PageType<'a> {
    Struct(&'a str, &'a Vec<StructField<'a>>),
    Enum(&'a str, &'a Vec<EnumVariant<'a>>),
}

impl<'a> PageType<'a> {
    pub(crate) fn get_name(&'a self) -> &'a str {
        match self {
            &PageType::Struct(name, _) => name,
            &PageType::Enum(name, _) => name,
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
            _ => None,
        }
    }

    pub(crate) fn get_type_key(&'a self) -> &'static str {
        match self {
            &PageType::Struct(_, _) => STRUCTS,
            &PageType::Enum(_, _) => ENUMS,
        }
    }

    pub(crate) fn get_type(&'a self) -> &str {
        match self {
            &PageType::Struct(_, _) => "Struct",
            &PageType::Enum(_, _) => "Enum",
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
    fn from(struct_dec: &'a EnumDeclaration<'a>) -> Self {
        let name = struct_dec.name.primary_name;
        let variants = &struct_dec.variants;

        PageType::Enum(name, variants)
    }
}
