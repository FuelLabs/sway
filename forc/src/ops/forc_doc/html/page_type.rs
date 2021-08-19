use core_lang::StructDeclaration;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PageType {
    Struct(PageStruct),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PageStruct {
    name: String,
    fields: Vec<String>,
}

impl PageType {
    pub(crate) fn name(&self) -> &str {
        match self {
            PageType::Struct(page_struct) => &page_struct.name,
        }
    }

    pub(crate) fn type_name(&self) -> &str {
        match self {
            PageType::Struct(_) => "Struct",
        }
    }

    pub(crate) fn is_struct(&self) -> bool {
        match self {
            PageType::Struct(_) => true,
            _ => false,
        }
    }

    pub(crate) fn get_fields(&self) -> Option<&Vec<String>> {
        match self {
            PageType::Struct(page_struct) => Some(&page_struct.fields),
            _ => None,
        }
    }
}

impl<'a> From<&StructDeclaration<'_>> for PageType {
    fn from(struct_dec: &StructDeclaration) -> Self {
        let name = struct_dec.name.primary_name.into();

        let fields: Vec<String> = struct_dec
            .fields
            .iter()
            .map(|field| field.name.primary_name.into())
            .collect();

        PageType::Struct(PageStruct { name, fields })
    }
}
