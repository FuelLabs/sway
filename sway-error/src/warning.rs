use crate::{
    diagnostic::{Code, Diagnostic, Hint, Issue, Reason, ToDiagnostic},
    formatting::{
        did_you_mean_help, num_to_str, sequence_to_list, sequence_to_str, Enclosing, Indent,
    },
};

use core::fmt;

use either::Either;

use sway_types::{Ident, IdentUnique, SourceId, Span, Spanned};

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

    pub fn source_id(&self) -> Option<SourceId> {
        self.span.source_id().cloned()
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
    AsmBlockIsEmpty,
    UninitializedAsmRegShadowsItem {
        /// Text "Constant" or "Configurable" or "Variable".
        /// Denotes the type of the `item` that shadows the uninitialized ASM register.
        constant_or_configurable_or_variable: &'static str,
        /// The name of the item that shadows the register, that points to the name in
        /// the item declaration.
        item: IdentUnique,
    },
    OverridingTraitImplementation,
    DeadDeclaration,
    DeadEnumDeclaration,
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
    MatchExpressionUnreachableArm {
        match_value: Span,
        match_type: String,
        // Either preceding non catch-all arms or a single interior catch-all arm.
        preceding_arms: Either<Vec<Span>, Span>,
        unreachable_arm: Span,
        is_last_arm: bool,
        is_catch_all_arm: bool,
    },
    UnknownAttribute {
        attribute: IdentUnique,
        known_attributes: &'static [&'static str],
    },
    UnknownAttributeArg {
        attribute: Ident,
        arg: IdentUnique,
        expected_args: Vec<&'static str>,
    },
    EffectAfterInteraction {
        effect: String,
        effect_in_suggestion: String,
        block_name: Ident,
    },
    UsingDeprecated {
        deprecated_element: DeprecatedElement,
        deprecated_element_name: String,
        help: Option<String>,
    },
    DuplicatedStorageKey {
        first_field: IdentUnique,
        first_field_full_name: String,
        first_field_key_is_compiler_generated: bool,
        second_field: IdentUnique,
        second_field_full_name: String,
        second_field_key_is_compiler_generated: bool,
        key: String,
    },
    ErrorTypeEmptyEnum {
        enum_name: IdentUnique,
    },
    ErrorEmptyErrorMessage {
        enum_name: Ident,
        enum_variant_name: Ident,
    },
    ErrorDuplicatedErrorMessage {
        last_occurrence: Span,
        previous_occurrences: Vec<Span>,
    },
}

/// Elements that can be deprecated.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeprecatedElement {
    Struct,
    StructField,
    Enum,
    EnumVariant,
    Function,
    Const,
    Configurable,
}

impl fmt::Display for DeprecatedElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Struct => write!(f, "Struct"),
            Self::StructField => write!(f, "Struct field"),
            Self::Enum => write!(f, "Enum"),
            Self::EnumVariant => write!(f, "Enum variant"),
            Self::Function => write!(f, "Function"),
            Self::Const => write!(f, "Constant"),
            Self::Configurable => write!(f, "Configurable"),
        }
    }
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
                    "Constant name \"{name}\" is not idiomatic. Constant names should be SCREAMING_SNAKE_CASE, like \
                    \"{}\".",
                    to_screaming_snake_case(name.as_str()),
                )
            },
            UnusedReturnValue { r#type } => write!(
                f,
                "This returns a value of type {type}, which is not assigned to anything and is \
                 ignored."
            ),
            SimilarMethodFound { lib, module, name } => write!(
                f,
                "A method with the same name was found for type {name} in dependency \"{lib}::{module}\". \
                 Traits must be in scope in order to access their methods. "
            ),
            ShadowsOtherSymbol { name } => write!(
                f,
                "This shadows another symbol in this scope with the same name \"{name}\"."
            ),
            AsmBlockIsEmpty => write!(
                f,
                "This ASM block is empty."
            ),
            UninitializedAsmRegShadowsItem { constant_or_configurable_or_variable, item } => write!(
                f,
                "This uninitialized register is shadowing a {}. You probably meant to also initialize it, like \"{item}: {item}\".",
                constant_or_configurable_or_variable.to_ascii_lowercase(),
            ),
            OverridingTraitImplementation => write!(
                f,
                "This trait implementation overrides another one that was previously defined."
            ),
            DeadDeclaration => write!(f, "This declaration is never used."),
            DeadEnumDeclaration => write!(f, "This enum is never used."),
            DeadStructDeclaration => write!(f, "This struct is never used."),
            DeadFunctionDeclaration => write!(f, "This function is never called."),
            UnreachableCode => write!(f, "This code is unreachable."),
            DeadEnumVariant { variant_name } => {
                write!(f, "Enum variant {variant_name} is never constructed.")
            }
            DeadTrait => write!(f, "This trait is never implemented."),
            DeadMethod => write!(f, "This method is never called."),
            StructFieldNeverRead => write!(f, "This struct field is never accessed."),
            ShadowingReservedRegister { reg_name } => write!(
                f,
                "This register declaration shadows the reserved register, \"{reg_name}\"."
            ),
            DeadStorageDeclaration => write!(
                f,
                "This storage declaration is never accessed and can be removed."
            ),
            DeadStorageDeclarationForFunction { unneeded_attrib } => write!(
                f,
                "This function's storage attributes declaration does not match its \
                 actual storage access pattern: '{unneeded_attrib}' attribute(s) can be removed."
            ),
            MatchExpressionUnreachableArm { .. } => write!(f, "This match arm is unreachable."),
            UnknownAttribute { attribute, .. } => write!(f, "Unknown attribute \"{attribute}\"."),
            UnknownAttributeArg { attribute, arg, expected_args } => write!(
                f,
                "\"{arg}\" is an unknown argument for attribute \"{attribute}\". Known arguments are: {}.", sequence_to_str(expected_args, Enclosing::DoubleQuote, usize::MAX)
            ),
            EffectAfterInteraction {effect, effect_in_suggestion, block_name} =>
                write!(f, "{effect} after external contract interaction in function or method \"{block_name}\". \
                          Consider {effect_in_suggestion} before calling another contract"),
            UsingDeprecated { deprecated_element_name, deprecated_element, help } =>
                write!(f, "{deprecated_element} \"{deprecated_element_name}\" is deprecated. {}", help.as_ref().unwrap_or(&"".into())),
            DuplicatedStorageKey { first_field_full_name, second_field_full_name, key, .. } =>
                write!(f, "Two storage fields have the same storage key.\nFirst field: {first_field_full_name}\nSecond field: {second_field_full_name}\nKey: {key}"),
            ErrorTypeEmptyEnum { enum_name } =>
                write!(f, "Empty error type enum \"{enum_name}\" can never be instantiated and used in `panic` expressions."),
            ErrorEmptyErrorMessage { enum_name, enum_variant_name } =>
                write!(f, "Error enum variant \"{enum_name}::{enum_variant_name}\" has an empty error message. Consider adding a helpful error message."),
            ErrorDuplicatedErrorMessage { previous_occurrences, .. } =>
                write!(f, "This error message is duplicated{}. Consider using a unique error message for every error variant.",
                    if previous_occurrences.len() == 1 {
                        "".to_string()
                    } else {
                        format!(" {} times", num_to_str(previous_occurrences.len()))
                    }
                ),
        }
    }
}

#[allow(dead_code)]
const FUTURE_HARD_ERROR_HELP: &str =
    "In future versions of Sway this warning will become a hard error.";

impl ToDiagnostic for CompileWarning {
    fn to_diagnostic(&self, source_engine: &sway_types::SourceEngine) -> Diagnostic {
        let code = Code::warnings;
        use sway_types::style::*;
        use Warning::*;
        match &self.warning_content {
            NonScreamingSnakeCaseConstName { name } => Diagnostic {
                reason: Some(Reason::new(code(1), "Constant name is not idiomatic".to_string())),
                issue: Issue::warning(
                    source_engine,
                    name.span(),
                    format!("Constant \"{name}\" should be SCREAMING_SNAKE_CASE."),
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        name.span(),
                        format!("Consider renaming it to, e.g., \"{}\".", to_screaming_snake_case(name.as_str())),
                    ),
                ],
                help: vec![
                    format!("In Sway, ABIs, structs, traits, and enums are CapitalCase."),
                    format!("Modules, variables, and functions are snake_case, while constants are SCREAMING_SNAKE_CASE."),
                ],
            },
            MatchExpressionUnreachableArm { match_value, match_type, preceding_arms, unreachable_arm, is_last_arm, is_catch_all_arm } => Diagnostic {
                reason: Some(Reason::new(code(1), "Match arm is unreachable".to_string())),
                issue: Issue::warning(
                    source_engine,
                    unreachable_arm.clone(),
                    match (*is_last_arm, *is_catch_all_arm) {
                        (true, true) => format!("Last catch-all match arm `{}` is unreachable.", unreachable_arm.as_str()),
                        _ => format!("Match arm `{}` is unreachable.", unreachable_arm.as_str())
                    }
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        match_value.clone(),
                        format!("The expression to match on is of type \"{match_type}\".")
                    ),
                    if preceding_arms.is_right() {
                        Hint::help(
                            source_engine,
                            preceding_arms.as_ref().unwrap_right().clone(),
                            format!("Catch-all arm `{}` makes all match arms below it unreachable.", preceding_arms.as_ref().unwrap_right().as_str())
                        )
                    }
                    else {
                        Hint::info(
                            source_engine,
                            Span::join_all(preceding_arms.as_ref().unwrap_left().clone()),
                            if *is_last_arm {
                                format!("Preceding match arms already match all possible values of `{}`.", match_value.as_str())
                            }
                            else {
                                format!("Preceding match arms already match all the values that `{}` can match.", unreachable_arm.as_str())
                            }
                        )
                    }
                ],
                help: if preceding_arms.is_right() {
                    let catch_all_arm = preceding_arms.as_ref().unwrap_right().as_str();
                    vec![
                        format!("Catch-all patterns make sense only in last match arms."),
                        format!("Consider removing the catch-all arm `{catch_all_arm}` or making it the last arm."),
                        format!("Consider removing the unreachable arms below the `{catch_all_arm}` arm."),
                    ]
                }
                else if *is_last_arm && *is_catch_all_arm {
                    vec![
                        format!("Catch-all patterns are often used in last match arms."),
                        format!("But in this case, the preceding arms already match all possible values of `{}`.", match_value.as_str()),
                        format!("Consider removing the unreachable last catch-all arm."),
                    ]
                }
                else {
                    vec![
                        format!("Consider removing the unreachable arm."),
                    ]
                }
            },
            UninitializedAsmRegShadowsItem { constant_or_configurable_or_variable, item } => Diagnostic {
                reason: Some(Reason::new(code(1), format!("Uninitialized ASM register is shadowing a {}", constant_or_configurable_or_variable.to_ascii_lowercase()))),
                issue: Issue::warning(
                    source_engine,
                    self.span(),
                    format!("Uninitialized register \"{item}\" is shadowing a {} of the same name.", constant_or_configurable_or_variable.to_ascii_lowercase()),
                ),
                hints: {
                    let mut hints = vec![
                        Hint::info(
                            source_engine,
                            item.span(),
                            format!("{constant_or_configurable_or_variable} \"{item}\" is declared here.")
                        ),
                    ];

                    hints.append(&mut Hint::multi_help(
                        source_engine,
                        &self.span(),
                        vec![
                            format!("Are you trying to initialize the register to the value of the {}?", constant_or_configurable_or_variable.to_ascii_lowercase()),
                            format!("In that case, you must do it explicitly: `{item}: {item}`."),
                            format!("Otherwise, to avoid the confusion with the shadowed {}, consider renaming the register \"{item}\".", constant_or_configurable_or_variable.to_ascii_lowercase()),
                        ]
                    ));

                    hints
                },
                help: vec![],
            },
            AsmBlockIsEmpty => Diagnostic {
                reason: Some(Reason::new(code(1), "ASM block is empty".to_string())),
                issue: Issue::warning(
                    source_engine,
                    self.span(),
                    "This ASM block is empty.".to_string(),
                ),
                hints: vec![],
                help: vec![
                    "Consider adding assembly instructions or a return register to the ASM block, or removing the block altogether.".to_string(),
                ],
            },
            DuplicatedStorageKey { first_field, first_field_full_name, first_field_key_is_compiler_generated, second_field, second_field_full_name, second_field_key_is_compiler_generated, key } => Diagnostic {
                reason: Some(Reason::new(code(1), "Two storage fields have the same storage key".to_string())),
                issue: Issue::warning(
                    source_engine,
                    first_field.span(),
                    format!("\"{first_field_full_name}\" has the same storage key as \"{second_field_full_name}\"."),
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        second_field.span(),
                        format!("\"{second_field_full_name}\" is declared here."),
                    ),
                ],
                help: vec![
                    if *first_field_key_is_compiler_generated || *second_field_key_is_compiler_generated {
                        format!("The key of \"{}\" is generated by the compiler using the following formula:",
                            if *first_field_key_is_compiler_generated {
                                first_field_full_name
                            } else {
                                second_field_full_name
                            }
                        )
                    } else {
                        "Both keys are explicitly defined by using the `in` keyword.".to_string()
                    },
                    if *first_field_key_is_compiler_generated || *second_field_key_is_compiler_generated {
                        format!("{}sha256((0u8, \"{}\"))",
                            Indent::Single,
                            if *first_field_key_is_compiler_generated {
                                first_field_full_name
                            } else {
                                second_field_full_name
                            }
                        )
                    } else {
                        Diagnostic::help_none()
                    },
                    format!("The common key is: {key}.")
                ],
            },
            UnknownAttribute { attribute, known_attributes } => Diagnostic {
                reason: Some(Reason::new(code(1), "Attribute is unknown".to_string())),
                issue: Issue::warning(
                    source_engine,
                    attribute.span(),
                    format!("\"{attribute}\" attribute is unknown.")
                ),
                hints: vec![did_you_mean_help(source_engine, attribute.span(), known_attributes.iter(), 2, Enclosing::DoubleQuote)],
                help: vec![
                    "Unknown attributes are allowed and can be used by third-party tools,".to_string(),
                    "but the compiler ignores them.".to_string(),
                ],
            },
            UnknownAttributeArg { attribute, arg, expected_args } => Diagnostic {
                reason: Some(Reason::new(code(1), "Attribute argument is unknown".to_string())),
                issue: Issue::warning(
                    source_engine,
                    arg.span(),
                    format!("\"{arg}\" is an unknown argument for attribute \"{attribute}\".")
                ),
                hints: {
                    let mut hints = vec![did_you_mean_help(source_engine, arg.span(), expected_args, 2, Enclosing::DoubleQuote)];
                    if expected_args.len() == 1 {
                        hints.push(Hint::help(source_engine, arg.span(), format!("The only known argument is \"{}\".", expected_args[0])));
                    } else if expected_args.len() <= 3 {
                        hints.push(Hint::help(source_engine, arg.span(), format!("Known arguments are {}.", sequence_to_str(expected_args, Enclosing::DoubleQuote, usize::MAX))));
                    } else {
                        hints.push(Hint::help(source_engine, arg.span(), "Known arguments are:".to_string()));
                        hints.append(&mut Hint::multi_help(source_engine, &arg.span(), sequence_to_list(expected_args, Indent::Single, usize::MAX)))
                    }
                    hints
                },
                help: vec![
                    format!("Unknown attribute arguments are allowed for some attributes like \"{attribute}\"."),
                    "They can be used by third-party tools, but the compiler ignores them.".to_string(),
                ],
            },
            UsingDeprecated { deprecated_element, deprecated_element_name, help } => Diagnostic {
                reason: Some(Reason::new(code(1), format!("{deprecated_element} is deprecated"))),
                issue: Issue::warning(
                    source_engine,
                    self.span(),
                    format!("{deprecated_element} \"{deprecated_element_name}\" is deprecated."),
                ),
                hints: help.as_ref().map_or(vec![], |help| vec![
                    Hint::help(
                        source_engine,
                        self.span(),
                        help.clone(),
                    ),
                ]),
                help: vec![],
            },
            ErrorTypeEmptyEnum { enum_name } => Diagnostic {
                reason: Some(Reason::new(code(1), "Empty error type enum cannot be used in `panic` expressions".to_string())),
                issue: Issue::warning(
                    source_engine,
                    enum_name.span(),
                    format!("Error type enum \"{enum_name}\" is empty and can never be used in `panic` expressions."),
                ),
                hints: vec![],
                help: vec![
                    "Empty enums with no enum variants can never be instantiated.".to_string(),
                    "Thus, they cannot have instances to use as arguments in `panic` expressions.".to_string(),
                    Diagnostic::help_empty_line(),
                    format!("Consider adding enum variants to \"{enum_name}\" and attributing them"),
                    "with the `#[error]` attribute.".to_string(),
                ],
            },
            ErrorEmptyErrorMessage { enum_name, enum_variant_name } => Diagnostic {
                reason: Some(Reason::new(code(1), "Error message is empty".to_string())),
                issue: Issue::warning(
                    source_engine,
                    self.span(),
                    format!("Error enum variant \"{enum_name}::{enum_variant_name}\" has an empty error message."),
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        self.span(),
                        "Consider adding a helpful error message here.".to_string(),
                    )
                ],
                help: vec![],
            },
            ErrorDuplicatedErrorMessage { last_occurrence, previous_occurrences } => Diagnostic {
                reason: Some(Reason::new(code(1), "Error message is duplicated".to_string())),
                issue: Issue::error(
                    source_engine,
                    last_occurrence.clone(),
                    "This error message is duplicated.".to_string(),
                ),
                hints: {
                    let (first_occurrence, other_occurrences) = previous_occurrences.split_first().expect("there is at least one previous occurrence in `previous_occurrences`");
                    let mut hints = vec![Hint::info(source_engine, first_occurrence.clone(), "It is already used here.".to_string())];
                    other_occurrences.iter().for_each(|occurrence| hints.push(Hint::info(source_engine, occurrence.clone(), "And here.".to_string())));
                    hints.push(Hint::help(source_engine, last_occurrence.clone(), "Consider using a unique error message for every error variant.".to_string()));
                    hints
                },
                help: vec![],
            },
           _ => Diagnostic {
                    // TODO: Temporarily we use self here to achieve backward compatibility.
                    //       In general, self must not be used and will not be used once we
                    //       switch to our own #[error] macro. All the values for the formatting
                    //       of a diagnostic must come from the enum variant parameters.
                    issue: Issue::warning(source_engine, self.span(), format!("{}", self.warning_content)),
                    ..Default::default()
                }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollectedTraitImpl {
    pub impl_span: Span,
    pub trait_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Info {
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Info::*;
        match self {
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompileInfo {
    pub span: Span,
    pub content: Info,
}

impl Spanned for CompileInfo {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl CompileInfo {
    pub fn source_id(&self) -> Option<SourceId> {
        self.span.source_id().cloned()
    }

    pub fn to_friendly_string(&self) -> String {
        self.content.to_string()
    }
}

impl ToDiagnostic for CompileInfo {
    fn to_diagnostic(&self, source_engine: &sway_types::SourceEngine) -> Diagnostic {
        let code = Code::warnings;
        use Info::*;
        match &self.content {
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
