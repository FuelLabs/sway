use sway_types::Spanned;

use crate::{error::*, Ident};

/// Find the first index in the string which separates a lowercase character from an uppercase
/// character. Used for splitting words in a CamelCase style identifier.
fn find_camel_case_word_boundary(ident: &str) -> Option<usize> {
    let mut previous_char_was_lowercase = false;
    for (index, c) in ident.char_indices() {
        if c.is_uppercase() && previous_char_was_lowercase {
            return Some(index);
        }
        previous_char_was_lowercase = c.is_lowercase();
    }
    None
}

/// Split a CamelCase style identifier into words.
fn camel_case_split_words(ident: &str) -> impl Iterator<Item = &str> {
    let mut ident = ident;
    std::iter::from_fn(move || {
        if ident.is_empty() {
            return None;
        }
        let index = find_camel_case_word_boundary(ident).unwrap_or(ident.len());
        let word = &ident[..index];
        ident = &ident[index..];
        Some(word)
    })
}

/// Split an identifier of unknown style into words.
fn split_words(ident: &str) -> impl Iterator<Item = &str> {
    ident.split('_').flat_map(camel_case_split_words)
}

/// Detect whether an identifier is written in snake_case.
pub fn is_snake_case(ident: &Ident) -> CompileResult<()> {
    let span = ident.span();
    let name = ident.as_str();
    let trimmed = name.trim_start_matches('_');
    if trimmed.contains("__") || trimmed.contains(char::is_uppercase) {
        return ok(
            (),
            vec![CompileWarning {
                span,
                warning_content: Warning::NonSnakeCaseFunctionName {
                    name: ident.clone(),
                },
            }],
            vec![],
        );
    }
    ok((), vec![], vec![])
}

/// Detect whether an identifier is written in SCREAMING_SNAKE_CASE.
pub fn is_screaming_snake_case(ident: &Ident) -> CompileResult<()> {
    let trimmed = ident.as_str().trim_start_matches('_');
    let span = ident.span();
    if trimmed.contains("__") || trimmed.contains(char::is_lowercase) {
        return ok(
            (),
            vec![CompileWarning {
                span,
                warning_content: Warning::NonScreamingSnakeCaseConstName {
                    name: ident.clone(),
                },
            }],
            vec![],
        );
    }
    ok((), vec![], vec![])
}

/// Detect whether an identifier is written in UpperCamelCase.
pub fn is_upper_camel_case(ident: &Ident) -> CompileResult<()> {
    let trimmed = ident.as_str().trim_start_matches('_');
    let span = ident.span();
    if trimmed.contains('_') || trimmed.starts_with(char::is_lowercase) {
        return ok(
            (),
            vec![CompileWarning {
                span,
                warning_content: Warning::NonScreamingSnakeCaseConstName {
                    name: ident.clone(),
                },
            }],
            vec![],
        );
    }
    ok((), vec![], vec![])
}

/// Convert an identifier into snake_case. This is a best-guess at what the identifier would look
/// like if it were expressed in the correct style.
pub fn to_snake_case(ident: &str) -> String {
    let mut ret = String::with_capacity(ident.len());

    let (leading_underscores, trimmed) =
        ident.split_at(ident.find(|c| c != '_').unwrap_or(ident.len()));
    ret.push_str(leading_underscores);
    let mut words = split_words(trimmed);
    if let Some(word) = words.next() {
        ret.extend(word.chars().flat_map(char::to_lowercase));
        for word in words {
            ret.push('_');
            ret.extend(word.chars().flat_map(char::to_lowercase));
        }
    }
    ret
}

/// Convert an identifier into SCREAMING_SNAKE_CASE. This is a best-guess at what the identifier
/// would look like if it were expressed in the correct style.
pub fn to_screaming_snake_case(ident: &str) -> String {
    let mut ret = String::with_capacity(ident.len());

    let (leading_underscores, trimmed) =
        ident.split_at(ident.find(|c| c != '_').unwrap_or(ident.len()));
    ret.push_str(leading_underscores);
    let mut words = split_words(trimmed);
    if let Some(word) = words.next() {
        ret.extend(word.chars().flat_map(char::to_uppercase));
        for word in words {
            ret.push('_');
            ret.extend(word.chars().flat_map(char::to_uppercase));
        }
    }
    ret
}

/// Convert an identifier into UpperCamelCase. This is a best-guess at what the identifier would
/// look like if it were expressed in the correct style.
pub fn to_upper_camel_case(ident: &str) -> String {
    let mut ret = String::with_capacity(ident.len());

    let (leading_underscores, trimmed) =
        ident.split_at(ident.find(|c| c != '_').unwrap_or(ident.len()));
    ret.push_str(leading_underscores);
    for word in split_words(trimmed) {
        let mut chars = word.chars();
        if let Some(c) = chars.next() {
            ret.extend(c.to_uppercase());
            ret.extend(chars.flat_map(char::to_lowercase));
        }
    }
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use sway_types::Span;

    #[test]
    fn detect_styles() {
        let snake_case_idents = [
            "hello",
            "__hello",
            "blah32",
            "some_words_here",
            "___some_words_here",
        ];
        let screaming_snake_case_idents = ["SOME_WORDS_HERE", "___SOME_WORDS_HERE"];
        let upper_camel_case_idents = [
            "Hello",
            "__Hello",
            "Blah32",
            "SomeWordsHere",
            "___SomeWordsHere",
        ];
        let screaming_snake_case_or_upper_camel_case_idents = ["HELLO", "__HELLO", "BLAH32"];
        let styleless_idents = ["Mix_Of_Things", "__Mix_Of_Things", "FooBar_123"];
        let snake_case_idents = snake_case_idents
            .into_iter()
            .map(|x| Ident::new_with_override(x, Span::dummy()));
        let screaming_snake_case_idents = screaming_snake_case_idents
            .into_iter()
            .map(|x| Ident::new_with_override(x, Span::dummy()));
        let styleless_idents = styleless_idents
            .into_iter()
            .map(|x| Ident::new_with_override(x, Span::dummy()));
        let screaming_snake_case_or_upper_camel_case_idents =
            screaming_snake_case_or_upper_camel_case_idents
                .into_iter()
                .map(|x| Ident::new_with_override(x, Span::dummy()));
        let upper_camel_case_idents = upper_camel_case_idents
            .into_iter()
            .map(|x| Ident::new_with_override(x, Span::dummy()));
        for ref ident in snake_case_idents {
            assert!(is_snake_case(ident).is_ok_no_warn());
            assert!(!is_screaming_snake_case(ident).is_ok_no_warn());
            assert!(!is_upper_camel_case(ident).is_ok_no_warn());
        }
        for ref ident in screaming_snake_case_idents {
            assert!(!is_snake_case(ident).is_ok_no_warn());
            assert!(is_screaming_snake_case(ident).is_ok_no_warn());
            assert!(!is_upper_camel_case(ident).is_ok_no_warn());
        }
        for ref ident in upper_camel_case_idents {
            assert!(!is_snake_case(ident).is_ok_no_warn());
            assert!(!is_screaming_snake_case(ident).is_ok_no_warn());
            assert!(is_upper_camel_case(ident).is_ok_no_warn());
        }
        for ref ident in screaming_snake_case_or_upper_camel_case_idents {
            assert!(!is_snake_case(ident).is_ok_no_warn());
            assert!(is_screaming_snake_case(ident).is_ok_no_warn());
            assert!(is_upper_camel_case(ident).is_ok_no_warn());
        }
        for ref ident in styleless_idents {
            assert!(!is_snake_case(ident).is_ok_no_warn());
            assert!(!is_screaming_snake_case(ident).is_ok_no_warn());
            assert!(!is_upper_camel_case(ident).is_ok_no_warn());
        }
    }

    #[test]
    fn convert_to_snake_case() {
        assert_eq!("hello", to_snake_case("HELLO"));
        assert_eq!("___hello", to_snake_case("___HELLO"));
        assert_eq!("blah32", to_snake_case("BLAH32"));
        assert_eq!("some_words_here", to_snake_case("SOME_WORDS_HERE"));
        assert_eq!("___some_words_here", to_snake_case("___SOME_WORDS_HERE"));
        assert_eq!("hello", to_snake_case("Hello"));
        assert_eq!("___hello", to_snake_case("___Hello"));
        assert_eq!("blah32", to_snake_case("Blah32"));
        assert_eq!("some_words_here", to_snake_case("SomeWordsHere"));
        assert_eq!("___some_words_here", to_snake_case("___SomeWordsHere"));
        assert_eq!("some_words_here", to_snake_case("someWordsHere"));
        assert_eq!("___some_words_here", to_snake_case("___someWordsHere"));
        assert_eq!("mix_of_things", to_snake_case("Mix_Of_Things"));
        assert_eq!("__mix_of_things", to_snake_case("__Mix_Of_Things"));
        assert_eq!("foo_bar_123", to_snake_case("FooBar_123"));
    }

    #[test]
    fn convert_to_screaming_snake_case() {
        assert_eq!("HELLO", to_screaming_snake_case("hello"));
        assert_eq!("___HELLO", to_screaming_snake_case("___hello"));
        assert_eq!("BLAH32", to_screaming_snake_case("blah32"));
        assert_eq!(
            "SOME_WORDS_HERE",
            to_screaming_snake_case("some_words_here")
        );
        assert_eq!(
            "___SOME_WORDS_HERE",
            to_screaming_snake_case("___some_words_here")
        );
        assert_eq!("HELLO", to_screaming_snake_case("Hello"));
        assert_eq!("___HELLO", to_screaming_snake_case("___Hello"));
        assert_eq!("BLAH32", to_screaming_snake_case("Blah32"));
        assert_eq!("SOME_WORDS_HERE", to_screaming_snake_case("SomeWordsHere"));
        assert_eq!(
            "___SOME_WORDS_HERE",
            to_screaming_snake_case("___SomeWordsHere")
        );
        assert_eq!("SOME_WORDS_HERE", to_screaming_snake_case("someWordsHere"));
        assert_eq!(
            "___SOME_WORDS_HERE",
            to_screaming_snake_case("___someWordsHere")
        );
        assert_eq!("MIX_OF_THINGS", to_screaming_snake_case("Mix_Of_Things"));
        assert_eq!(
            "__MIX_OF_THINGS",
            to_screaming_snake_case("__Mix_Of_Things")
        );
        assert_eq!("FOO_BAR_123", to_screaming_snake_case("FooBar_123"));
    }

    #[test]
    fn convert_to_upper_camel_case() {
        assert_eq!("Hello", to_upper_camel_case("hello"));
        assert_eq!("___Hello", to_upper_camel_case("___hello"));
        assert_eq!("Blah32", to_upper_camel_case("blah32"));
        assert_eq!("SomeWordsHere", to_upper_camel_case("some_words_here"));
        assert_eq!(
            "___SomeWordsHere",
            to_upper_camel_case("___some_words_here")
        );
        assert_eq!("Hello", to_upper_camel_case("HELLO"));
        assert_eq!("___Hello", to_upper_camel_case("___HELLO"));
        assert_eq!("Blah32", to_upper_camel_case("BLAH32"));
        assert_eq!("SomeWordsHere", to_upper_camel_case("SOME_WORDS_HERE"));
        assert_eq!(
            "___SomeWordsHere",
            to_upper_camel_case("___SOME_WORDS_HERE")
        );
        assert_eq!("SomeWordsHere", to_upper_camel_case("someWordsHere"));
        assert_eq!("___SomeWordsHere", to_upper_camel_case("___someWordsHere"));
        assert_eq!("MixOfThings", to_upper_camel_case("Mix_Of_Things"));
        assert_eq!("__MixOfThings", to_upper_camel_case("__Mix_Of_Things"));
        assert_eq!("FooBar123", to_upper_camel_case("FooBar_123"));
    }
}
