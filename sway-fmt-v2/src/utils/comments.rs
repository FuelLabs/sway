use crate::fmt::{FormattedCode, FormatterError};
use ropey::Rope;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::Write,
    ops::Bound::{Excluded, Included},
    path::PathBuf,
    sync::Arc,
};
use sway_parse::{
    attribute::Annotated,
    brackets::{Parens, SquareBrackets},
    keywords::{
        AddToken, ColonToken, CommaToken, ForToken, ForwardSlashToken, RightArrowToken,
        SemicolonToken,
    },
    token::{lex_commented, Comment, CommentedTokenTree, CommentedTree},
    Braces, Module, Parse, TypeField,
};
use sway_types::{Ident, Span, Spanned};
/// Represents a span for the comments in a spesific file
/// A stripped down version of sway-types::src::Span
#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct ByteSpan {
    // The byte position in the string of the start of the span.
    pub start: usize,
    // The byte position in the string of the end of the span.
    pub end: usize,
}

impl ByteSpan {
    /// Takes `start` and `end` from `sway::types::Span` and constructs a `ByteSpan`
    pub fn from_span(span: Span) -> ByteSpan {
        ByteSpan {
            start: span.start(),
            end: span.end(),
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl Ord for ByteSpan {
    fn cmp(&self, other: &Self) -> Ordering {
        // If the starting position is the same encapsulatig span (i.e, wider one) should come
        // first
        match self.start.cmp(&other.start) {
            Ordering::Equal => other.end.cmp(&self.end),
            ord => ord,
        }
    }
}

impl PartialOrd for ByteSpan {
    fn partial_cmp(&self, other: &ByteSpan) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub type CommentMap = BTreeMap<ByteSpan, Comment>;

/// Get the CommentedTokenStream and collect the spans -> Comment mapping for the input source
/// code.
pub fn comment_map_from_src(input: Arc<str>) -> Result<CommentMap, FormatterError> {
    let mut comment_map = BTreeMap::new();

    // pass the input through lexer
    let commented_token_stream = lex_commented(&input, 0, input.len(), None)?;
    let tts = commented_token_stream.token_trees().iter();

    for comment in tts {
        collect_comments_from_token_stream(comment, &mut comment_map);
    }
    Ok(comment_map)
}

/// Collects `Comment`s from the token stream and insert it with its span to the `CommentMap`.
/// Handles both the standalone and in-block comments.
fn collect_comments_from_token_stream(
    commented_token_tree: &CommentedTokenTree,
    comment_map: &mut CommentMap,
) {
    match commented_token_tree {
        CommentedTokenTree::Comment(comment) => {
            let comment_span = ByteSpan {
                start: comment.span.start(),
                end: comment.span.end(),
            };
            comment_map.insert(comment_span, comment.clone());
        }
        CommentedTokenTree::Tree(CommentedTree::Group(group)) => {
            for item in group.token_stream.token_trees().iter() {
                collect_comments_from_token_stream(item, comment_map);
            }
        }
        _ => {}
    }
}

impl<T> CommentVisitor for Braces<T>
where
    T: CommentVisitor + Clone,
{
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        let mut opening_brace_span = ByteSpan::from_span(self.span());
        opening_brace_span.end = opening_brace_span.start + 1;
        // Add opening brace's ByteSpan
        collected_spans.push(opening_brace_span);
        // Add T's collected ByteSpan
        collected_spans.append(&mut self.clone().into_inner().collect_spans());
        let mut closing_brace_span = ByteSpan::from_span(self.span());
        closing_brace_span.start = closing_brace_span.end - 1;
        // Add closing brace's ByteSpan
        collected_spans.push(closing_brace_span);
        collected_spans
    }
}

impl<T> CommentVisitor for Parens<T>
where
    T: CommentVisitor + Clone,
{
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        let mut opening_paren_span = ByteSpan::from_span(self.span());
        opening_paren_span.end = opening_paren_span.start + 1;
        // Add opening paren's span
        collected_spans.push(opening_paren_span);
        // Add T's collected ByteSpan
        collected_spans.append(&mut self.clone().into_inner().collect_spans());
        let mut closing_paren_span = ByteSpan::from_span(self.span());
        closing_paren_span.start = closing_paren_span.end - 1;
        // Add closing paren's ByteSpan
        collected_spans.push(closing_paren_span);
        collected_spans
    }
}

impl<T> CommentVisitor for SquareBrackets<T>
where
    T: CommentVisitor + Clone,
{
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        let mut opening_bracket_span = ByteSpan::from_span(self.span());
        opening_bracket_span.end = opening_bracket_span.start + 1;
        // Add opening bracket's span
        collected_spans.push(opening_bracket_span);
        // Add T's collected ByteSpan
        collected_spans.append(&mut self.clone().into_inner().collect_spans());
        let mut closing_bracket_span = ByteSpan::from_span(self.span());
        closing_bracket_span.start = closing_bracket_span.end - 1;
        // Add closing bracket's ByteSpan
        collected_spans.push(closing_bracket_span);
        collected_spans
    }
}

impl<T, P> CommentVisitor for (T, P)
where
    T: CommentVisitor,
    P: CommentVisitor,
{
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.0.collect_spans();
        collected_spans.append(&mut self.1.collect_spans());
        collected_spans
    }
}
impl<T> CommentVisitor for Vec<T>
where
    T: CommentVisitor,
{
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        for t in self {
            collected_spans.append(&mut t.collect_spans());
        }
        collected_spans
    }
}
impl<T> CommentVisitor for Annotated<T>
where
    T: CommentVisitor + Parse,
{
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.attribute_list.collect_spans();
        collected_spans.append(&mut self.value.collect_spans());
        collected_spans
    }
}

impl CommentVisitor for Ident {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from_span(self.span())]
    }
}
impl CommentVisitor for CommaToken {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        vec![(ByteSpan::from_span(self.span()))]
    }
}

impl CommentVisitor for TypeField {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from_span(self.name.span())];
        collected_spans.push(ByteSpan::from_span(self.colon_token.span()));
        collected_spans.append(&mut self.ty.collect_spans());
        collected_spans
    }
}

impl CommentVisitor for AddToken {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from_span(self.span())]
    }
}

impl CommentVisitor for SemicolonToken {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from_span(self.span())]
    }
}

impl CommentVisitor for ColonToken {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from_span(self.span())]
    }
}

impl CommentVisitor for RightArrowToken {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from_span(self.span())]
    }
}

impl CommentVisitor for ForToken {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from_span(self.span())]
    }
}

impl CommentVisitor for ForwardSlashToken {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from_span(self.span())]
    }
}
/// Handles comments by first creating the CommentMap which is used for fast seaching comments.
/// Traverses items for finding a comment in unformatted input and placing it in correct place in formatted output.
pub fn handle_comments(
    unformatted_input: Arc<str>,
    formatted_input: Arc<str>,
    path: Option<Arc<PathBuf>>,
    formatted_code: &mut FormattedCode,
) -> Result<(), FormatterError> {
    // Collect Span -> Comment mapping from unformatted input
    let comment_map = comment_map_from_src(unformatted_input.clone())?;
    // Parse unformatted code so that we can get the spans of items in their original places.
    // This is required since we collected the spans in from unformatted source file.
    let unformatted_module = sway_parse::parse_file(unformatted_input.clone(), path.clone())?;
    // After the formatting items should be the same but their spans will be changed since we applied formatting to them.
    let formatted_module = sway_parse::parse_file(formatted_input, path)?;
    // Actually insert the comments
    add_comments(
        comment_map,
        &unformatted_module,
        &formatted_module,
        formatted_code,
        unformatted_input,
    )?;
    Ok(())
}
/// Adds the comments from comment_map to correct places in the formatted code. This requires us
/// both the unformatted and formatted code's items as they will have different spans for their
/// nodes. While traversing the unformatted items, `add_comments` searches for comments. If there is a comment found
/// places the comment.
fn add_comments(
    comment_map: CommentMap,
    unformatted_module: &Module,
    formatted_module: &Module,
    formatted_code: &mut FormattedCode,
    unformatted_code: Arc<str>,
) -> Result<(), FormatterError> {
    let unformatted_comment_spans = unformatted_module.collect_spans();
    let formatted_comment_spans = formatted_module.collect_spans();

    // Since we are adding comments into formatted code, in the next iteration the spans we find for the formatted code needs to be offsetted
    // as the total length of comments we added in previous iterations.
    let mut offset = 0;

    // We will definetly have a span in the collected span since for a source code to be parsed there should be some tokens present.
    let mut previous_unformatted_comment_span = unformatted_comment_spans
        .first()
        .ok_or(FormatterError::CommentError)?;
    let mut previous_formatted_comment_span = formatted_comment_spans
        .first()
        .ok_or(FormatterError::CommentError)?;
    for (unformatted_comment_span, formatted_comment_span) in unformatted_comment_spans
        .iter()
        .skip(1)
        .zip(formatted_comment_spans.iter().skip(1))
    {
        let comments_found = get_comments_between_spans(
            previous_unformatted_comment_span,
            unformatted_comment_span,
            &comment_map,
            &unformatted_code,
        );
        if !comments_found.is_empty() {
            offset += insert_after_span(
                previous_formatted_comment_span,
                comments_found,
                offset,
                formatted_code,
            )?;
        }
        previous_unformatted_comment_span = unformatted_comment_span;
        previous_formatted_comment_span = formatted_comment_span;
    }
    Ok(())
}

// A CommentWithContext is the Comment and the offset before it. The offset can be between the (from) item we searched for this comment or from the last comment inside range
type CommentWithContext = (Comment, String);

/// Returns a list of comments between given spans. For each comment returns the Context
/// Context of a comment is basically the offset (the characters between the last item/comment) to the current comment
fn get_comments_between_spans(
    from: &ByteSpan,
    to: &ByteSpan,
    comment_map: &CommentMap,
    unformatted_code: &Arc<str>,
) -> Vec<CommentWithContext> {
    let mut comments_with_context = Vec::new();
    for (index, comment_tuple) in comment_map
        .range((Included(from), Excluded(to)))
        .enumerate()
    {
        if comments_with_context.is_empty() {
            // This is the first comment in the current range the context should be collected between from's end and comment's beginning
            comments_with_context.push((
                comment_tuple.1.clone(),
                unformatted_code[from.end..comment_tuple.0.start].to_string(),
            ));
            println!("this");
        } else {
            // There is a comment before this one, so we should get the context starting from the last comment's end to the beginning of the current comment
            comments_with_context.push((
                comment_tuple.1.clone(),
                unformatted_code
                    [comments_with_context[index - 1].0.span.end()..comment_tuple.0.start]
                    .to_string(),
            ));
        }
    }
    println!(
        " found {:?} between {:?} and {:?} \n\n",
        comments_with_context, from, to
    );
    comments_with_context
}

/// Inserts after given span and returns the offset. While inserting comments this also inserts Context of the comments so that the alignment whitespaces/newlines are intact
fn insert_after_span(
    from: &ByteSpan,
    comments_to_insert: Vec<CommentWithContext>,
    offset: usize,
    formatted_code: &mut FormattedCode,
) -> Result<usize, FormatterError> {
    let mut src_rope = Rope::from_str(formatted_code);
    // prepare the comment str
    let mut comment_str = format!(
        "{}{}",
        comments_to_insert[0].1,
        format_comment(&comments_to_insert[0].0)
    );
    for comment in comments_to_insert.iter().skip(1) {
        write!(comment_str, "{}{}", comment.1, &format_comment(&comment.0))?;
    }
    src_rope.insert(from.end + offset, &comment_str);
    formatted_code.clear();
    formatted_code.push_str(&src_rope.to_string());
    Ok(comment_str.len())
}

/// Applies formatting to the comment.
/// Currently just checks if it is a multiline comment, if that is the case it adds a trailing `/` to the end.
fn format_comment(comment: &Comment) -> String {
    if comment.span().str().starts_with("/*") {
        format!("{}/", comment.span().str())
    } else {
        comment.span().str()
    }
}
/// While searching for a comment we need the possible places a comment can be placed in a structure
/// `collect_spans` collects all field's spans so that we can check in between them.
pub trait CommentVisitor {
    fn collect_spans(&self) -> Vec<ByteSpan>;
}

#[cfg(test)]
mod tests {
    use super::{comment_map_from_src, ByteSpan};
    use std::{ops::Bound::Included, sync::Arc};

    #[test]
    fn test_comment_span_ordering() {
        let first_span = ByteSpan { start: 2, end: 6 };
        let second_span = ByteSpan { start: 2, end: 4 };
        let third_span = ByteSpan { start: 4, end: 7 };

        let mut vec = vec![second_span.clone(), third_span.clone(), first_span.clone()];
        vec.sort();

        assert_eq!(vec[0], first_span);
        assert_eq!(vec[1], second_span);
        assert_eq!(vec[2], third_span);
    }

    #[test]
    fn test_comment_span_map_standalone_comment() {
        let input = r#"
        // Single-line comment.
        let var = 256; // This is a comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let map = comment_map_from_src(Arc::from(input)).unwrap();
        assert!(!map.is_empty());
        let range_start_span = ByteSpan { start: 0, end: 32 };
        let range_end_span = ByteSpan { start: 33, end: 34 };
        let found_comment = map
            .range((Included(range_start_span), Included(range_end_span)))
            .last()
            .unwrap();
        assert_eq!(found_comment.1.span.as_str(), "// Single-line comment.");
    }
    #[test]
    fn test_comment_span_map_standalone_next_to_item() {
        let input = r#"
        // Single-line comment.
        let var = 256; // This is a comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let map = comment_map_from_src(Arc::from(input)).unwrap();
        assert!(!map.is_empty());
        let range_start_span = ByteSpan { start: 40, end: 54 };
        let range_end_span = ByteSpan {
            start: 100,
            end: 115,
        };
        let found_comment = map
            .range((Included(range_start_span), Included(range_end_span)))
            .last()
            .unwrap();
        assert_eq!(found_comment.1.span.as_str(), "// This is a comment.");
    }
    #[test]
    fn test_comment_span_map_standalone_inside_block() {
        let input = r#"
        // Single-line comment.
        let var = 256; // This is a comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let map = comment_map_from_src(Arc::from(input)).unwrap();
        assert!(!map.is_empty());
        let range_start_span = ByteSpan {
            start: 110,
            end: 116,
        };
        let range_end_span = ByteSpan {
            start: 200,
            end: 201,
        };
        let found_comment = map
            .range((Included(range_start_span), Included(range_end_span)))
            .last()
            .unwrap();
        assert_eq!(
            found_comment.1.span.as_str(),
            "/* multi-\n             * line-\n             * comment *"
        );
    }
}
