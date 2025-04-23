mod attribute;
mod brackets;
mod expr;
mod generics;
mod item;
mod keywords;
mod literal;
mod module;
mod parse;
mod parser;
mod path;
mod pattern;
mod priv_prelude;
mod punctuated;
mod submodule;
#[cfg(test)]
mod test_utils;
mod token;
mod ty;
mod where_clause;

use crate::priv_prelude::*;
pub use crate::{
    keywords::RESERVED_KEYWORDS,
    parse::Parse,
    parser::Parser,
    token::{lex, lex_commented, parse_int_suffix},
};

use sway_ast::{
    attribute::Annotated,
    token::{DocComment, DocStyle},
    Module, ModuleKind,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{span::Source, SourceId};

pub fn parse_file(
    handler: &Handler,
    src: Source,
    source_id: Option<SourceId>,
) -> Result<Annotated<Module>, ErrorEmitted> {
    let end = src.text.len();
    let ts = lex(handler, src, 0, end, source_id)?;
    let (m, _) = Parser::new(handler, &ts).parse_to_end()?;
    Ok(m)
}

pub fn parse_module_kind(
    handler: &Handler,
    src: Source,
    source_id: Option<SourceId>,
) -> Result<ModuleKind, ErrorEmitted> {
    let end = src.text.len();
    let ts = lex(handler, src, 0, end, source_id)?;
    let mut parser = Parser::new(handler, &ts);
    while let Some(DocComment {
        doc_style: DocStyle::Inner,
        ..
    }) = parser.peek()
    {
        parser.parse::<DocComment>()?;
    }
    parser.parse()
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn parse_invalid() {
        // just make sure these do not panic
        let _res = parse_file(&Handler::default(), "script; fn main(256߄".into(), None);
        let _res = parse_file(
            &Handler::default(),
            "script;
            fn karr() {
                let c: f828 =  0x00000000000000000000000vncifxp;
            abi Zezybt {
                #[mfzbezc, storage(r#
            true }
            }
            cug"
            .into(),
            None,
        );
        let _res = parse_file(
            &Handler::default(),
            "script;

            stdfn main() {
                let a: b256 =  0x000>0000000scri s = \"flibrary I24;

            use std::primitives::*;
            use std::assert::assert;

            ///\u{7eb}"
                .into(),
            None,
        );
        let _res = parse_file(&Handler::default(), "script; \"\u{7eb}\u{7eb}".into(), None);
    }
}
