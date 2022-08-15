use crate::core::{
    session::Session,
    token::{AstToken, TokenMap},
};
use crate::utils::common::get_range_from_span;
use sway_core::{Declaration, ExpressionKind, Literal};
use sway_types::Span;
use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensResult,
    SemanticTokensServerCapabilities, Url,
};

// https://github.com/microsoft/vscode-extension-samples/blob/5ae1f7787122812dcc84e37427ca90af5ee09f14/semantic-tokens-sample/vscode.proposed.d.ts#L71
pub fn semantic_tokens_full(session: &Session, url: &Url) -> Option<SemanticTokensResult> {
    match session.semantic_tokens(url) {
        Some(semantic_tokens) => {
            if semantic_tokens.is_empty() {
                return None;
            }

            Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: semantic_tokens,
            }))
        }
        _ => None,
    }
}

pub fn to_semantic_tokens(token_map: &TokenMap) -> Vec<SemanticToken> {
    let mut semantic_tokens: Vec<SemanticToken> = Vec::new();

    let mut prev_token_span = None;
    for item in token_map.iter() {
        let ((_, span), token) = item.pair();
        let token_type_idx = type_idx(&token.parsed);
        let semantic_token = semantic_token(token_type_idx, span, prev_token_span);

        semantic_tokens.push(semantic_token);
        prev_token_span = Some(span.clone());
    }

    semantic_tokens
}

fn semantic_token(
    token_type_idx: u32,
    next_token_span: &Span,
    prev_token_span: Option<Span>,
) -> SemanticToken {
    let next_token_range = get_range_from_span(next_token_span);
    let next_token_line_start = next_token_range.start.line;

    // TODO - improve with modifiers
    let token_modifiers_bitset = 0;
    let length = next_token_range.end.character - next_token_range.start.character;

    let next_token_start_char = next_token_range.start.character;

    let (delta_line, delta_start) = if let Some(prev_token_span) = prev_token_span {
        let prev_token_range = get_range_from_span(&prev_token_span);
        let prev_token_line_start = prev_token_range.start.line;
        let delta_start = if next_token_line_start == prev_token_line_start {
            next_token_start_char - prev_token_range.start.character
        } else {
            next_token_start_char
        };
        (next_token_line_start - prev_token_line_start, delta_start)
    } else {
        (next_token_line_start, next_token_start_char)
    };

    SemanticToken {
        token_modifiers_bitset,
        token_type: token_type_idx,
        length,
        delta_line,
        delta_start,
    }
}

/// these values should reflect indexes in `token_types`
#[repr(u32)]
enum TokenTypeIndex {
    Function = 1,
    Parameter = 5,
    String = 6,
    Variable = 9,
    Enum = 10,
    Struct = 11,
    Interface = 12,
}

fn type_idx(ast_token: &AstToken) -> u32 {
    match ast_token {
        AstToken::Declaration(dec) => {
            match dec {
                Declaration::VariableDeclaration(_) => TokenTypeIndex::Variable as u32,
                Declaration::FunctionDeclaration(_) => TokenTypeIndex::Function as u32,
                Declaration::TraitDeclaration(_) | Declaration::ImplTrait { .. } => {
                    TokenTypeIndex::Interface as u32
                }
                Declaration::StructDeclaration(_) => TokenTypeIndex::Struct as u32,
                Declaration::EnumDeclaration(_) => TokenTypeIndex::Enum as u32,
                // currently we return `variable` type as default
                _ => TokenTypeIndex::Variable as u32,
            }
        }
        AstToken::Expression(exp) => {
            match &exp.kind {
                ExpressionKind::Literal(Literal::String(_)) => TokenTypeIndex::String as u32,
                ExpressionKind::FunctionApplication(_) => TokenTypeIndex::Function as u32,
                ExpressionKind::Variable(_) => TokenTypeIndex::Variable as u32,
                ExpressionKind::Struct(_) => TokenTypeIndex::Struct as u32,
                // currently we return `variable` type as default
                _ => TokenTypeIndex::Variable as u32,
            }
        }
        AstToken::FunctionDeclaration(_) => TokenTypeIndex::Function as u32,
        AstToken::FunctionParameter(_) => TokenTypeIndex::Parameter as u32,
        AstToken::TraitFn(_) => TokenTypeIndex::Function as u32,
        // currently we return `variable` type as default
        _ => TokenTypeIndex::Variable as u32,
    }
}

pub fn semantic_tokens() -> Option<SemanticTokensServerCapabilities> {
    let token_types = vec![
        SemanticTokenType::CLASS,          // 0
        SemanticTokenType::FUNCTION,       // 1
        SemanticTokenType::KEYWORD,        // 2
        SemanticTokenType::NAMESPACE,      // 3
        SemanticTokenType::OPERATOR,       // 4
        SemanticTokenType::PARAMETER,      // 5
        SemanticTokenType::STRING,         // 6
        SemanticTokenType::TYPE,           // 7
        SemanticTokenType::TYPE_PARAMETER, // 8
        SemanticTokenType::VARIABLE,       // 9
        SemanticTokenType::ENUM,           // 10
        SemanticTokenType::STRUCT,         // 11
        SemanticTokenType::INTERFACE,      // 12
    ];

    let token_modifiers: Vec<SemanticTokenModifier> = vec![
        // declaration of symbols
        SemanticTokenModifier::DECLARATION,
        // definition of symbols as in header files
        SemanticTokenModifier::DEFINITION,
        SemanticTokenModifier::READONLY,
        SemanticTokenModifier::STATIC,
        // for variable references where the variable is assigned to
        SemanticTokenModifier::MODIFICATION,
        SemanticTokenModifier::DOCUMENTATION,
        // for symbols that are part of stdlib
        SemanticTokenModifier::DEFAULT_LIBRARY,
    ];

    let legend = SemanticTokensLegend {
        token_types,
        token_modifiers,
    };

    let options = SemanticTokensOptions {
        legend,
        range: None,
        full: Some(SemanticTokensFullOptions::Bool(true)),
        ..Default::default()
    };

    Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
        options,
    ))
}
