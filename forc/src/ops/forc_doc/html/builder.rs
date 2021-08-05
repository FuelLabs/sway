use std::fs::File;

use core_lang::StructDeclaration;
use maud::{html, Markup};

use crate::ops::forc_doc::html::common::{footer, header};

pub fn build_struct(struct_dec: &StructDeclaration) {
    let name = struct_dec.name.primary_name;
    let fields = &struct_dec.fields;

    let markup_file: Markup = html! {
        (header(name))
        h1 {
            (name)
        }

        ol {
            @for field in fields {
                li { (field.name.primary_name) }
            }
        }
        (footer())
    };

    let file_name = format!("./{}.html", name);
    let _ = File::create(&file_name).unwrap();
    std::fs::write(&file_name, markup_file.into_string()).unwrap();
}
