use crate::diagnostic::{Code, Diagnostic, Hint, Issue, Reason, ToDiagnostic};

use crate::error::StructFieldUsageContext; // TODO: Only temporary. Will be removed once struct field privacy becomes a hard error.
use crate::error::StructFieldUsageContext::*;

use crate::formatting::*;

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
    UninitializedAsmRegShadowsVariable {
        name: Ident,
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
    UnrecognizedAttribute {
        attrib_name: Ident,
    },
    AttributeExpectedNumberOfArguments {
        attrib_name: Ident,
        received_args: usize,
        expected_min_len: usize,
        expected_max_len: Option<usize>,
    },
    UnexpectedAttributeArgumentValue {
        attrib_name: Ident,
        received_value: String,
        expected_values: Vec<String>,
    },
    EffectAfterInteraction {
        effect: String,
        effect_in_suggestion: String,
        block_name: Ident,
    },
    ModulePrivacyDisabled,
    UsingDeprecated {
        message: String,
    },
    // TODO: Remove all these warnings once private struct fields violations become a hard error.
    MatchStructPatternMustIgnorePrivateFields {
        private_fields: Vec<Ident>,
        /// Original, non-aliased struct name.
        struct_name: Ident,
        struct_decl_span: Span,
        all_fields_are_private: bool,
        span: Span,
    },
    StructFieldIsPrivate {
        field_name: IdentUnique,
        /// Original, non-aliased struct name.
        struct_name: Ident,
        field_decl_span: Span,
        struct_can_be_changed: bool,
        usage_context: StructFieldUsageContext,
    },
    StructCannotBeInstantiated {
        /// Original, non-aliased struct name.
        struct_name: Ident,
        span: Span,
        struct_decl_span: Span,
        private_fields: Vec<Ident>,
        constructors: Vec<String>,
        /// True if the struct has only private fields.
        all_fields_are_private: bool,
        is_in_storage_declaration: bool,
        struct_can_be_changed: bool,
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
            UninitializedAsmRegShadowsVariable { name } => write!(
                f,
                "This unitialized register is shadowing a variable, you probably meant to also initialize it like \"{name}: {name}\"."
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
            UnrecognizedAttribute {attrib_name} => write!(f, "Unknown attribute: \"{attrib_name}\"."),
            AttributeExpectedNumberOfArguments {attrib_name, received_args, expected_min_len, expected_max_len } => write!(
                f,
                "Attribute: \"{attrib_name}\" expected {} argument(s) received {received_args}.",
                if let Some(expected_max_len) = expected_max_len {
                    if expected_min_len == expected_max_len {
                        format!("exactly {expected_min_len}")
                    } else {
                        format!("between {expected_min_len} and {expected_max_len}")
                    }
                } else {
                    format!("at least {expected_min_len}")
                }
            ),
            UnexpectedAttributeArgumentValue {attrib_name, received_value, expected_values } => write!(
                f,
                "Unexpected attribute value: \"{received_value}\" for attribute: \"{attrib_name}\" expected value {}",
                expected_values.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(" or ")
            ),
            EffectAfterInteraction {effect, effect_in_suggestion, block_name} =>
                write!(f, "{effect} after external contract interaction in function or method \"{block_name}\". \
                          Consider {effect_in_suggestion} before calling another contract"),
            ModulePrivacyDisabled => write!(f, "Module privacy rules will soon change to make modules private by default.
                                            You can enable the new behavior with the --experimental-private-modules flag, which will become the default behavior in a later release.
                                            More details are available in the related RFC: https://github.com/FuelLabs/sway-rfcs/blob/master/rfcs/0008-private-modules.md"),
            UsingDeprecated { message } => write!(f, "{}", message),
            MatchStructPatternMustIgnorePrivateFields { private_fields, .. } => write!(f,
                "Struct pattern must ignore inaccessible private field{} {}.",
                plural_s(private_fields.len()),
                sequence_to_str(private_fields, Enclosing::DoubleQuote, 2)
            ),
            StructFieldIsPrivate { field_name, struct_name, .. } => write!(f,
                "Field \"{field_name}\" of the struct \"{struct_name}\" is private."
            ),
            StructCannotBeInstantiated { struct_name, ..} => write!(f,
                "Struct \"{struct_name}\" cannot be instantiated here because it has private fields."
            ),
        }
    }
}

#[allow(dead_code)]
const FUTURE_HARD_ERROR_HELP: &str =
    "*** In the upcoming versions of Sway this warning will become a hard error. ***";

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
            MatchStructPatternMustIgnorePrivateFields { private_fields, struct_name, struct_decl_span, all_fields_are_private, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Struct pattern must ignore inaccessible private fields".to_string())),
                issue: Issue::warning(
                    source_engine,
                    span.clone(),
                    format!("Struct pattern must ignore inaccessible private field{} {}.",
                        plural_s(private_fields.len()),
                        sequence_to_str(private_fields, Enclosing::DoubleQuote, 2)
                    )
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        span.clone(),
                        format!("To ignore the private field{}, end the struct pattern with `..`.",
                            plural_s(private_fields.len()),
                        )
                    ),
                    Hint::warning(
                        source_engine,
                        span.clone(),
                        FUTURE_HARD_ERROR_HELP.to_string()
                    ),
                    Hint::info(
                        source_engine,
                        struct_decl_span.clone(),
                        format!("Struct \"{struct_name}\" is declared here, and has {}.",
                            if *all_fields_are_private {
                                "all private fields".to_string()
                            } else {
                                format!("private field{} {}",
                                    plural_s(private_fields.len()),
                                    sequence_to_str(private_fields, Enclosing::DoubleQuote, 2)
                                )
                            }
                        )
                    ),
                ],
                help: vec![],
            },
            StructFieldIsPrivate { field_name, struct_name, field_decl_span, struct_can_be_changed, usage_context } => Diagnostic {
                reason: Some(Reason::new(code(1), "Private struct field is inaccessible".to_string())),
                issue: Issue::warning(
                    source_engine,
                    field_name.span(),
                    format!("Private field \"{field_name}\" {}is inaccessible in this module.",
                        match usage_context {
                            StructInstantiation { .. } | StorageDeclaration { .. } | PatternMatching { .. } => "".to_string(),
                            StorageAccess | StructFieldAccess => format!("of the struct \"{struct_name}\" "),
                        }
                    )
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        field_name.span(),
                        format!("Private fields can be {} only within the module in which their struct is declared.",
                            match usage_context {
                                StructInstantiation { .. } | StorageDeclaration { .. } => "initialized",
                                StorageAccess | StructFieldAccess => "accessed",
                                PatternMatching { .. } => "matched",
                            }
                        )
                    ),
                    if matches!(usage_context, PatternMatching { has_rest_pattern } if !has_rest_pattern) {
                        Hint::help(
                            source_engine,
                            field_name.span(),
                            "Otherwise, they must be ignored by ending the struct pattern with `..`.".to_string()
                        )
                    } else {
                        Hint::none()
                    },
                    Hint::warning(
                        source_engine,
                        field_name.span(),
                        FUTURE_HARD_ERROR_HELP.to_string()
                    ),
                    Hint::info(
                        source_engine,
                        field_decl_span.clone(),
                        format!("Field \"{field_name}\" {}is declared here as private.",
                            match usage_context {
                                StructInstantiation { .. } | StorageDeclaration { .. } | PatternMatching { .. } => format!("of the struct \"{struct_name}\" "),
                                StorageAccess | StructFieldAccess => "".to_string(),
                            }
                        )
                    ),
                ],
                help: vec![
                    if matches!(usage_context, PatternMatching { has_rest_pattern } if !has_rest_pattern) {
                        format!("Consider removing the field \"{field_name}\" from the struct pattern, and ending the pattern with `..`.")
                    } else {
                        Diagnostic::help_none()
                    },
                    if *struct_can_be_changed {
                        match usage_context {
                            StorageAccess | StructFieldAccess | PatternMatching { .. } => {
                                format!("{} declaring the field \"{field_name}\" as public in \"{struct_name}\": `pub {field_name}: ...,`.",
                                    if matches!(usage_context, PatternMatching { has_rest_pattern } if !has_rest_pattern) {
                                        "Alternatively, consider"
                                    } else {
                                        "Consider"
                                    }
                                )
                            },
                            // For all other usages, detailed instructions are already given in specific messages.
                            _ => Diagnostic::help_none(),
                        }
                    } else {
                        Diagnostic::help_none()
                    },
                ],
            },
            StructCannotBeInstantiated { struct_name, span, struct_decl_span, private_fields, constructors, all_fields_are_private, is_in_storage_declaration, struct_can_be_changed } => Diagnostic {
                reason: Some(Reason::new(code(1), "Struct cannot be instantiated due to inaccessible private fields".to_string())),
                issue: Issue::warning(
                    source_engine,
                    span.clone(),
                    format!("\"{struct_name}\" cannot be {}instantiated in this {}, due to {}inaccessible private field{}.",
                        if *is_in_storage_declaration { "" } else { "directly " },
                        if *is_in_storage_declaration { "storage declaration" } else { "module" },
                        singular_plural(private_fields.len(), "an ", ""),
                        plural_s(private_fields.len())
                    )
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        span.clone(),
                        format!("Inaccessible field{} {} {}.",
                            plural_s(private_fields.len()),
                            is_are(private_fields.len()),
                            sequence_to_str(private_fields, Enclosing::DoubleQuote, 5)
                        )
                    ),
                    Hint::help(
                        source_engine,
                        span.clone(),
                        if *is_in_storage_declaration {
                            "Structs with private fields can be instantiated in storage declarations only if they are declared in the same module as the storage.".to_string()
                        } else {
                            "Structs with private fields can be instantiated only within the module in which they are declared.".to_string()
                        }
                    ),
                    if *is_in_storage_declaration {
                        Hint::help(
                            source_engine,
                            span.clone(),
                            "They can still be initialized in storage declarations if they have public constructors that evaluate to a constant.".to_string()
                        )
                    } else {
                        Hint::none()
                    },
                    if *is_in_storage_declaration {
                        Hint::help(
                            source_engine,
                            span.clone(),
                            "They can always be stored in storage by using the `read` and `write` functions provided in the `std::storage::storage_api`.".to_string()
                        )
                    } else {
                        Hint::none()
                    },
                    if !*is_in_storage_declaration && !constructors.is_empty() {
                        Hint::help(
                            source_engine,
                            span.clone(),
                            format!("\"{struct_name}\" can be instantiated via public constructors suggested below.")
                        )
                    } else {
                        Hint::none()
                    },
                    Hint::info(
                        source_engine,
                        struct_decl_span.clone(),
                        format!("Struct \"{struct_name}\" is declared here, and has {}.",
                            if *all_fields_are_private {
                                "all private fields".to_string()
                            } else {
                                format!("private field{} {}",
                                    plural_s(private_fields.len()),
                                    sequence_to_str(private_fields, Enclosing::DoubleQuote, 2)
                                )
                            }
                        )
                    ),
                ],
                help: {
                    let mut help = vec![];

                    if *is_in_storage_declaration {
                        help.push(format!("Consider initializing \"{struct_name}\" by finding an available constructor that evaluates to a constant{}.",
                            if *struct_can_be_changed {
                                ", or implement a new one"
                            } else {
                                ""
                            }
                        ));

                        if !constructors.is_empty() {
                            help.push("Check these already available constructors. They might evaluate to a constant:".to_string());
                            // We always expect a very few candidates here. So let's list all of them by using `usize::MAX`.
                            for constructor in sequence_to_list(constructors, Indent::Single, usize::MAX) {
                                help.push(constructor);
                            }
                        };

                        help.push(Diagnostic::help_empty_line());

                        help.push(format!("Or you can always store instances of \"{struct_name}\" in the contract storage, by using the `std::storage::storage_api`:"));
                        help.push(format!("{}use std::storage::storage_api::{{read, write}};", Indent::Single));
                        help.push(format!("{}write(STORAGE_KEY, 0, my_{});", Indent::Single, to_snake_case(struct_name.as_str())));
                        help.push(format!("{}let my_{}_option = read::<{struct_name}>(STORAGE_KEY, 0);", Indent::Single, to_snake_case(struct_name.as_str())));
                    }
                    else if !constructors.is_empty() {
                        help.push(format!("Consider instantiating \"{struct_name}\" by using one of the available constructors{}:",
                            if *struct_can_be_changed {
                                ", or implement a new one"
                            } else {
                                ""
                            }
                        ));
                        for constructor in sequence_to_list(constructors, Indent::Single, 5) {
                            help.push(constructor);
                        }
                    }

                    if *struct_can_be_changed {
                        if *is_in_storage_declaration || !constructors.is_empty() {
                            help.push(Diagnostic::help_empty_line());
                        }

                        if !*is_in_storage_declaration && constructors.is_empty() {
                            help.push(format!("Consider implementing a public constructor for \"{struct_name}\"."));
                        };

                        help.push(
                            // Alternatively, consider declaring the field "f" as public in "Struct": `pub f: ...,`.
                            //  or
                            // Alternatively, consider declaring the fields "f" and "g" as public in "Struct": `pub <field>: ...,`.
                            //  or
                            // Alternatively, consider declaring all fields as public in "Struct": `pub <field>: ...,`.
                            format!("Alternatively, consider declaring {} as public in \"{struct_name}\": `pub {}: ...,`.",
                                if *all_fields_are_private {
                                    "all fields".to_string()
                                } else {
                                    format!("{} {}",
                                        singular_plural(private_fields.len(), "the field", "the fields"),
                                        sequence_to_str(private_fields, Enclosing::DoubleQuote, 2)
                                    )
                                },
                                if *all_fields_are_private {
                                    "<field>".to_string()
                                } else {
                                    match &private_fields[..] {
                                        [field] => format!("{field}"),
                                        _ => "<field>".to_string(),
                                    }
                                },
                            )
                        )
                    };

                    help
                }
            },
           _ => Diagnostic {
                    // TODO: Temporary we use self here to achieve backward compatibility.
                    //       In general, self must not be used and will not be used once we
                    //       switch to our own #[error] macro. All the values for the formating
                    //       of a diagnostic must come from the enum variant parameters.
                    issue: Issue::warning(source_engine, self.span(), format!("{}", self.warning_content)),
                    ..Default::default()
                }
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
