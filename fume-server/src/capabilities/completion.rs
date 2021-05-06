use crate::core::session::Session;
use lspower::lsp::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
};
use parser::{HllParser, Rule};
use pest::iterators::Pairs;
use pest::Parser;

use std::sync::Arc;

pub fn get_completion(session: Arc<Session>, params: CompletionParams) -> Option<CompletionResponse> {
    let uri = params.text_document_position.text_document.uri;

    match session.get_document_text(&uri) {
        Ok(document) => match HllParser::parse(Rule::program, &document) {
            Ok(rules) => {
                let completion_items = get_completion_items(rules);
                Some(CompletionResponse::Array(completion_items))
            }
            Err(e) => None,
        },
        Err(e) => None,
    }
}

fn get_completion_items(pairs: Pairs<Rule>) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    for rule in pairs.flatten() {
        match rule.as_rule() {
            Rule::trait_name => completion_items.push(create_completion_item(
                rule.as_str(),
                CompletionItemKind::Interface,
            )),
            Rule::library_name => completion_items.push(create_completion_item(
                rule.as_str(),
                CompletionItemKind::Class,
            )),
            Rule::fn_decl_name => completion_items.push(create_completion_item(
                rule.as_str(),
                CompletionItemKind::Function,
            )),
            Rule::enum_name => completion_items.push(create_completion_item(
                rule.as_str(),
                CompletionItemKind::Enum,
            )),
            Rule::var_name => completion_items.push(create_completion_item(
                rule.as_str(),
                CompletionItemKind::Variable,
            )),
            Rule::contract => completion_items.push(create_completion_item(
                rule.as_str(),
                CompletionItemKind::Struct,
            )),
            _ => {}
        }
    }
    completion_items
}

fn create_completion_item(name: &str, kind: CompletionItemKind) -> CompletionItem {
    CompletionItem {
        label: name.to_string(),
        kind: Some(kind),
        ..Default::default()
    }
}
