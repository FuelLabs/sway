use core::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use sway_types::{integer_bits::IntegerBits, Ident, Span, Spanned};

// TODO: since moving to using Idents instead of strings,
// the warning_content will usually contain a duplicate of the span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompileWarning {
    pub span: Span,
    pub warning_content: Warning,
}

impl Spanned for CompileWarning {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl CompileWarning {
    pub fn to_friendly_warning_string(&self) -> String {
        self.warning_content.to_string()
    }

    pub fn path(&self) -> Option<Arc<PathBuf>> {
        self.span.path().cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Warning {
    NonClassCaseStructName {
        struct_name: Ident,
    },
    NonClassCaseTypeParameter {
        name: Ident,
    },
    NonClassCaseTraitName {
        name: Ident,
    },
    NonClassCaseEnumName {
        enum_name: Ident,
    },
    NonClassCaseEnumVariantName {
        variant_name: Ident,
    },
    NonSnakeCaseStructFieldName {
        field_name: Ident,
    },
    NonSnakeCaseFunctionName {
        name: Ident,
    },
    NonScreamingSnakeCaseConstName {
        name: Ident,
    },
    LossOfPrecision {
        initial_type: IntegerBits,
        cast_to: IntegerBits,
    },
    UnusedReturnValue {
        r#type: String,
    },
    SimilarMethodFound {
        lib: Ident,
        module: Ident,
        name: Ident,
    },
    ShadowsOtherSymbol {
        name: Ident,
    },
    OverridingTraitImplementation,
    DeadDeclaration,
    DeadFunctionDeclaration,
    DeadStructDeclaration,
    DeadTrait,
    UnreachableCode,
    DeadEnumVariant {
        variant_name: Ident,
    },
    DeadMethod,
    StructFieldNeverRead,
    ShadowingReservedRegister {
        reg_name: Ident,
    },
    DeadStorageDeclaration,
    DeadStorageDeclarationForFunction {
        unneeded_attrib: String,
    },
    MatchExpressionUnreachableArm,
    UnrecognizedAttribute {
        attrib_name: Ident,
    },
    StorageWriteAfterInteraction {
        block_name: Ident,
    },
}

impl fmt::Display for Warning {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use sway_types::style::*;
        use Warning::*;
        match self {
            NonClassCaseStructName { struct_name } => {
                write!(f,
                "Struct name \"{}\" is not idiomatic. Structs should have a ClassCase name, like \
                 \"{}\".",
                struct_name,
                to_upper_camel_case(struct_name.as_str())
            )
            }
            NonClassCaseTypeParameter { name } => {
                write!(f,
                "Type parameter \"{}\" is not idiomatic. TypeParameters should have a ClassCase name, like \
                 \"{}\".",
                name,
                to_upper_camel_case(name.as_str())
            )
            }
            NonClassCaseTraitName { name } => {
                write!(f,
                "Trait name \"{}\" is not idiomatic. Traits should have a ClassCase name, like \
                 \"{}\".",
                name,
                to_upper_camel_case(name.as_str())
            )
            }
            NonClassCaseEnumName { enum_name } => write!(
                f,
                "Enum \"{}\"'s capitalization is not idiomatic. Enums should have a ClassCase \
                 name, like \"{}\".",
                enum_name,
                to_upper_camel_case(enum_name.as_str())
            ),
            NonSnakeCaseStructFieldName { field_name } => write!(
                f,
                "Struct field name \"{}\" is not idiomatic. Struct field names should have a \
                 snake_case name, like \"{}\".",
                field_name,
                to_snake_case(field_name.as_str())
            ),
            NonClassCaseEnumVariantName { variant_name } => write!(
                f,
                "Enum variant name \"{}\" is not idiomatic. Enum variant names should be \
                 ClassCase, like \"{}\".",
                variant_name,
                to_upper_camel_case(variant_name.as_str())
            ),
            NonSnakeCaseFunctionName { name } => {
                write!(f,
                "Function name \"{}\" is not idiomatic. Function names should be snake_case, like \
                 \"{}\".",
                name,
                to_snake_case(name.as_str())
            )
            }
            NonScreamingSnakeCaseConstName { name } => {
                write!(
                    f,
                    "Constant name \"{}\" is not idiomatic. Constant names should be SCREAMING_SNAKE_CASE, like \
                    \"{}\".",
                    name,
                    to_screaming_snake_case(name.as_str()),
                )
            },
            LossOfPrecision {
                initial_type,
                cast_to,
            } => write!(f,
                "This cast, from integer type of width {} to integer type of width {}, will lose precision.",
                initial_type,
                cast_to
            ),
            UnusedReturnValue { r#type } => write!(
                f,
                "This returns a value of type {}, which is not assigned to anything and is \
                 ignored.",
                r#type
            ),
            SimilarMethodFound { lib, module, name } => write!(
                f,
                "A method with the same name was found for type {} in dependency \"{}::{}\". \
                 Traits must be in scope in order to access their methods. ",
                name, lib, module
            ),
            ShadowsOtherSymbol { name } => write!(
                f,
                "This shadows another symbol in this scope with the same name \"{}\".",
                name
            ),
            OverridingTraitImplementation => write!(
                f,
                "This trait implementation overrides another one that was previously defined."
            ),
            DeadDeclaration => write!(f, "This declaration is never used."),
            DeadStructDeclaration => write!(f, "This struct is never used."),
            DeadFunctionDeclaration => write!(f, "This function is never called."),
            UnreachableCode => write!(f, "This code is unreachable."),
            DeadEnumVariant { variant_name } => {
                write!(f, "Enum variant {} is never constructed.", variant_name)
            }
            DeadTrait => write!(f, "This trait is never implemented."),
            DeadMethod => write!(f, "This method is never called."),
            StructFieldNeverRead => write!(f, "This struct field is never accessed."),
            ShadowingReservedRegister { reg_name } => write!(
                f,
                "This register declaration shadows the reserved register, \"{}\".",
                reg_name
            ),
            DeadStorageDeclaration => write!(
                f,
                "This storage declaration is never accessed and can be removed."
            ),
            DeadStorageDeclarationForFunction { unneeded_attrib } => write!(
                f,
                "The '{unneeded_attrib}' storage declaration for this function is never accessed \
                and can be removed."
            ),
            MatchExpressionUnreachableArm => write!(f, "This match arm is unreachable."),
            UnrecognizedAttribute {attrib_name} => write!(f, "Unknown attribute: \"{attrib_name}\"."),
            StorageWriteAfterInteraction {block_name} => write!(f, "Storage modification after external contract interaction in function or method \"{block_name}\". \
            Consider making all storage writes before calling another contract"),
        }
    }
}

#[cfg(test)]
mod test {
    use sway_types::style::*;

    #[test]
    fn detect_styles() {
        let snake_cases = [
            "hello",
            "__hello",
            "blah32",
            "some_words_here",
            "___some_words_here",
        ];
        let screaming_snake_cases = ["SOME_WORDS_HERE", "___SOME_WORDS_HERE"];
        let upper_camel_cases = [
            "Hello",
            "__Hello",
            "Blah32",
            "SomeWordsHere",
            "___SomeWordsHere",
        ];
        let screaming_snake_case_or_upper_camel_case_idents = ["HELLO", "__HELLO", "BLAH32"];
        let styleless_idents = ["Mix_Of_Things", "__Mix_Of_Things", "FooBar_123"];
        for name in &snake_cases {
            assert!(is_snake_case(name));
            assert!(!is_screaming_snake_case(name));
            assert!(!is_upper_camel_case(name));
        }
        for name in &screaming_snake_cases {
            assert!(!is_snake_case(name));
            assert!(is_screaming_snake_case(name));
            assert!(!is_upper_camel_case(name));
        }
        for name in &upper_camel_cases {
            assert!(!is_snake_case(name));
            assert!(!is_screaming_snake_case(name));
            assert!(is_upper_camel_case(name));
        }
        for name in &screaming_snake_case_or_upper_camel_case_idents {
            assert!(!is_snake_case(name));
            assert!(is_screaming_snake_case(name));
            assert!(is_upper_camel_case(name));
        }
        for name in &styleless_idents {
            assert!(!is_snake_case(name));
            assert!(!is_screaming_snake_case(name));
            assert!(!is_upper_camel_case(name));
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
