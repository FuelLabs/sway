use crate::core::{
    session::Session,
    token::{Token, TokenType},
};
use lspower::lsp::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensResult, SemanticTokensServerCapabilities,
};
use std::sync::Arc;

// https://github.com/microsoft/vscode-extension-samples/blob/5ae1f7787122812dcc84e37427ca90af5ee09f14/semantic-tokens-sample/vscode.proposed.d.ts#L71
pub fn get_semantic_tokens_full(
    session: Arc<Session>,
    params: SemanticTokensParams,
) -> Option<SemanticTokensResult> {
    let url = params.text_document.uri;

    match session.get_tokens_from_file(&url) {
        Some(tokens) => {
            if tokens.is_empty() {
                return None;
            }

            let mut semantic_tokens: Vec<SemanticToken> =
                vec![create_semantic_token(&tokens[0], None)];

            for i in 1..tokens.len() {
                let semantic_token = create_semantic_token(&tokens[i], Some(&tokens[i - 1]));
                semantic_tokens.push(semantic_token);
            }

            Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: semantic_tokens,
            }))
        }
        _ => None,
    }
}

fn create_semantic_token(next_token: &Token, prev_token: Option<&Token>) -> SemanticToken {
    // TODO - improve with modifiers
    let token_modifiers_bitset = 0;
    let token_type = get_type(&next_token.token_type);
    let length = next_token.length;

    let next_token_start_char = next_token.range.start.character;

    let (delta_line, delta_start) = if let Some(prev_token) = prev_token {
        let delta_start = if next_token.line_start == prev_token.line_start {
            next_token_start_char - prev_token.range.start.character
        } else {
            next_token_start_char
        };
        (next_token.line_start - prev_token.line_start, delta_start)
    } else {
        (next_token.line_start, next_token_start_char)
    };

    SemanticToken {
        token_modifiers_bitset,
        token_type,
        length,
        delta_line,
        delta_start,
    }
}

fn get_type(token_type: &TokenType) -> u32 {
    match token_type {
        TokenType::Function => 1,
        TokenType::Library => 3,
        TokenType::Variable => 9,
        TokenType::Enum => 10,
        TokenType::Struct => 11,
        TokenType::Trait => 12,
        _ => 9,
    }
}

pub fn get_semantic_tokens() -> Option<SemanticTokensServerCapabilities> {
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
