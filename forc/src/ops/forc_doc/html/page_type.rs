use core_lang::{StructDeclaration, StructField};

const STRUCTS: &'static str = "structs";
pub(crate) const SWAY_TYPES: [&'static str; 1] = [STRUCTS];

pub(crate) enum PageType<'a> {
    Struct(&'a str, &'a Vec<StructField<'a>>),
}

impl<'a> PageType<'a> {
    pub(crate) fn get_name(&'a self) -> &'a str {
        match self {
            &PageType::Struct(name, _) => name,
        }
    }

    pub(crate) fn is_struct(&'a self) -> bool {
        match self {
            &PageType::Struct(_, _) => true,
            _ => false,
        }
    }

    pub(crate) fn get_fields(&'a self) -> &Vec<StructField<'a>> {
        match self {
            &PageType::Struct(_, fields) => fields,
        }
    }

    pub(crate) fn get_type_key(&'a self) -> &'static str {
        match self {
            &PageType::Struct(_, _) => STRUCTS,
        }
    }

    pub(crate) fn get_type(&'a self) -> &str {
        match self {
            &PageType::Struct(_, _) => "Struct",
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
