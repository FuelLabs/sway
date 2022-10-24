use crate::{
    core::{
        session::Session,
        token::{SymbolKind, Token},
    },
    utils::common::get_range_from_span,
};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use sway_types::Span;
use tower_lsp::lsp_types::{
    Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensResult, Url,
};

// https://github.com/microsoft/vscode-extension-samples/blob/5ae1f7787122812dcc84e37427ca90af5ee09f14/semantic-tokens-sample/vscode.proposed.d.ts#L71
pub fn semantic_tokens_full(session: Arc<Session>, url: &Url) -> Option<SemanticTokensResult> {
    let tokens = session.tokens_for_file(url);

    // The tokens need sorting by their span so each token is sequential
    // If this step isn't done, then the bit offsets used for the lsp_types::SemanticToken are incorrect.
    let mut tokens_sorted: Vec<_> = tokens
        .iter()
        .map(|item| {
            let ((_, span), token) = item.pair();
            (span.clone(), token.clone())
        })
        .collect();

    tokens_sorted.sort_by(|(a_span, _), (b_span, _)| {
        let a = (a_span.start(), a_span.end());
        let b = (b_span.start(), b_span.end());
        a.cmp(&b)
    });

    let semantic_tokens = semantic_tokens(&tokens_sorted);

    Some(semantic_tokens.into())
}

//-------------------------------
/// Tokens are encoded relative to each other.
///
/// This is taken from rust-analyzer which is also a direct port of <https://github.com/microsoft/vscode-languageserver-node/blob/f425af9de46a0187adb78ec8a46b9b2ce80c5412/server/src/sematicTokens.proposed.ts#L45>
pub(crate) struct SemanticTokensBuilder {
    id:        String,
    prev_line: u32,
    prev_char: u32,
    data:      Vec<SemanticToken>,
}

impl SemanticTokensBuilder {
    pub(crate) fn new(id: String) -> Self {
        SemanticTokensBuilder {
            id,
            prev_line: 0,
            prev_char: 0,
            data: Default::default(),
        }
    }

    /// Push a new token onto the builder
    pub(crate) fn push(&mut self, range: Range, token_index: u32, modifier_bitset: u32) {
        let mut push_line = range.start.line as u32;
        let mut push_char = range.start.character as u32;

        if !self.data.is_empty() {
            push_line -= self.prev_line;
            if push_line == 0 {
                push_char -= self.prev_char;
            }
        }

        // A token cannot be multiline
        let token_len = range.end.character - range.start.character;

        let token = SemanticToken {
            delta_line:             push_line,
            delta_start:            push_char,
            length:                 token_len as u32,
            token_type:             token_index,
            token_modifiers_bitset: modifier_bitset,
        };

        self.data.push(token);

        self.prev_line = range.start.line as u32;
        self.prev_char = range.start.character as u32;
    }

    pub(crate) fn build(self) -> SemanticTokens {
        SemanticTokens {
            result_id: Some(self.id),
            data:      self.data,
        }
    }
}

pub(crate) fn semantic_tokens(tokens_sorted: &[(Span, Token)]) -> SemanticTokens {
    static TOKEN_RESULT_COUNTER: AtomicU32 = AtomicU32::new(1);
    let id = TOKEN_RESULT_COUNTER
        .fetch_add(1, Ordering::SeqCst)
        .to_string();
    let mut builder = SemanticTokensBuilder::new(id);

    for (span, token) in tokens_sorted.iter() {
        let ty = semantic_token_type(&token.kind);
        let token_index = type_index(ty);
        // TODO - improve with modifiers
        let modifier_bitset = 0;
        let range = get_range_from_span(span);

        builder.push(range, token_index, modifier_bitset);
    }
    builder.build()
}

fn semantic_token_type(kind: &SymbolKind) -> SemanticTokenType {
    match kind {
        SymbolKind::Field => SemanticTokenType::PROPERTY,
        SymbolKind::ValueParam => SemanticTokenType::PARAMETER,
        SymbolKind::Variable => SemanticTokenType::VARIABLE,
        SymbolKind::Function => SemanticTokenType::FUNCTION,
        SymbolKind::Const => SemanticTokenType::VARIABLE,
        SymbolKind::Struct => SemanticTokenType::STRUCT,
        SymbolKind::Enum => SemanticTokenType::ENUM,
        SymbolKind::Variant => SemanticTokenType::ENUM_MEMBER,
        SymbolKind::Trait => SemanticTokenType::INTERFACE,
        SymbolKind::TypeParameter => SemanticTokenType::TYPE_PARAMETER,
        SymbolKind::BoolLiteral => SemanticTokenType::new("boolean"),
        SymbolKind::ByteLiteral | SymbolKind::NumericLiteral => SemanticTokenType::NUMBER,
        SymbolKind::StringLiteral => SemanticTokenType::STRING,
        SymbolKind::BuiltinType => SemanticTokenType::new("builtinType"),
        SymbolKind::Module => SemanticTokenType::NAMESPACE,
        SymbolKind::Unknown => SemanticTokenType::new("generic"),
    }
}

pub(crate) fn type_index(ty: SemanticTokenType) -> u32 {
    SUPPORTED_TYPES.iter().position(|it| *it == ty).unwrap() as u32
}

pub(crate) const SUPPORTED_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::TYPE,
    SemanticTokenType::STRUCT,
    SemanticTokenType::CLASS,
    SemanticTokenType::INTERFACE,
    SemanticTokenType::ENUM,
    SemanticTokenType::ENUM_MEMBER,
    SemanticTokenType::TYPE_PARAMETER,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::METHOD,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::new("generic"),
    SemanticTokenType::new("boolean"),
    SemanticTokenType::new("builtinType"),
];

pub(crate) const SUPPORTED_MODIFIERS: &[SemanticTokenModifier] = &[
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
