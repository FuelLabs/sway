use crate::core::{
    token::{SymbolKind, Token, TokenIdent},
    token_map::TokenMap,
};
use dashmap::mapref::multiple::RefMulti;
use lsp_types::{
    Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensRangeResult, SemanticTokensResult, Url,
};
use std::sync::atomic::{AtomicU32, Ordering};

// https://github.com/microsoft/vscode-extension-samples/blob/5ae1f7787122812dcc84e37427ca90af5ee09f14/semantic-tokens-sample/vscode.proposed.d.ts#L71

/// Get the semantic tokens for the entire file.
pub fn semantic_tokens_full(token_map: &TokenMap, url: &Url) -> Option<SemanticTokensResult> {
    let tokens: Vec<_> = token_map.tokens_for_file(url).collect();
    let sorted_tokens_refs = sort_tokens(&tokens);
    Some(semantic_tokens(&sorted_tokens_refs[..]).into())
}

/// Get the semantic tokens within a range.
pub fn semantic_tokens_range(
    token_map: &TokenMap,
    url: &Url,
    range: &Range,
) -> Option<SemanticTokensRangeResult> {
    let _p = tracing::trace_span!("semantic_tokens_range").entered();
    let tokens: Vec<_> = token_map
        .tokens_for_file(url)
        .filter(|item| {
            // make sure the token_ident range is within the range that was passed in
            let token_range = item.key().range;
            token_range.start >= range.start && token_range.end <= range.end
        })
        .collect();
    let sorted_tokens_refs = sort_tokens(&tokens);
    Some(semantic_tokens(&sorted_tokens_refs[..]).into())
}

pub fn semantic_tokens(tokens_sorted: &[&RefMulti<TokenIdent, Token>]) -> SemanticTokens {
    static TOKEN_RESULT_COUNTER: AtomicU32 = AtomicU32::new(1);
    let id = TOKEN_RESULT_COUNTER
        .fetch_add(1, Ordering::SeqCst)
        .to_string();
    let mut builder = SemanticTokensBuilder::new(id);

    for entry in tokens_sorted {
        let (ident, token) = entry.pair();
        let ty = semantic_token_type(&token.kind);
        if let Some(token_index) = type_index(&ty) {
            // TODO - improve with modifiers
            let modifier_bitset = 0;
            builder.push(ident.range, token_index, modifier_bitset);
        } else {
            tracing::error!("Unsupported token type: {:?} for token: {:#?}", ty, token);
        }
    }
    builder.build()
}

/// Sort tokens by their span so each token is sequential.
///
/// If this step isn't done, then the bit offsets used for the `lsp_types::SemanticToken` are incorrect.
fn sort_tokens<'a>(
    tokens: &'a [RefMulti<'a, TokenIdent, Token>],
) -> Vec<&'a RefMulti<'a, TokenIdent, Token>> {
    let mut refs: Vec<_> = tokens.iter().collect();
    // Sort the vector of references based on the spans of the tokens
    refs.sort_by(|a, b| {
        let a_span = a.key().range;
        let b_span = b.key().range;
        (a_span.start, a_span.end).cmp(&(b_span.start, b_span.end))
    });
    refs
}
//-------------------------------
/// Tokens are encoded relative to each other.
///
/// This is taken from rust-analyzer which is also a direct port of <https://github.com/microsoft/vscode-languageserver-node/blob/f425af9de46a0187adb78ec8a46b9b2ce80c5412/server/src/sematicTokens.proposed.ts#L45>
struct SemanticTokensBuilder {
    id: String,
    prev_line: u32,
    prev_char: u32,
    data: Vec<SemanticToken>,
}

impl SemanticTokensBuilder {
    pub fn new(id: String) -> Self {
        SemanticTokensBuilder {
            id,
            prev_line: 0,
            prev_char: 0,
            data: Vec::default(),
        }
    }

    /// Push a new token onto the builder
    pub fn push(&mut self, range: Range, token_index: u32, modifier_bitset: u32) {
        let mut push_line = range.start.line;
        let mut push_char = range.start.character;

        if !self.data.is_empty() {
            push_line -= self.prev_line;
            if push_line == 0 {
                push_char -= self.prev_char;
            }
        }

        // A token cannot be multiline
        let token_len = range.end.character - range.start.character;

        let token = SemanticToken {
            delta_line: push_line,
            delta_start: push_char,
            length: token_len,
            token_type: token_index,
            token_modifiers_bitset: modifier_bitset,
        };

        self.data.push(token);

        self.prev_line = range.start.line;
        self.prev_char = range.start.character;
    }

    pub fn build(self) -> SemanticTokens {
        SemanticTokens {
            result_id: Some(self.id),
            data: self.data,
        }
    }
}

pub(crate) const SUPPORTED_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::NAMESPACE,
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
    SemanticTokenType::new("keyword"),
    SemanticTokenType::new("builtinType"),
    SemanticTokenType::new("deriveHelper"),
    SemanticTokenType::new("selfKeyword"),
    SemanticTokenType::new("selfTypeKeyword"),
    SemanticTokenType::new("typeAlias"),
    SemanticTokenType::new("traitType"),
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

/// Get the semantic token type from the symbol kind.
fn semantic_token_type(kind: &SymbolKind) -> SemanticTokenType {
    match kind {
        SymbolKind::Field => SemanticTokenType::PROPERTY,
        SymbolKind::ValueParam => SemanticTokenType::PARAMETER,
        SymbolKind::Variable | SymbolKind::Const => SemanticTokenType::VARIABLE,
        SymbolKind::Function | SymbolKind::Intrinsic => SemanticTokenType::FUNCTION,
        SymbolKind::Struct => SemanticTokenType::STRUCT,
        SymbolKind::Enum => SemanticTokenType::ENUM,
        SymbolKind::Variant => SemanticTokenType::ENUM_MEMBER,
        SymbolKind::Trait => SemanticTokenType::INTERFACE,
        SymbolKind::TypeParameter => SemanticTokenType::TYPE_PARAMETER,
        SymbolKind::Module => SemanticTokenType::NAMESPACE,
        SymbolKind::StringLiteral => SemanticTokenType::STRING,
        SymbolKind::ByteLiteral | SymbolKind::NumericLiteral => SemanticTokenType::NUMBER,
        SymbolKind::BoolLiteral => SemanticTokenType::new("boolean"),
        SymbolKind::TypeAlias => SemanticTokenType::new("typeAlias"),
        SymbolKind::TraitType => SemanticTokenType::new("traitType"),
        SymbolKind::Keyword | SymbolKind::ProgramTypeKeyword => SemanticTokenType::new("keyword"),
        SymbolKind::Unknown => SemanticTokenType::new("generic"),
        SymbolKind::BuiltinType => SemanticTokenType::new("builtinType"),
        SymbolKind::DeriveHelper => SemanticTokenType::new("deriveHelper"),
        SymbolKind::SelfKeyword => SemanticTokenType::new("selfKeyword"),
        SymbolKind::SelfTypeKeyword => SemanticTokenType::new("selfTypeKeyword"),
    }
}

fn type_index(ty: &SemanticTokenType) -> Option<u32> {
    SUPPORTED_TYPES
        .iter()
        .position(|it| it == ty)
        .map(|x| x as u32)
}
