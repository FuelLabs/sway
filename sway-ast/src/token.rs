use crate::priv_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Spacing {
    Joint,
    Alone,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Punct {
    pub span: Span,
    pub kind: PunctKind,
    pub spacing: Spacing,
}

impl Spanned for Punct {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct GenericGroup<T> {
    pub delimiter: Delimiter,
    pub token_stream: T,
    pub span: Span,
}

pub type Group = GenericGroup<TokenStream>;
pub type CommentedGroup = GenericGroup<CommentedTokenStream>;

impl<T> Spanned for GenericGroup<T> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum CommentKind {
    /// A newlined comment is a comment with a preceding newline before another token.
    ///
    /// # Examples
    ///
    /// ```sway
    /// pub fn main() -> bool {
    ///
    ///     // Newlined comment
    ///     true
    /// }
    /// ```
    Newlined,

    /// A trailing comment is a comment without a preceding newline before another token.
    ///
    /// # Examples
    ///
    /// ```sway
    /// var foo = 1; // Trailing comment
    /// ```
    Trailing,

    /// An inlined comment is a block comment nested between 2 tokens without a newline after it.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn some_function(baz: /* inlined comment */ u64) {}
    /// ```
    Inlined,

    /// A multiline comment is a block comment that may be nested between 2 tokens with 1 or more newlines within it.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn some_function(baz: /* multiline
    ///                          comment */ u64) {}
    /// ```
    Multilined,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Comment {
    pub span: Span,
    pub comment_kind: CommentKind,
}

impl Spanned for Comment {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum DocStyle {
    Outer,
    Inner,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct DocComment {
    pub span: Span,
    pub content_span: Span,
    pub doc_style: DocStyle,
}

impl Spanned for DocComment {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

/// Allows for generalizing over commented and uncommented token streams.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum GenericTokenTree<T> {
    Punct(Punct),
    Ident(Ident),
    Group(GenericGroup<T>),
    Literal(Literal),
    DocComment(DocComment),
}

pub type TokenTree = GenericTokenTree<TokenStream>;
pub type CommentedTree = GenericTokenTree<CommentedTokenStream>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum CommentedTokenTree {
    Comment(Comment),
    Tree(CommentedTree),
}

impl CommentedGroup {
    pub fn strip_comments(self) -> Group {
        Group {
            delimiter: self.delimiter,
            token_stream: self.token_stream.strip_comments(),
            span: self.span,
        }
    }
}

impl<T> Spanned for GenericTokenTree<T> {
    fn span(&self) -> Span {
        match self {
            Self::Punct(punct) => punct.span(),
            Self::Ident(ident) => ident.span(),
            Self::Group(group) => group.span(),
            Self::Literal(literal) => literal.span(),
            Self::DocComment(doc_comment) => doc_comment.span(),
        }
    }
}

impl Spanned for CommentedTokenTree {
    fn span(&self) -> Span {
        match self {
            Self::Comment(cmt) => cmt.span(),
            Self::Tree(tt) => tt.span(),
        }
    }
}

impl<T> From<Punct> for GenericTokenTree<T> {
    fn from(punct: Punct) -> Self {
        Self::Punct(punct)
    }
}

impl<T> From<Ident> for GenericTokenTree<T> {
    fn from(ident: Ident) -> Self {
        Self::Ident(ident)
    }
}

impl<T> From<GenericGroup<T>> for GenericTokenTree<T> {
    fn from(group: GenericGroup<T>) -> Self {
        Self::Group(group)
    }
}

impl<T> From<Literal> for GenericTokenTree<T> {
    fn from(lit: Literal) -> Self {
        Self::Literal(lit)
    }
}

impl<T> From<DocComment> for GenericTokenTree<T> {
    fn from(doc_comment: DocComment) -> Self {
        Self::DocComment(doc_comment)
    }
}

impl From<Comment> for CommentedTokenTree {
    fn from(comment: Comment) -> Self {
        Self::Comment(comment)
    }
}

impl From<CommentedTree> for CommentedTokenTree {
    fn from(tree: CommentedTree) -> Self {
        Self::Tree(tree)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct TokenStream {
    token_trees: Vec<TokenTree>,
    full_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct CommentedTokenStream {
    pub token_trees: Vec<CommentedTokenTree>,
    pub full_span: Span,
}

#[extension_trait]
impl CharExt for char {
    fn as_open_delimiter(self) -> Option<Delimiter> {
        match self {
            '(' => Some(Delimiter::Parenthesis),
            '{' => Some(Delimiter::Brace),
            '[' => Some(Delimiter::Bracket),
            _ => None,
        }
    }

    fn as_close_delimiter(self) -> Option<Delimiter> {
        match self {
            ')' => Some(Delimiter::Parenthesis),
            '}' => Some(Delimiter::Brace),
            ']' => Some(Delimiter::Bracket),
            _ => None,
        }
    }

    fn as_punct_kind(self) -> Option<PunctKind> {
        match self {
            ';' => Some(PunctKind::Semicolon),
            ':' => Some(PunctKind::Colon),
            '/' => Some(PunctKind::ForwardSlash),
            ',' => Some(PunctKind::Comma),
            '*' => Some(PunctKind::Star),
            '+' => Some(PunctKind::Add),
            '-' => Some(PunctKind::Sub),
            '<' => Some(PunctKind::LessThan),
            '>' => Some(PunctKind::GreaterThan),
            '=' => Some(PunctKind::Equals),
            '.' => Some(PunctKind::Dot),
            '!' => Some(PunctKind::Bang),
            '%' => Some(PunctKind::Percent),
            '&' => Some(PunctKind::Ampersand),
            '^' => Some(PunctKind::Caret),
            '|' => Some(PunctKind::Pipe),
            '_' => Some(PunctKind::Underscore),
            '#' => Some(PunctKind::Sharp),
            _ => None,
        }
    }
}

impl TokenStream {
    pub fn token_trees(&self) -> &[TokenTree] {
        &self.token_trees
    }
}

impl Spanned for TokenStream {
    fn span(&self) -> Span {
        self.full_span.clone()
    }
}

impl CommentedTokenTree {
    pub fn strip_comments(self) -> Option<TokenTree> {
        let commented_tt = match self {
            Self::Comment(_) => return None,
            Self::Tree(commented_tt) => commented_tt,
        };
        let tt = match commented_tt {
            CommentedTree::Punct(punct) => punct.into(),
            CommentedTree::Ident(ident) => ident.into(),
            CommentedTree::Group(group) => group.strip_comments().into(),
            CommentedTree::Literal(lit) => lit.into(),
            CommentedTree::DocComment(doc_comment) => doc_comment.into(),
        };
        Some(tt)
    }
}

impl CommentedTokenStream {
    pub fn token_trees(&self) -> &[CommentedTokenTree] {
        &self.token_trees
    }

    pub fn strip_comments(self) -> TokenStream {
        let token_trees = self
            .token_trees
            .into_iter()
            .filter_map(|tree| tree.strip_comments())
            .collect();
        TokenStream {
            token_trees,
            full_span: self.full_span,
        }
    }
}

impl Spanned for CommentedTokenStream {
    fn span(&self) -> Span {
        self.full_span.clone()
    }
}
