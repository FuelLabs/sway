use crate::convert_parse_tree_error::ConvertParseTreeError;
use crate::diagnostic::{Code, Diagnostic, Hint, Issue, Reason, ToDiagnostic};
use crate::formatting::*;
use crate::lex_error::LexError;
use crate::parser_error::{ParseError, ParseErrorKind};
use crate::type_error::TypeError;

use core::fmt;
use std::fmt::Formatter;
use sway_types::constants::STORAGE_PURITY_ATTRIBUTE_NAME;
use sway_types::style::to_snake_case;
use sway_types::{BaseIdent, Ident, IdentUnique, SourceEngine, Span, Spanned};
use thiserror::Error;

use self::StructFieldUsageContext::*;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterfaceName {
    Abi(Ident),
    Trait(Ident),
}

impl fmt::Display for InterfaceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterfaceName::Abi(name) => write!(f, "ABI \"{name}\""),
            InterfaceName::Trait(name) => write!(f, "trait \"{name}\""),
        }
    }
}

// TODO: Since moving to using Idents instead of strings, there are a lot of redundant spans in
//       this type. When replacing Strings + Spans with Idents, be aware of the rule explained below.

// When defining error structures that display identifiers, we prefer passing Idents over Strings.
// The error span can come from that same Ident or can be a different span.
// We handle those two cases in the following way:
//   - If the error span equals Ident's span, we use IdentUnique and never the plain Ident.
//   - If the error span is different then Ident's span, we pass Ident and Span as two separate fields.
//
// The reason for this rule is clearly communicating the difference of the two cases in every error,
// as well as avoiding issues with the error message deduplication explained below.
//
// Deduplication of error messages might remove errors that are actually not duplicates because
// although they point to the same Ident (in terms of the identifier's name), the span can be different.
// Deduplication works on hashes and Ident's hash contains only the name and not the span.
// That's why we always use IdentUnique whenever we extract the span from the provided Ident.
// Using IdentUnique also clearly communicates that we are extracting the span from the
// provided identifier.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompileError {
    #[error(
        "There was an error while evaluating the evaluation order for the module dependency graph."
    )]
    ModuleDepGraphEvaluationError {},
    #[error("A cyclic reference was found between the modules: {}.",
        modules.iter().map(|ident| ident.as_str().to_string())
    .collect::<Vec<_>>()
    .join(", "))]
    ModuleDepGraphCyclicReference { modules: Vec<BaseIdent> },

    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariable { var_name: Ident, span: Span },
    #[error("Identifier \"{name}\" was used as a variable, but it is actually a {what_it_is}.")]
    NotAVariable {
        name: Ident,
        what_it_is: &'static str,
        span: Span,
    },
    #[error("{feature} is currently not implemented.")]
    Unimplemented {
        /// The description of the unimplemented feature,
        /// formulated in a way that fits into common ending
        /// "is currently not implemented."
        /// E.g., "Using something".
        feature: String,
        /// Help lines. Empty if there is no additional help.
        /// To get an empty line between the help lines,
        /// insert a [String] containing only a space: `" ".to_string()`.
        help: Vec<String>,
        span: Span,
    },
    #[error("{0}")]
    TypeError(TypeError),
    #[error("Error parsing input: {err:?}")]
    ParseError { span: Span, err: String },
    #[error(
        "Internal compiler error: {0}\nPlease file an issue on the repository and include the \
         code that triggered this error."
    )]
    Internal(&'static str, Span),
    #[error(
        "Internal compiler error: {0}\nPlease file an issue on the repository and include the \
         code that triggered this error."
    )]
    InternalOwned(String, Span),
    #[error(
        "Predicate declaration contains no main function. Predicates require a main function."
    )]
    NoPredicateMainFunction(Span),
    #[error("A predicate's main function must return a boolean.")]
    PredicateMainDoesNotReturnBool(Span),
    #[error("Script declaration contains no main function. Scripts require a main function.")]
    NoScriptMainFunction(Span),
    #[error("Fallback function already defined in scope.")]
    MultipleDefinitionsOfFallbackFunction { name: Ident, span: Span },
    #[error("Function \"{name}\" was already defined in scope.")]
    MultipleDefinitionsOfFunction { name: Ident, span: Span },
    #[error("Name \"{name}\" is defined multiple times.")]
    MultipleDefinitionsOfName { name: Ident, span: Span },
    #[error("Constant \"{name}\" was already defined in scope.")]
    MultipleDefinitionsOfConstant { name: Ident, span: Span },
    #[error("Type \"{name}\" was already defined in scope.")]
    MultipleDefinitionsOfType { name: Ident, span: Span },
    #[error("Variable \"{}\" is already defined in match arm.", first_definition.as_str())]
    MultipleDefinitionsOfMatchArmVariable {
        match_value: Span,
        match_type: String,
        first_definition: Span,
        first_definition_is_struct_field: bool,
        duplicate: Span,
        duplicate_is_struct_field: bool,
    },
    #[error(
        "Assignment to an immutable variable. Variable \"{decl_name} is not declared as mutable."
    )]
    AssignmentToNonMutableVariable {
        /// Variable name pointing to the name in the variable declaration.
        decl_name: Ident,
        /// The complete left-hand side of the assignment.
        lhs_span: Span,
    },
    #[error(
        "Assignment to a {}. {} cannot be assigned to.",
        if *is_configurable {
            "configurable"
        } else {
            "constant"
        },
        if *is_configurable {
            "Configurables"
        } else {
            "Constants"
        }
    )]
    AssignmentToConstantOrConfigurable {
        /// Constant or configurable name pointing to the name in the constant declaration.
        decl_name: Ident,
        is_configurable: bool,
        /// The complete left-hand side of the assignment.
        lhs_span: Span,
    },
    #[error(
        "This assignment target cannot be assigned to, because {} is {}{decl_friendly_type_name} and not a mutable variable.",
        if let Some(decl_name) = decl_name {
            format!("\"{decl_name}\"")
        } else {
            "this".to_string()
        },
        a_or_an(decl_friendly_type_name)
    )]
    DeclAssignmentTargetCannotBeAssignedTo {
        /// Name of the declared variant, pointing to the name in the declaration.
        decl_name: Option<Ident>,
        /// Friendly name of the type of the declaration. E.g., "function", or "struct".
        decl_friendly_type_name: &'static str,
        /// The complete left-hand side of the assignment.
        lhs_span: Span,
    },
    #[error("This reference is not a reference to a mutable value (`&mut`).")]
    AssignmentViaNonMutableReference {
        /// Name of the reference, if the left-hand side of the assignment is a reference variable,
        /// pointing to the name in the reference variable declaration.
        ///
        /// `None` if the assignment LHS is an arbitrary expression and not a variable.
        decl_reference_name: Option<Ident>,
        /// [Span] of the right-hand side of the reference variable definition,
        /// if the left-hand side of the assignment is a reference variable.
        decl_reference_rhs: Option<Span>,
        /// The type of the reference, if the left-hand side of the assignment is a reference variable,
        /// expected to start with `&`.
        decl_reference_type: String,
        span: Span,
    },
    #[error(
        "Cannot call method \"{method_name}\" on variable \"{variable_name}\" because \
            \"{variable_name}\" is not declared as mutable."
    )]
    MethodRequiresMutableSelf {
        method_name: Ident,
        variable_name: Ident,
        span: Span,
    },
    #[error(
        "This parameter was declared as mutable, which is not supported yet, did you mean to use ref mut?"
    )]
    MutableParameterNotSupported { param_name: Ident, span: Span },
    #[error("Cannot pass immutable argument to mutable parameter.")]
    ImmutableArgumentToMutableParameter { span: Span },
    #[error("ref mut or mut parameter is not allowed for contract ABI function.")]
    RefMutableNotAllowedInContractAbi { param_name: Ident, span: Span },
    #[error("Reference to a mutable value cannot reference a constant.")]
    RefMutCannotReferenceConstant {
        /// Constant, as accessed in code. E.g.:
        ///  - `MY_CONST`
        ///  - `LIB_CONST_ALIAS`
        ///  - `::lib::module::SOME_CONST`
        constant: String,
        span: Span,
    },
    #[error("Reference to a mutable value cannot reference an immutable variable.")]
    RefMutCannotReferenceImmutableVariable {
        /// Variable name pointing to the name in the variable declaration.
        decl_name: Ident,
        span: Span,
    },
    #[error(
        "Cannot call associated function \"{fn_name}\" as a method. Use associated function \
        syntax instead."
    )]
    AssociatedFunctionCalledAsMethod { fn_name: Ident, span: Span },
    #[error(
        "Generic type \"{name}\" is not in scope. Perhaps you meant to specify type parameters in \
         the function signature? For example: \n`fn \
         {fn_name}<{comma_separated_generic_params}>({args}) -> ... `"
    )]
    TypeParameterNotInTypeScope {
        name: Ident,
        span: Span,
        comma_separated_generic_params: String,
        fn_name: Ident,
        args: String,
    },
    #[error(
        "expected: {expected} \n\
         found:    {given} \n\
         help:     The definition of this {decl_type} must \
         match the one in the {interface_name} declaration."
    )]
    MismatchedTypeInInterfaceSurface {
        interface_name: InterfaceName,
        span: Span,
        decl_type: String,
        given: String,
        expected: String,
    },
    #[error("Trait \"{name}\" cannot be found in the current scope.")]
    UnknownTrait { span: Span, name: Ident },
    #[error("Function \"{name}\" is not a part of {interface_name}'s interface surface.")]
    FunctionNotAPartOfInterfaceSurface {
        name: Ident,
        interface_name: InterfaceName,
        span: Span,
    },
    #[error("Constant \"{name}\" is not a part of {interface_name}'s interface surface.")]
    ConstantNotAPartOfInterfaceSurface {
        name: Ident,
        interface_name: InterfaceName,
        span: Span,
    },
    #[error("Type \"{name}\" is not a part of {interface_name}'s interface surface.")]
    TypeNotAPartOfInterfaceSurface {
        name: Ident,
        interface_name: InterfaceName,
        span: Span,
    },
    #[error("Constants are missing from this trait implementation: {}",
        missing_constants.iter().map(|ident| ident.as_str().to_string())
        .collect::<Vec<_>>()
        .join("\n"))]
    MissingInterfaceSurfaceConstants {
        missing_constants: Vec<BaseIdent>,
        span: Span,
    },
    #[error("Associated types are missing from this trait implementation: {}",
        missing_types.iter().map(|ident| ident.as_str().to_string())
        .collect::<Vec<_>>()
        .join("\n"))]
    MissingInterfaceSurfaceTypes {
        missing_types: Vec<BaseIdent>,
        span: Span,
    },
    #[error("Functions are missing from this trait implementation: {}",
        missing_functions.iter().map(|ident| ident.as_str().to_string())
        .collect::<Vec<_>>()
        .join("\n"))]
    MissingInterfaceSurfaceMethods {
        missing_functions: Vec<BaseIdent>,
        span: Span,
    },
    #[error("Expected {} type {} for \"{name}\", but instead found {}.", expected, if *expected == 1usize { "argument" } else { "arguments" }, given)]
    IncorrectNumberOfTypeArguments {
        name: Ident,
        given: usize,
        expected: usize,
        span: Span,
    },
    #[error("\"{name}\" does not take type arguments.")]
    DoesNotTakeTypeArguments { name: Ident, span: Span },
    #[error("\"{name}\" does not take type arguments as prefix.")]
    DoesNotTakeTypeArgumentsAsPrefix { name: Ident, span: Span },
    #[error("Type arguments are not allowed for this type.")]
    TypeArgumentsNotAllowed { span: Span },
    #[error("\"{name}\" needs type arguments.")]
    NeedsTypeArguments { name: Ident, span: Span },
    #[error(
        "Enum with name \"{name}\" could not be found in this scope. Perhaps you need to import \
         it?"
    )]
    EnumNotFound { name: Ident, span: Span },
    /// This error is used only for error recovery and is not emitted as a compiler
    /// error to the final compilation output. The compiler emits the cumulative error
    /// [CompileError::StructInstantiationMissingFields] given below, and that one also
    /// only if the struct can actually be instantiated.
    #[error("Instantiation of the struct \"{struct_name}\" is missing field \"{field_name}\".")]
    StructInstantiationMissingFieldForErrorRecovery {
        field_name: Ident,
        /// Original, non-aliased struct name.
        struct_name: Ident,
        span: Span,
    },
    #[error("Instantiation of the struct \"{struct_name}\" is missing {} {}.",
        if field_names.len() == 1 { "field" } else { "fields" },
        field_names.iter().map(|name| format!("\"{name}\"")).collect::<Vec::<_>>().join(", "))]
    StructInstantiationMissingFields {
        field_names: Vec<Ident>,
        /// Original, non-aliased struct name.
        struct_name: Ident,
        span: Span,
        struct_decl_span: Span,
        total_number_of_fields: usize,
    },
    #[error("Struct \"{struct_name}\" cannot be instantiated here because it has private fields.")]
    StructCannotBeInstantiated {
        /// Original, non-aliased struct name.
        struct_name: Ident,
        span: Span,
        struct_decl_span: Span,
        private_fields: Vec<Ident>,
        /// All available public constructors if `is_in_storage_declaration` is false,
        /// or only the public constructors that potentially evaluate to a constant
        /// if `is_in_storage_declaration` is true.
        constructors: Vec<String>,
        /// True if the struct has only private fields.
        all_fields_are_private: bool,
        is_in_storage_declaration: bool,
        struct_can_be_changed: bool,
    },
    #[error("Field \"{field_name}\" of the struct \"{struct_name}\" is private.")]
    StructFieldIsPrivate {
        field_name: IdentUnique,
        /// Original, non-aliased struct name.
        struct_name: Ident,
        field_decl_span: Span,
        struct_can_be_changed: bool,
        usage_context: StructFieldUsageContext,
    },
    #[error("Field \"{field_name}\" does not exist in struct \"{struct_name}\".")]
    StructFieldDoesNotExist {
        field_name: IdentUnique,
        /// Only public fields if `is_public_struct_access` is true.
        available_fields: Vec<Ident>,
        is_public_struct_access: bool,
        /// Original, non-aliased struct name.
        struct_name: Ident,
        struct_decl_span: Span,
        struct_is_empty: bool,
        usage_context: StructFieldUsageContext,
    },
    #[error("No method named \"{method_name}\" found for type \"{type_name}\".")]
    MethodNotFound {
        method_name: Ident,
        type_name: String,
        span: Span,
    },
    #[error("Module \"{name}\" could not be found.")]
    ModuleNotFound { span: Span, name: String },
    #[error("This expression has type \"{actually}\", which is not a struct. Fields can only be accessed on structs.")]
    FieldAccessOnNonStruct {
        actually: String,
        /// Name of the storage variable, if the field access
        /// happens within the access to a storage variable.
        storage_variable: Option<String>,
        /// Name of the field that is tried to be accessed.
        field_name: IdentUnique,
        span: Span,
    },
    #[error("This expression has type \"{actually}\", which is not a tuple. Elements can only be accessed on tuples.")]
    TupleElementAccessOnNonTuple {
        actually: String,
        span: Span,
        index: usize,
        index_span: Span,
    },
    #[error("This expression has type \"{actually}\", which is not an indexable type.")]
    NotIndexable { actually: String, span: Span },
    #[error("\"{name}\" is a {actually}, not an enum.")]
    NotAnEnum {
        name: String,
        span: Span,
        actually: String,
    },
    #[error("This is a {actually}, not a struct.")]
    NotAStruct { span: Span, actually: String },
    #[error("This is a {actually}, not an enum.")]
    DeclIsNotAnEnum { actually: String, span: Span },
    #[error("This is a {actually}, not a struct.")]
    DeclIsNotAStruct { actually: String, span: Span },
    #[error("This is a {actually}, not a function.")]
    DeclIsNotAFunction { actually: String, span: Span },
    #[error("This is a {actually}, not a variable.")]
    DeclIsNotAVariable { actually: String, span: Span },
    #[error("This is a {actually}, not an ABI.")]
    DeclIsNotAnAbi { actually: String, span: Span },
    #[error("This is a {actually}, not a trait.")]
    DeclIsNotATrait { actually: String, span: Span },
    #[error("This is a {actually}, not an impl block.")]
    DeclIsNotAnImplTrait { actually: String, span: Span },
    #[error("This is a {actually}, not a trait function.")]
    DeclIsNotATraitFn { actually: String, span: Span },
    #[error("This is a {actually}, not storage.")]
    DeclIsNotStorage { actually: String, span: Span },
    #[error("This is a {actually}, not a constant")]
    DeclIsNotAConstant { actually: String, span: Span },
    #[error("This is a {actually}, not a type alias")]
    DeclIsNotATypeAlias { actually: String, span: Span },
    #[error("Could not find symbol \"{name}\" in this scope.")]
    SymbolNotFound { name: Ident, span: Span },
    #[error("Found multiple bindings for \"{name}\" in this scope.")]
    SymbolWithMultipleBindings {
        name: Ident,
        paths: Vec<String>,
        span: Span,
    },
    #[error("Symbol \"{name}\" is private.")]
    ImportPrivateSymbol { name: Ident, span: Span },
    #[error("Module \"{name}\" is private.")]
    ImportPrivateModule { name: Ident, span: Span },
    #[error(
        "Because this if expression's value is used, an \"else\" branch is required and it must \
         return type \"{r#type}\""
    )]
    NoElseBranch { span: Span, r#type: String },
    #[error(
        "Symbol \"{name}\" does not refer to a type, it refers to a {actually_is}. It cannot be \
         used in this position."
    )]
    NotAType {
        span: Span,
        name: String,
        actually_is: &'static str,
    },
    #[error(
        "This enum variant requires an instantiation expression. Try initializing it with \
         arguments in parentheses."
    )]
    MissingEnumInstantiator { span: Span },
    #[error(
        "This path must return a value of type \"{ty}\" from function \"{function_name}\", but it \
         does not."
    )]
    PathDoesNotReturn {
        span: Span,
        ty: String,
        function_name: Ident,
    },
    #[error(
        "Expected Module level doc comment. All other attributes are unsupported at this level."
    )]
    ExpectedModuleDocComment { span: Span },
    #[error(
        "This register was not initialized in the initialization section of the ASM expression. \
         Initialized registers are: {initialized_registers}"
    )]
    UnknownRegister {
        span: Span,
        initialized_registers: String,
    },
    #[error("This opcode takes an immediate value but none was provided.")]
    MissingImmediate { span: Span },
    #[error("This immediate value is invalid.")]
    InvalidImmediateValue { span: Span },
    #[error("Variant \"{variant_name}\" does not exist on enum \"{enum_name}\"")]
    UnknownEnumVariant {
        enum_name: Ident,
        variant_name: Ident,
        span: Span,
    },
    #[error("Unknown opcode: \"{op_name}\".")]
    UnrecognizedOp { op_name: Ident, span: Span },
    #[error("Cannot infer type for type parameter \"{ty}\". Insufficient type information provided. Try annotating its type.")]
    UnableToInferGeneric { ty: String, span: Span },
    #[error("The generic type parameter \"{ty}\" is unconstrained.")]
    UnconstrainedGenericParameter { ty: String, span: Span },
    #[error("Trait \"{trait_name}\" is not implemented for type \"{ty}\".")]
    TraitConstraintNotSatisfied {
        type_id: usize, // Used to filter errors in method application type check.
        ty: String,
        trait_name: String,
        span: Span,
    },
    #[error(
        "Expects trait constraint \"{param}: {trait_name}\" which is missing from type parameter \"{param}\"."
    )]
    TraitConstraintMissing {
        param: String,
        trait_name: String,
        span: Span,
    },
    #[error("The value \"{val}\" is too large to fit in this 6-bit immediate spot.")]
    Immediate06TooLarge { val: u64, span: Span },
    #[error("The value \"{val}\" is too large to fit in this 12-bit immediate spot.")]
    Immediate12TooLarge { val: u64, span: Span },
    #[error("The value \"{val}\" is too large to fit in this 18-bit immediate spot.")]
    Immediate18TooLarge { val: u64, span: Span },
    #[error("The value \"{val}\" is too large to fit in this 24-bit immediate spot.")]
    Immediate24TooLarge { val: u64, span: Span },
    #[error(
        "This op expects {expected} register(s) as arguments, but you provided {received} register(s)."
    )]
    IncorrectNumberOfAsmRegisters {
        span: Span,
        expected: usize,
        received: usize,
    },
    #[error("This op does not take an immediate value.")]
    UnnecessaryImmediate { span: Span },
    #[error("This reference is ambiguous, and could refer to a module, enum, or function of the same name. Try qualifying the name with a path.")]
    AmbiguousPath { span: Span },
    #[error("This is a module path, and not an expression.")]
    ModulePathIsNotAnExpression { module_path: String, span: Span },
    #[error("Unknown type name.")]
    UnknownType { span: Span },
    #[error("Unknown type name \"{name}\".")]
    UnknownTypeName { name: String, span: Span },
    #[error("The file {file_path} could not be read: {stringified_error}")]
    FileCouldNotBeRead {
        span: Span,
        file_path: String,
        stringified_error: String,
    },
    #[error("This imported file must be a library. It must start with \"library;\"")]
    ImportMustBeLibrary { span: Span },
    #[error("An enum instantiaton cannot contain more than one value. This should be a single value of type {ty}.")]
    MoreThanOneEnumInstantiator { span: Span, ty: String },
    #[error("This enum variant represents the unit type, so it should not be instantiated with any value.")]
    UnnecessaryEnumInstantiator { span: Span },
    #[error("The enum variant `{ty}` is of type `unit`, so its constructor does not take arguments or parentheses. Try removing the ().")]
    UnitVariantWithParenthesesEnumInstantiator { span: Span, ty: String },
    #[error("Cannot find trait \"{name}\" in this scope.")]
    TraitNotFound { name: String, span: Span },
    #[error("Trait \"{trait_name}\" is not imported when calling \"{function_name}\".\nThe import is needed because \"{function_name}\" uses \"{trait_name}\" in one of its trait constraints.")]
    TraitNotImportedAtFunctionApplication {
        trait_name: String,
        function_name: String,
        function_call_site_span: Span,
        trait_constraint_span: Span,
        trait_candidates: Vec<String>,
    },
    #[error("This expression is not valid on the left hand side of a reassignment.")]
    InvalidExpressionOnLhs { span: Span },
    #[error("This code cannot be evaluated to a constant")]
    CannotBeEvaluatedToConst { span: Span },
    #[error(
        "This code cannot be evaluated to a configurable because its size is not always limited."
    )]
    CannotBeEvaluatedToConfigurableSizeUnknown { span: Span },
    #[error("{} \"{method_name}\" expects {expected} {} but you provided {received}.",
        if *dot_syntax_used { "Method" } else { "Function" },
        if *expected == 1usize { "argument" } else {"arguments"},
    )]
    TooManyArgumentsForFunction {
        span: Span,
        method_name: Ident,
        dot_syntax_used: bool,
        expected: usize,
        received: usize,
    },
    #[error("{} \"{method_name}\" expects {expected} {} but you provided {received}.",
        if *dot_syntax_used { "Method" } else { "Function" },
        if *expected == 1usize { "argument" } else {"arguments"},
    )]
    TooFewArgumentsForFunction {
        span: Span,
        method_name: Ident,
        dot_syntax_used: bool,
        expected: usize,
        received: usize,
    },
    #[error("The function \"{method_name}\" was called without parentheses. Try adding ().")]
    MissingParenthesesForFunction { span: Span, method_name: Ident },
    #[error("This type is invalid in a function selector. A contract ABI function selector must be a known sized type, not generic.")]
    InvalidAbiType { span: Span },
    #[error("This is a {actually_is}, not an ABI. An ABI cast requires a valid ABI to cast the address to.")]
    NotAnAbi {
        span: Span,
        actually_is: &'static str,
    },
    #[error("An ABI can only be implemented for the `Contract` type, so this implementation of an ABI for type \"{ty}\" is invalid.")]
    ImplAbiForNonContract { span: Span, ty: String },
    #[error("Conflicting implementations of trait \"{trait_name}\" for type \"{type_implementing_for}\".")]
    ConflictingImplsForTraitAndType {
        trait_name: String,
        type_implementing_for: String,
        existing_impl_span: Span,
        second_impl_span: Span,
    },
    #[error("Duplicate definitions for the {decl_kind} \"{decl_name}\" for type \"{type_implementing_for}\".")]
    DuplicateDeclDefinedForType {
        decl_kind: String,
        decl_name: String,
        type_implementing_for: String,
        span: Span,
    },
    #[error("The function \"{fn_name}\" in {interface_name} is defined with {num_parameters} parameters, but the provided implementation has {provided_parameters} parameters.")]
    IncorrectNumberOfInterfaceSurfaceFunctionParameters {
        fn_name: Ident,
        interface_name: InterfaceName,
        num_parameters: usize,
        provided_parameters: usize,
        span: Span,
    },
    #[error("This parameter was declared as type {should_be}, but argument of type {provided} was provided.")]
    ArgumentParameterTypeMismatch {
        span: Span,
        should_be: String,
        provided: String,
    },
    #[error("Function {fn_name} is recursive, which is unsupported at this time.")]
    RecursiveCall { fn_name: Ident, span: Span },
    #[error(
        "Function {fn_name} is recursive via {call_chain}, which is unsupported at this time."
    )]
    RecursiveCallChain {
        fn_name: Ident,
        call_chain: String, // Pretty list of symbols, e.g., "a, b and c".
        span: Span,
    },
    #[error("Type {name} is recursive, which is unsupported at this time.")]
    RecursiveType { name: Ident, span: Span },
    #[error("Type {name} is recursive via {type_chain}, which is unsupported at this time.")]
    RecursiveTypeChain {
        name: Ident,
        type_chain: String, // Pretty list of symbols, e.g., "a, b and c".
        span: Span,
    },
    #[error("The GM (get-metadata) opcode, when called from an external context, will cause the VM to panic.")]
    GMFromExternalContext { span: Span },
    #[error("The MINT opcode cannot be used in an external context.")]
    MintFromExternalContext { span: Span },
    #[error("The BURN opcode cannot be used in an external context.")]
    BurnFromExternalContext { span: Span },
    #[error("Contract storage cannot be used in an external context.")]
    ContractStorageFromExternalContext { span: Span },
    #[error("The {opcode} opcode cannot be used in a predicate.")]
    InvalidOpcodeFromPredicate { opcode: String, span: Span },
    #[error("Array index out of bounds; the length is {count} but the index is {index}.")]
    ArrayOutOfBounds { index: u64, count: u64, span: Span },
    #[error("Tuple index {index} is out of bounds. The tuple has {count} element{}.", plural_s(*count))]
    TupleIndexOutOfBounds {
        index: usize,
        count: usize,
        tuple_type: String,
        span: Span,
        prefix_span: Span,
    },
    #[error("Constants cannot be shadowed. {variable_or_constant} \"{name}\" shadows constant with the same name.")]
    ConstantsCannotBeShadowed {
        variable_or_constant: String,
        name: IdentUnique,
        constant_span: Span,
        constant_decl: Span,
        is_alias: bool,
    },
    #[error("Constants cannot shadow variables. The constant \"{name}\" shadows variable with the same name.")]
    ConstantShadowsVariable {
        name: IdentUnique,
        variable_span: Span,
    },
    #[error("The imported symbol \"{name}\" shadows another symbol with the same name.")]
    ShadowsOtherSymbol { name: IdentUnique },
    #[error("The name \"{name}\" is already used for a generic parameter in this scope.")]
    GenericShadowsGeneric { name: IdentUnique },
    #[error("Non-exhaustive match expression. Missing patterns {missing_patterns}")]
    MatchExpressionNonExhaustive {
        missing_patterns: String,
        span: Span,
    },
    #[error("Struct pattern is missing the {}field{} {}.",
        if *missing_fields_are_public { "public " } else { "" },
        plural_s(missing_fields.len()),
        sequence_to_str(missing_fields, Enclosing::DoubleQuote, 2)
    )]
    MatchStructPatternMissingFields {
        missing_fields: Vec<Ident>,
        missing_fields_are_public: bool,
        /// Original, non-aliased struct name.
        struct_name: Ident,
        struct_decl_span: Span,
        total_number_of_fields: usize,
        span: Span,
    },
    #[error("Struct pattern must ignore inaccessible private field{} {}.",
        plural_s(private_fields.len()),
        sequence_to_str(private_fields, Enclosing::DoubleQuote, 2))]
    MatchStructPatternMustIgnorePrivateFields {
        private_fields: Vec<Ident>,
        /// Original, non-aliased struct name.
        struct_name: Ident,
        struct_decl_span: Span,
        all_fields_are_private: bool,
        span: Span,
    },
    #[error("Variable \"{variable}\" is not defined in all alternatives.")]
    MatchArmVariableNotDefinedInAllAlternatives {
        match_value: Span,
        match_type: String,
        variable: Ident,
        missing_in_alternatives: Vec<Span>,
    },
    #[error(
        "Variable \"{variable}\" is expected to be of type \"{expected}\", but is \"{received}\"."
    )]
    MatchArmVariableMismatchedType {
        match_value: Span,
        match_type: String,
        variable: Ident,
        first_definition: Span,
        expected: String,
        received: String,
    },
    #[error("This cannot be matched.")]
    MatchedValueIsNotValid {
        /// Common message describing which Sway types
        /// are currently supported in match expressions.
        supported_types_message: Vec<&'static str>,
        span: Span,
    },
    #[error(
        "Storage attribute access mismatch. Try giving the surrounding function more access by \
        adding \"#[{STORAGE_PURITY_ATTRIBUTE_NAME}({attrs})]\" to the function declaration."
    )]
    StorageAccessMismatch { attrs: String, span: Span },
    #[error(
        "The function \"{fn_name}\" in {interface_name} is pure, but this \
        implementation is not.  The \"{STORAGE_PURITY_ATTRIBUTE_NAME}\" annotation must be \
        removed, or the trait declaration must be changed to \
        \"#[{STORAGE_PURITY_ATTRIBUTE_NAME}({attrs})]\"."
    )]
    TraitDeclPureImplImpure {
        fn_name: Ident,
        interface_name: InterfaceName,
        attrs: String,
        span: Span,
    },
    #[error(
        "Storage attribute access mismatch. The function \"{fn_name}\" in \
        {interface_name} requires the storage attribute(s) #[{STORAGE_PURITY_ATTRIBUTE_NAME}({attrs})]."
    )]
    TraitImplPurityMismatch {
        fn_name: Ident,
        interface_name: InterfaceName,
        attrs: String,
        span: Span,
    },
    #[error("Impure function inside of non-contract. Contract storage is only accessible from contracts.")]
    ImpureInNonContract { span: Span },
    #[error(
        "This function performs a storage {storage_op} but does not have the required \
        attribute(s).  Try adding \"#[{STORAGE_PURITY_ATTRIBUTE_NAME}({attrs})]\" to the function \
        declaration."
    )]
    ImpureInPureContext {
        storage_op: &'static str,
        attrs: String,
        span: Span,
    },
    #[error(
        "Parameter reference type or mutability mismatch between the trait function declaration and its implementation."
    )]
    ParameterRefMutabilityMismatch { span: Span },
    #[error("Literal value is too large for type {ty}.")]
    IntegerTooLarge { span: Span, ty: String },
    #[error("Literal value underflows type {ty}.")]
    IntegerTooSmall { span: Span, ty: String },
    #[error("Literal value contains digits which are not valid for type {ty}.")]
    IntegerContainsInvalidDigit { span: Span, ty: String },
    #[error("A trait cannot be a subtrait of an ABI.")]
    AbiAsSupertrait { span: Span },
    #[error(
        "Implementation of trait \"{supertrait_name}\" is required by this bound in \"{trait_name}\""
    )]
    SupertraitImplRequired {
        supertrait_name: String,
        trait_name: Ident,
        span: Span,
    },
    #[error(
        "Contract ABI method parameter \"{param_name}\" is set multiple times for this contract ABI method call"
    )]
    ContractCallParamRepeated { param_name: String, span: Span },
    #[error(
        "Unrecognized contract ABI method parameter \"{param_name}\". The only available parameters are \"gas\", \"coins\", and \"asset_id\""
    )]
    UnrecognizedContractParam { param_name: String, span: Span },
    #[error("Attempting to specify a contract method parameter for a non-contract function call")]
    CallParamForNonContractCallMethod { span: Span },
    #[error("Storage field \"{field_name}\" does not exist.")]
    StorageFieldDoesNotExist {
        field_name: IdentUnique,
        available_fields: Vec<Ident>,
        storage_decl_span: Span,
    },
    #[error("No storage has been declared")]
    NoDeclaredStorage { span: Span },
    #[error("Multiple storage declarations were found")]
    MultipleStorageDeclarations { span: Span },
    #[error("Type {ty} can only be declared directly as a storage field")]
    InvalidStorageOnlyTypeDecl { ty: String, span: Span },
    #[error(
        "Internal compiler error: Unexpected {decl_type} declaration found.\n\
        Please file an issue on the repository and include the code that triggered this error."
    )]
    UnexpectedDeclaration { decl_type: &'static str, span: Span },
    #[error("This contract caller has no known address. Try instantiating a contract caller with a known contract address instead.")]
    ContractAddressMustBeKnown { span: Span },
    #[error("{}", error)]
    ConvertParseTree {
        #[from]
        error: ConvertParseTreeError,
    },
    #[error("{}", error)]
    Lex { error: LexError },
    #[error("{}", error)]
    Parse { error: ParseError },
    #[error("Could not evaluate initializer to a const declaration.")]
    NonConstantDeclValue { span: Span },
    #[error("Declaring storage in a {program_kind} is not allowed.")]
    StorageDeclarationInNonContract { program_kind: String, span: Span },
    #[error("Unsupported argument type to intrinsic \"{name}\".{}", if hint.is_empty() { "".to_string() } else { format!(" Hint: {hint}") })]
    IntrinsicUnsupportedArgType {
        name: String,
        span: Span,
        hint: String,
    },
    #[error("Call to \"{name}\" expects {expected} arguments")]
    IntrinsicIncorrectNumArgs {
        name: String,
        expected: u64,
        span: Span,
    },
    #[error("Call to \"{name}\" expects {expected} type arguments")]
    IntrinsicIncorrectNumTArgs {
        name: String,
        expected: u64,
        span: Span,
    },
    #[error("Expected string literal")]
    ExpectedStringLiteral { span: Span },
    #[error("\"break\" used outside of a loop")]
    BreakOutsideLoop { span: Span },
    #[error("\"continue\" used outside of a loop")]
    ContinueOutsideLoop { span: Span },
    /// This will be removed once loading contract IDs in a dependency namespace is refactored and no longer manual:
    /// https://github.com/FuelLabs/sway/issues/3077
    #[error("Contract ID is not a constant item.")]
    ContractIdConstantNotAConstDecl { span: Span },
    /// This will be removed once loading contract IDs in a dependency namespace is refactored and no longer manual:
    /// https://github.com/FuelLabs/sway/issues/3077
    #[error("Contract ID value is not a literal.")]
    ContractIdValueNotALiteral { span: Span },

    #[error("{reason}")]
    TypeNotAllowed {
        reason: TypeNotAllowedReason,
        span: Span,
    },
    #[error("ref mut parameter not allowed for main()")]
    RefMutableNotAllowedInMain { param_name: Ident, span: Span },
    #[error(
        "Register \"{name}\" is initialized and later reassigned which is not allowed. \
            Consider assigning to a different register inside the ASM block."
    )]
    InitializedRegisterReassignment { name: String, span: Span },
    #[error("Control flow VM instructions are not allowed in assembly blocks.")]
    DisallowedControlFlowInstruction { name: String, span: Span },
    #[error("Calling private library method {name} is not allowed.")]
    CallingPrivateLibraryMethod { name: String, span: Span },
    #[error("Using intrinsic \"{intrinsic}\" in a predicate is not allowed.")]
    DisallowedIntrinsicInPredicate { intrinsic: String, span: Span },
    #[error("Possibly non-zero amount of coins transferred to non-payable contract method \"{fn_name}\".")]
    CoinsPassedToNonPayableMethod { fn_name: Ident, span: Span },
    #[error(
        "Payable attribute mismatch. The \"{fn_name}\" method implementation \
         {} in its signature in {interface_name}.",
        if *missing_impl_attribute {
            "is missing #[payable] attribute specified"
        } else {
            "has extra #[payable] attribute not mentioned"
        }
    )]
    TraitImplPayabilityMismatch {
        fn_name: Ident,
        interface_name: InterfaceName,
        missing_impl_attribute: bool,
        span: Span,
    },
    #[error("Configurable constants are not allowed in libraries.")]
    ConfigurableInLibrary { span: Span },
    #[error("Multiple applicable items in scope. {}", {
        let mut candidates = "".to_string();
        let mut as_traits = as_traits.clone();
        // Make order deterministic
        as_traits.sort_by_key(|a| a.to_lowercase());
        for (index, as_trait) in as_traits.iter().enumerate() {
            candidates = format!("{candidates}\n  Disambiguate the associated {item_kind} for candidate #{index}\n    <{type_name} as {as_trait}>::{item_name}");
        }
        candidates
    })]
    MultipleApplicableItemsInScope {
        span: Span,
        type_name: String,
        item_name: String,
        item_kind: String,
        as_traits: Vec<String>,
    },
    #[error("Provided generic type is not of type str.")]
    NonStrGenericType { span: Span },
    #[error("A contract method cannot call methods belonging to the same ABI")]
    ContractCallsItsOwnMethod { span: Span },
    #[error("ABI cannot define a method with the same name as its super-ABI \"{superabi}\"")]
    AbiShadowsSuperAbiMethod { span: Span, superabi: Ident },
    #[error("ABI cannot inherit samely named method (\"{method_name}\") from several super-ABIs: \"{superabi1}\" and \"{superabi2}\"")]
    ConflictingSuperAbiMethods {
        span: Span,
        method_name: String,
        superabi1: String,
        superabi2: String,
    },
    #[error("Associated types not supported in ABI.")]
    AssociatedTypeNotSupportedInAbi { span: Span },
    #[error("Cannot call ABI supertrait's method as a contract method: \"{fn_name}\"")]
    AbiSupertraitMethodCallAsContractCall { fn_name: Ident, span: Span },
    #[error("{invalid_type} is not a valid type in the self type of an impl block.")]
    TypeIsNotValidAsImplementingFor {
        invalid_type: InvalidImplementingForType,
        /// Name of the trait if the impl implements a trait, `None` otherwise.
        trait_name: Option<String>,
        span: Span,
    },
    #[error("Uninitialized register is being read before being written")]
    UninitRegisterInAsmBlockBeingRead { span: Span },
    #[error("Expression of type \"{expression_type}\" cannot be dereferenced.")]
    ExpressionCannotBeDereferenced { expression_type: String, span: Span },
    #[error("Fallback functions can only exist in contracts")]
    FallbackFnsAreContractOnly { span: Span },
    #[error("Fallback functions cannot have parameters")]
    FallbackFnsCannotHaveParameters { span: Span },
    #[error("Could not generate the entry method. See errors above for more details.")]
    CouldNotGenerateEntry { span: Span },
    #[error("Missing `core` in dependencies.")]
    CouldNotGenerateEntryMissingCore { span: Span },
    #[error("Type \"{ty}\" does not implement AbiEncode or AbiDecode.")]
    CouldNotGenerateEntryMissingImpl { ty: String, span: Span },
    #[error("Only bool, u8, u16, u32, u64, u256, b256, string arrays and string slices can be used here.")]
    EncodingUnsupportedType { span: Span },
}

impl std::convert::From<TypeError> for CompileError {
    fn from(other: TypeError) -> CompileError {
        CompileError::TypeError(other)
    }
}

impl Spanned for CompileError {
    fn span(&self) -> Span {
        use CompileError::*;
        match self {
            ModuleDepGraphEvaluationError { .. } => Span::dummy(),
            ModuleDepGraphCyclicReference { .. } => Span::dummy(),
            UnknownVariable { span, .. } => span.clone(),
            NotAVariable { span, .. } => span.clone(),
            Unimplemented { span, .. } => span.clone(),
            TypeError(err) => err.span(),
            ParseError { span, .. } => span.clone(),
            Internal(_, span) => span.clone(),
            InternalOwned(_, span) => span.clone(),
            NoPredicateMainFunction(span) => span.clone(),
            PredicateMainDoesNotReturnBool(span) => span.clone(),
            NoScriptMainFunction(span) => span.clone(),
            MultipleDefinitionsOfFunction { span, .. } => span.clone(),
            MultipleDefinitionsOfName { span, .. } => span.clone(),
            MultipleDefinitionsOfConstant { span, .. } => span.clone(),
            MultipleDefinitionsOfType { span, .. } => span.clone(),
            MultipleDefinitionsOfMatchArmVariable { duplicate, .. } => duplicate.clone(),
            MultipleDefinitionsOfFallbackFunction { span, .. } => span.clone(),
            AssignmentToNonMutableVariable { lhs_span, .. } => lhs_span.clone(),
            AssignmentToConstantOrConfigurable { lhs_span, .. } => lhs_span.clone(),
            DeclAssignmentTargetCannotBeAssignedTo { lhs_span, .. } => lhs_span.clone(),
            AssignmentViaNonMutableReference { span, .. } => span.clone(),
            MutableParameterNotSupported { span, .. } => span.clone(),
            ImmutableArgumentToMutableParameter { span } => span.clone(),
            RefMutableNotAllowedInContractAbi { span, .. } => span.clone(),
            RefMutCannotReferenceConstant { span, .. } => span.clone(),
            RefMutCannotReferenceImmutableVariable { span, .. } => span.clone(),
            MethodRequiresMutableSelf { span, .. } => span.clone(),
            AssociatedFunctionCalledAsMethod { span, .. } => span.clone(),
            TypeParameterNotInTypeScope { span, .. } => span.clone(),
            MismatchedTypeInInterfaceSurface { span, .. } => span.clone(),
            UnknownTrait { span, .. } => span.clone(),
            FunctionNotAPartOfInterfaceSurface { span, .. } => span.clone(),
            ConstantNotAPartOfInterfaceSurface { span, .. } => span.clone(),
            TypeNotAPartOfInterfaceSurface { span, .. } => span.clone(),
            MissingInterfaceSurfaceConstants { span, .. } => span.clone(),
            MissingInterfaceSurfaceTypes { span, .. } => span.clone(),
            MissingInterfaceSurfaceMethods { span, .. } => span.clone(),
            IncorrectNumberOfTypeArguments { span, .. } => span.clone(),
            DoesNotTakeTypeArguments { span, .. } => span.clone(),
            DoesNotTakeTypeArgumentsAsPrefix { span, .. } => span.clone(),
            TypeArgumentsNotAllowed { span } => span.clone(),
            NeedsTypeArguments { span, .. } => span.clone(),
            StructInstantiationMissingFieldForErrorRecovery { span, .. } => span.clone(),
            StructInstantiationMissingFields { span, .. } => span.clone(),
            StructCannotBeInstantiated { span, .. } => span.clone(),
            StructFieldIsPrivate { field_name, .. } => field_name.span(),
            StructFieldDoesNotExist { field_name, .. } => field_name.span(),
            MethodNotFound { span, .. } => span.clone(),
            ModuleNotFound { span, .. } => span.clone(),
            TupleElementAccessOnNonTuple { span, .. } => span.clone(),
            NotAStruct { span, .. } => span.clone(),
            NotIndexable { span, .. } => span.clone(),
            FieldAccessOnNonStruct { span, .. } => span.clone(),
            SymbolNotFound { span, .. } => span.clone(),
            SymbolWithMultipleBindings { span, .. } => span.clone(),
            ImportPrivateSymbol { span, .. } => span.clone(),
            ImportPrivateModule { span, .. } => span.clone(),
            NoElseBranch { span, .. } => span.clone(),
            NotAType { span, .. } => span.clone(),
            MissingEnumInstantiator { span, .. } => span.clone(),
            PathDoesNotReturn { span, .. } => span.clone(),
            ExpectedModuleDocComment { span } => span.clone(),
            UnknownRegister { span, .. } => span.clone(),
            MissingImmediate { span, .. } => span.clone(),
            InvalidImmediateValue { span, .. } => span.clone(),
            UnknownEnumVariant { span, .. } => span.clone(),
            UnrecognizedOp { span, .. } => span.clone(),
            UnableToInferGeneric { span, .. } => span.clone(),
            UnconstrainedGenericParameter { span, .. } => span.clone(),
            TraitConstraintNotSatisfied { span, .. } => span.clone(),
            TraitConstraintMissing { span, .. } => span.clone(),
            Immediate06TooLarge { span, .. } => span.clone(),
            Immediate12TooLarge { span, .. } => span.clone(),
            Immediate18TooLarge { span, .. } => span.clone(),
            Immediate24TooLarge { span, .. } => span.clone(),
            IncorrectNumberOfAsmRegisters { span, .. } => span.clone(),
            UnnecessaryImmediate { span, .. } => span.clone(),
            AmbiguousPath { span } => span.clone(),
            ModulePathIsNotAnExpression { span, .. } => span.clone(),
            UnknownType { span, .. } => span.clone(),
            UnknownTypeName { span, .. } => span.clone(),
            FileCouldNotBeRead { span, .. } => span.clone(),
            ImportMustBeLibrary { span, .. } => span.clone(),
            MoreThanOneEnumInstantiator { span, .. } => span.clone(),
            UnnecessaryEnumInstantiator { span, .. } => span.clone(),
            UnitVariantWithParenthesesEnumInstantiator { span, .. } => span.clone(),
            TraitNotFound { span, .. } => span.clone(),
            TraitNotImportedAtFunctionApplication {
                function_call_site_span,
                ..
            } => function_call_site_span.clone(),
            InvalidExpressionOnLhs { span, .. } => span.clone(),
            TooManyArgumentsForFunction { span, .. } => span.clone(),
            TooFewArgumentsForFunction { span, .. } => span.clone(),
            MissingParenthesesForFunction { span, .. } => span.clone(),
            InvalidAbiType { span, .. } => span.clone(),
            NotAnAbi { span, .. } => span.clone(),
            ImplAbiForNonContract { span, .. } => span.clone(),
            ConflictingImplsForTraitAndType {
                second_impl_span, ..
            } => second_impl_span.clone(),
            DuplicateDeclDefinedForType { span, .. } => span.clone(),
            IncorrectNumberOfInterfaceSurfaceFunctionParameters { span, .. } => span.clone(),
            ArgumentParameterTypeMismatch { span, .. } => span.clone(),
            RecursiveCall { span, .. } => span.clone(),
            RecursiveCallChain { span, .. } => span.clone(),
            RecursiveType { span, .. } => span.clone(),
            RecursiveTypeChain { span, .. } => span.clone(),
            GMFromExternalContext { span, .. } => span.clone(),
            MintFromExternalContext { span, .. } => span.clone(),
            BurnFromExternalContext { span, .. } => span.clone(),
            ContractStorageFromExternalContext { span, .. } => span.clone(),
            InvalidOpcodeFromPredicate { span, .. } => span.clone(),
            ArrayOutOfBounds { span, .. } => span.clone(),
            ConstantsCannotBeShadowed { name, .. } => name.span(),
            ConstantShadowsVariable { name, .. } => name.span(),
            ShadowsOtherSymbol { name } => name.span(),
            GenericShadowsGeneric { name } => name.span(),
            MatchExpressionNonExhaustive { span, .. } => span.clone(),
            MatchStructPatternMissingFields { span, .. } => span.clone(),
            MatchStructPatternMustIgnorePrivateFields { span, .. } => span.clone(),
            MatchArmVariableNotDefinedInAllAlternatives { variable, .. } => variable.span(),
            MatchArmVariableMismatchedType { variable, .. } => variable.span(),
            MatchedValueIsNotValid { span, .. } => span.clone(),
            NotAnEnum { span, .. } => span.clone(),
            StorageAccessMismatch { span, .. } => span.clone(),
            TraitDeclPureImplImpure { span, .. } => span.clone(),
            TraitImplPurityMismatch { span, .. } => span.clone(),
            DeclIsNotAnEnum { span, .. } => span.clone(),
            DeclIsNotAStruct { span, .. } => span.clone(),
            DeclIsNotAFunction { span, .. } => span.clone(),
            DeclIsNotAVariable { span, .. } => span.clone(),
            DeclIsNotAnAbi { span, .. } => span.clone(),
            DeclIsNotATrait { span, .. } => span.clone(),
            DeclIsNotAnImplTrait { span, .. } => span.clone(),
            DeclIsNotATraitFn { span, .. } => span.clone(),
            DeclIsNotStorage { span, .. } => span.clone(),
            DeclIsNotAConstant { span, .. } => span.clone(),
            DeclIsNotATypeAlias { span, .. } => span.clone(),
            ImpureInNonContract { span, .. } => span.clone(),
            ImpureInPureContext { span, .. } => span.clone(),
            ParameterRefMutabilityMismatch { span, .. } => span.clone(),
            IntegerTooLarge { span, .. } => span.clone(),
            IntegerTooSmall { span, .. } => span.clone(),
            IntegerContainsInvalidDigit { span, .. } => span.clone(),
            AbiAsSupertrait { span, .. } => span.clone(),
            SupertraitImplRequired { span, .. } => span.clone(),
            ContractCallParamRepeated { span, .. } => span.clone(),
            UnrecognizedContractParam { span, .. } => span.clone(),
            CallParamForNonContractCallMethod { span, .. } => span.clone(),
            StorageFieldDoesNotExist { field_name, .. } => field_name.span(),
            InvalidStorageOnlyTypeDecl { span, .. } => span.clone(),
            NoDeclaredStorage { span, .. } => span.clone(),
            MultipleStorageDeclarations { span, .. } => span.clone(),
            UnexpectedDeclaration { span, .. } => span.clone(),
            ContractAddressMustBeKnown { span, .. } => span.clone(),
            ConvertParseTree { error } => error.span(),
            Lex { error } => error.span(),
            Parse { error } => error.span.clone(),
            EnumNotFound { span, .. } => span.clone(),
            TupleIndexOutOfBounds { span, .. } => span.clone(),
            NonConstantDeclValue { span, .. } => span.clone(),
            StorageDeclarationInNonContract { span, .. } => span.clone(),
            IntrinsicUnsupportedArgType { span, .. } => span.clone(),
            IntrinsicIncorrectNumArgs { span, .. } => span.clone(),
            IntrinsicIncorrectNumTArgs { span, .. } => span.clone(),
            BreakOutsideLoop { span } => span.clone(),
            ContinueOutsideLoop { span } => span.clone(),
            ContractIdConstantNotAConstDecl { span } => span.clone(),
            ContractIdValueNotALiteral { span } => span.clone(),
            RefMutableNotAllowedInMain { span, .. } => span.clone(),
            InitializedRegisterReassignment { span, .. } => span.clone(),
            DisallowedControlFlowInstruction { span, .. } => span.clone(),
            CallingPrivateLibraryMethod { span, .. } => span.clone(),
            DisallowedIntrinsicInPredicate { span, .. } => span.clone(),
            CoinsPassedToNonPayableMethod { span, .. } => span.clone(),
            TraitImplPayabilityMismatch { span, .. } => span.clone(),
            ConfigurableInLibrary { span } => span.clone(),
            MultipleApplicableItemsInScope { span, .. } => span.clone(),
            NonStrGenericType { span } => span.clone(),
            CannotBeEvaluatedToConst { span } => span.clone(),
            ContractCallsItsOwnMethod { span } => span.clone(),
            AbiShadowsSuperAbiMethod { span, .. } => span.clone(),
            ConflictingSuperAbiMethods { span, .. } => span.clone(),
            AssociatedTypeNotSupportedInAbi { span, .. } => span.clone(),
            AbiSupertraitMethodCallAsContractCall { span, .. } => span.clone(),
            TypeNotAllowed { span, .. } => span.clone(),
            ExpectedStringLiteral { span } => span.clone(),
            TypeIsNotValidAsImplementingFor { span, .. } => span.clone(),
            UninitRegisterInAsmBlockBeingRead { span } => span.clone(),
            ExpressionCannotBeDereferenced { span, .. } => span.clone(),
            FallbackFnsAreContractOnly { span } => span.clone(),
            FallbackFnsCannotHaveParameters { span } => span.clone(),
            CouldNotGenerateEntry { span } => span.clone(),
            CouldNotGenerateEntryMissingCore { span } => span.clone(),
            CouldNotGenerateEntryMissingImpl { span, .. } => span.clone(),
            CannotBeEvaluatedToConfigurableSizeUnknown { span } => span.clone(),
            EncodingUnsupportedType { span } => span.clone(),
        }
    }
}

// When implementing diagnostics, follow these two guidelines outlined in the Expressive Diagnostics RFC:
// - Guide-level explanation: https://github.com/FuelLabs/sway-rfcs/blob/master/rfcs/0011-expressive-diagnostics.md#guide-level-explanation
// - Wording guidelines: https://github.com/FuelLabs/sway-rfcs/blob/master/rfcs/0011-expressive-diagnostics.md#wording-guidelines
// For concrete examples, look at the existing diagnostics.
impl ToDiagnostic for CompileError {
    fn to_diagnostic(&self, source_engine: &SourceEngine) -> Diagnostic {
        let code = Code::semantic_analysis;
        use CompileError::*;
        match self {
            ConstantsCannotBeShadowed { variable_or_constant, name, constant_span, constant_decl, is_alias } => Diagnostic {
                reason: Some(Reason::new(code(1), "Constants cannot be shadowed".to_string())),
                // NOTE: Issue level should actually be the part of the reason. But it would complicate handling of labels in the transitional
                //       period when we still have "old-style" diagnostics.
                //       Let's leave it like this, refactoring at the moment does not pay of.
                //       And our #[error] macro will anyhow encapsulate it and ensure consistency.
                issue: Issue::error(
                    source_engine,
                    name.span(),
                    format!(
                        // Variable "x" shadows constant with the same name
                        //  or
                        // Constant "x" shadows imported constant with the same name
                        //  or
                        // ...
                        "{variable_or_constant} \"{name}\" shadows {}constant of the same name.",
                        if constant_decl.clone() != Span::dummy() { "imported " } else { "" }
                    )
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        constant_span.clone(),
                        format!(
                            // Constant "x" is declared here.
                            //  or
                            // Constant "x" gets imported here.
                            "Shadowed constant \"{name}\" {} here{}.",
                            if constant_decl.clone() != Span::dummy() { "gets imported" } else { "is declared" },
                            if *is_alias { " as alias" } else { "" }
                        )
                    ),
                    Hint::info( // Ignored if the constant_decl is Span::dummy().
                        source_engine,
                        constant_decl.clone(),
                        format!("This is the original declaration of the imported constant \"{name}\".")
                    ),
                ],
                help: vec![
                    format!("Unlike variables, constants cannot be shadowed by other constants or variables."),
                    match (variable_or_constant.as_str(), constant_decl.clone() != Span::dummy()) {
                        ("Variable", false) => format!("Consider renaming either the variable \"{name}\" or the constant \"{name}\"."),
                        ("Constant", false) => "Consider renaming one of the constants.".to_string(),
                        (variable_or_constant, true) => format!(
                            "Consider renaming the {} \"{name}\" or using {} for the imported constant.",
                            variable_or_constant.to_lowercase(),
                            if *is_alias { "a different alias" } else { "an alias" }
                        ),
                        _ => unreachable!("We can have only the listed combinations: variable/constant shadows a non imported/imported constant.")
                    }
                ],
            },
            ConstantShadowsVariable { name , variable_span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Constants cannot shadow variables".to_string())),
                issue: Issue::error(
                    source_engine,
                    name.span(),
                    format!("Constant \"{name}\" shadows variable of the same name.")
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        variable_span.clone(),
                        format!("This is the shadowed variable \"{name}\".")
                    ),
                ],
                help: vec![
                    format!("Variables can shadow other variables, but constants cannot."),
                    format!("Consider renaming either the variable or the constant."),
                ],
            },
            MultipleDefinitionsOfMatchArmVariable { match_value, match_type, first_definition, first_definition_is_struct_field, duplicate, duplicate_is_struct_field } => Diagnostic {
                reason: Some(Reason::new(code(1), "Match pattern variable is already defined".to_string())),
                issue: Issue::error(
                    source_engine,
                    duplicate.clone(),
                    format!("Variable \"{}\" is already defined in this match arm.", first_definition.as_str())
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        if *duplicate_is_struct_field {
                            duplicate.clone()
                        }
                        else {
                            Span::dummy()
                        },
                        format!("Struct field \"{0}\" is just a shorthand notation for `{0}: {0}`. It defines a variable \"{0}\".", first_definition.as_str())
                    ),
                    Hint::info(
                        source_engine,
                        first_definition.clone(),
                        format!(
                            "This {}is the first definition of the variable \"{}\".",
                            if *first_definition_is_struct_field {
                                format!("struct field \"{}\" ", first_definition.as_str())
                            }
                            else {
                                "".to_string()
                            },
                            first_definition.as_str(),
                        )
                    ),
                    Hint::help(
                        source_engine,
                        if *first_definition_is_struct_field && !*duplicate_is_struct_field {
                            first_definition.clone()
                        }
                        else {
                            Span::dummy()
                        },
                        format!("Struct field \"{0}\" is just a shorthand notation for `{0}: {0}`. It defines a variable \"{0}\".", first_definition.as_str()),
                    ),
                    Hint::info(
                        source_engine,
                        match_value.clone(),
                        format!("The expression to match on is of type \"{match_type}\".")
                    ),
                ],
                help: vec![
                    format!("Variables used in match arm patterns must be unique within a pattern, except in alternatives."),
                    match (*first_definition_is_struct_field, *duplicate_is_struct_field) {
                        (true, true) => format!("Consider declaring a variable with different name for either of the fields. E.g., `{0}: var_{0}`.", first_definition.as_str()),
                        (true, false) | (false, true) => format!("Consider declaring a variable for the field \"{0}\" (e.g., `{0}: var_{0}`), or renaming the variable \"{0}\".", first_definition.as_str()),
                        (false, false) => "Consider renaming either of the variables.".to_string(),
                    },
                ],
            },
            MatchArmVariableMismatchedType { match_value, match_type, variable, first_definition, expected, received } => Diagnostic {
                reason: Some(Reason::new(code(1), "Match pattern variable has mismatched type".to_string())),
                issue: Issue::error(
                    source_engine,
                    variable.span(),
                    format!("Variable \"{variable}\" is expected to be of type \"{expected}\", but is \"{received}\".")
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        first_definition.clone(),
                        format!("\"{variable}\" is first defined here with type \"{expected}\".")
                    ),
                    Hint::info(
                        source_engine,
                        match_value.clone(),
                        format!("The expression to match on is of type \"{match_type}\".")
                    ),
                ],
                help: vec![
                    format!("In the same match arm, a variable must have the same type in all alternatives."),
                ],
            },
            MatchArmVariableNotDefinedInAllAlternatives { match_value, match_type, variable, missing_in_alternatives} => Diagnostic {
                reason: Some(Reason::new(code(1), "Match pattern variable is not defined in all alternatives".to_string())),
                issue: Issue::error(
                    source_engine,
                    variable.span(),
                    format!("Variable \"{variable}\" is not defined in all alternatives.")
                ),
                hints: {
                    let mut hints = vec![
                        Hint::info(
                            source_engine,
                            match_value.clone(),
                            format!("The expression to match on is of type \"{match_type}\".")
                        ),
                    ];

                    for (i, alternative) in missing_in_alternatives.iter().enumerate() {
                        hints.push(
                            Hint::info(
                                source_engine,
                                alternative.clone(),
                                format!("\"{variable}\" is {}missing in this alternative.", if i != 0 { "also " } else { "" }),
                            )
                        )
                    }

                    hints
                },
                help: vec![
                    format!("Consider removing the variable \"{variable}\" altogether, or adding it to all alternatives."),
                ],
            },
            MatchStructPatternMissingFields { missing_fields, missing_fields_are_public, struct_name, struct_decl_span, total_number_of_fields, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Struct pattern has missing fields".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("Struct pattern is missing the {}field{} {}.",
                        if *missing_fields_are_public { "public " } else { "" },
                        plural_s(missing_fields.len()),
                        sequence_to_str(missing_fields, Enclosing::DoubleQuote, 2)
                    )
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        span.clone(),
                        "Struct pattern must either contain or ignore each struct field.".to_string()
                    ),
                    Hint::info(
                        source_engine,
                        struct_decl_span.clone(),
                        format!("Struct \"{struct_name}\" is declared here, and has {} field{}.",
                            number_to_str(*total_number_of_fields),
                            plural_s(*total_number_of_fields),
                        )
                    ),
                ],
                help: vec![
                    // Consider ignoring the field "x_1" by using the `_` pattern: `x_1: _`.
                    //  or
                    // Consider ignoring individual fields by using the `_` pattern. E.g, `x_1: _`.
                    format!("Consider ignoring {} field{} {}by using the `_` pattern{} `{}: _`.",
                        singular_plural(missing_fields.len(), "the", "individual"),
                        plural_s(missing_fields.len()),
                        singular_plural(missing_fields.len(), &format!("\"{}\" ", missing_fields[0]), ""),
                        singular_plural(missing_fields.len(), ":", ". E.g.,"),
                        missing_fields[0]
                    ),
                    "Alternatively, consider ignoring all the missing fields by ending the struct pattern with `..`.".to_string(),
                ],
            },
            MatchStructPatternMustIgnorePrivateFields { private_fields, struct_name, struct_decl_span, all_fields_are_private, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Struct pattern must ignore inaccessible private fields".to_string())),
                issue: Issue::error(
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
            TraitNotImportedAtFunctionApplication { trait_name, function_name, function_call_site_span, trait_constraint_span, trait_candidates } => {
                // Make candidates order deterministic.
                let mut trait_candidates = trait_candidates.clone();
                trait_candidates.sort();
                let trait_candidates = &trait_candidates; // Remove mutability.

                Diagnostic {
                    reason: Some(Reason::new(code(1), "Trait is not imported".to_string())),
                    issue: Issue::error(
                        source_engine,
                        function_call_site_span.clone(),
                        format!(
                            "Trait \"{trait_name}\" is not imported {}when calling \"{function_name}\".",
                            get_file_name(source_engine, function_call_site_span.source_id())
                                .map_or("".to_string(), |file_name| format!("into \"{file_name}\" "))
                        )
                    ),
                    hints: {
                        let mut hints = vec![
                            Hint::help(
                                source_engine,
                                function_call_site_span.clone(),
                                format!("This import is needed because \"{function_name}\" requires \"{trait_name}\" in one of its trait constraints.")
                            ),
                            Hint::info(
                                source_engine,
                                trait_constraint_span.clone(),
                                format!("In the definition of \"{function_name}\", \"{trait_name}\" is used in this trait constraint.")
                            ),
                        ];

                        match trait_candidates.len() {
                            // If no candidates are found, that means that an alias was used in the trait constraint definition.
                            // The way how constraint checking works now, the trait will not be found when we try to check if
                            // the trait constraints are satisfied for type, and we will never end up in this case here.
                            // So we will simply ignore it.
                            0 => (),
                            // The most common case. Exactly one known trait with the given name.
                            1 => hints.push(Hint::help(
                                    source_engine,
                                    function_call_site_span.clone(),
                                    format!(
                                        "Import the \"{trait_name}\" trait {}by using: `use {};`.",
                                        get_file_name(source_engine, function_call_site_span.source_id())
                                            .map_or("".to_string(), |file_name| format!("into \"{file_name}\" ")),
                                        trait_candidates[0]
                                    )
                                )),
                            // Unlikely (for now) case of having several traits with the same name.
                            _ => hints.push(Hint::help(
                                    source_engine,
                                    function_call_site_span.clone(),
                                    format!(
                                        "To import the proper \"{trait_name}\" {}follow the detailed instructions given below.",
                                        get_file_name(source_engine, function_call_site_span.source_id())
                                            .map_or("".to_string(), |file_name| format!("into \"{file_name}\" "))
                                    )
                                )),
                        }

                        hints
                    },
                    help: {
                        let mut help = vec![];

                        if trait_candidates.len() > 1 {
                            help.push(format!("There are these {} traits with the name \"{trait_name}\" available in the modules:", number_to_str(trait_candidates.len())));
                            for trait_candidate in trait_candidates.iter() {
                                help.push(format!("{}- {trait_candidate}", Indent::Single));
                            }
                            help.push("To import the proper one follow these steps:".to_string());
                            help.push(format!(
                                "{}1. Look at the definition of the \"{function_name}\"{}.",
                                    Indent::Single,
                                    get_file_name(source_engine, trait_constraint_span.source_id())
                                        .map_or("".to_string(), |file_name| format!(" in the \"{file_name}\""))
                            ));
                            help.push(format!(
                                "{}2. Detect which exact \"{trait_name}\" is used in the trait constraint in the \"{function_name}\".",
                                Indent::Single
                            ));
                            help.push(format!(
                                "{}3. Import that \"{trait_name}\"{}.",
                                Indent::Single,
                                get_file_name(source_engine, function_call_site_span.source_id())
                                    .map_or("".to_string(), |file_name| format!(" into \"{file_name}\""))
                            ));
                            help.push(format!("{} E.g., assuming it is the first one on the list, use: `use {};`", Indent::Double, trait_candidates[0]));
                        }

                        help
                    },
                }
            },
            // TODO-IG: Extend error messages to pointers, once typed pointers are defined and can be dereferenced.
            ExpressionCannotBeDereferenced { expression_type, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Expression cannot be dereferenced".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("This expression cannot be dereferenced, because it is of type \"{expression_type}\", which is not a reference type.")
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        span.clone(),
                        "In Sway, only references can be dereferenced.".to_string()
                    ),
                    Hint::help(
                        source_engine,
                        span.clone(),
                        "Are you missing the reference operator `&` somewhere in the code?".to_string()
                    ),
                ],
                help: vec![],
            },
            StructInstantiationMissingFields { field_names, struct_name, span, struct_decl_span, total_number_of_fields } => Diagnostic {
                reason: Some(Reason::new(code(1), "Struct instantiation has missing fields".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("Instantiation of the struct \"{struct_name}\" is missing the field{} {}.",
                            plural_s(field_names.len()),
                            sequence_to_str(field_names, Enclosing::DoubleQuote, 2)
                        )
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        span.clone(),
                        "Struct instantiation must initialize all the fields of the struct.".to_string()
                    ),
                    Hint::info(
                        source_engine,
                        struct_decl_span.clone(),
                        format!("Struct \"{struct_name}\" is declared here, and has {} field{}.",
                            number_to_str(*total_number_of_fields),
                            plural_s(*total_number_of_fields),
                        )
                    ),
                ],
                help: vec![],
            },
            StructCannotBeInstantiated { struct_name, span, struct_decl_span, private_fields, constructors, all_fields_are_private, is_in_storage_declaration, struct_can_be_changed } => Diagnostic {
                reason: Some(Reason::new(code(1), "Struct cannot be instantiated due to inaccessible private fields".to_string())),
                issue: Issue::error(
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
            StructFieldIsPrivate { field_name, struct_name, field_decl_span, struct_can_be_changed, usage_context } => Diagnostic {
                reason: Some(Reason::new(code(1), "Private struct field is inaccessible".to_string())),
                issue: Issue::error(
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
                        format!("Private fields can only be {} within the module in which their struct is declared.",
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
            StructFieldDoesNotExist { field_name, available_fields, is_public_struct_access, struct_name, struct_decl_span, struct_is_empty, usage_context } => Diagnostic {
                reason: Some(Reason::new(code(1), "Struct field does not exist".to_string())),
                issue: Issue::error(
                    source_engine,
                    field_name.span(),
                    format!("Field \"{field_name}\" does not exist in the struct \"{struct_name}\".")
                ),
                hints: {
                    let public = if *is_public_struct_access { "public " } else { "" };

                    let (hint, show_struct_decl) = if *struct_is_empty {
                        (Some(format!("\"{struct_name}\" is an empty struct. It doesn't have any fields.")), false)
                    }
                    // If the struct anyhow cannot be instantiated (in the struct instantiation or storage declaration),
                    // we don't show any additional hints.
                    // Showing any available fields would be inconsistent and misleading, because they anyhow cannot be used.
                    // Besides, "Struct cannot be instantiated" error will provide all the explanations and suggestions.
                    else if (matches!(usage_context, StorageAccess) && *is_public_struct_access && available_fields.is_empty())
                            ||
                            (matches!(usage_context, StructInstantiation { struct_can_be_instantiated: false } | StorageDeclaration { struct_can_be_instantiated: false })) {
                        // If the struct anyhow cannot be instantiated in the storage, don't show any additional hint
                        // if there is an attempt to access a non existing field of such non-instantiable struct.
                        //   or
                        // Likewise, if we are in the struct instantiation or storage declaration and the struct
                        // cannot be instantiated.
                        (None, false)
                    } else if !available_fields.is_empty() {
                        // In all other cases, show the available fields.
                        const NUM_OF_FIELDS_TO_DISPLAY: usize = 4;
                        match &available_fields[..] {
                            [field] => (Some(format!("Only available {public}field is \"{field}\".")), false),
                            _ => (Some(format!("Available {public}fields are {}.", sequence_to_str(available_fields, Enclosing::DoubleQuote, NUM_OF_FIELDS_TO_DISPLAY))),
                                    available_fields.len() > NUM_OF_FIELDS_TO_DISPLAY
                                ),
                        }
                    }
                    else {
                        (None, false)
                    };

                    let mut hints = vec![];

                    if let Some(hint) = hint {
                        hints.push(Hint::help(source_engine, field_name.span(), hint));
                    };

                    if show_struct_decl {
                        hints.push(Hint::info(
                            source_engine,
                            struct_decl_span.clone(),
                            format!("Struct \"{struct_name}\" is declared here, and has {} {public}fields.",
                                number_to_str(available_fields.len())
                            )
                        ));
                    }

                    hints
                },
                help: vec![],
            },
            NotIndexable { actually, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Type is not indexable".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("This expression has type \"{actually}\", which is not an indexable type.")
                ),
                hints: vec![],
                help: vec![
                    "Index operator `[]` can be used only on indexable types.".to_string(),
                    "In Sway, indexable types are:".to_string(),
                    format!("{}- arrays. E.g., `[u64;3]`.", Indent::Single),
                    format!("{}- references, direct or indirect, to arrays. E.g., `&[u64;3]` or `&&&[u64;3]`.", Indent::Single),
                ],
            },
            FieldAccessOnNonStruct { actually, storage_variable, field_name, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Field access requires a struct".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("{} has type \"{actually}\", which is not a struct{}.",
                        if let Some(storage_variable) = storage_variable {
                            format!("Storage variable \"{storage_variable}\"")
                        } else {
                            "This expression".to_string()
                        },
                        if storage_variable.is_some() {
                            ""
                        } else {
                            " or a reference to a struct"
                        }
                    )
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        field_name.span(),
                        format!("Field access happens here, on \"{field_name}\".")
                    )
                ],
                help: if storage_variable.is_some() {
                    vec![
                        "Fields can only be accessed on storage variables that are structs.".to_string(),
                    ]
                } else {
                    vec![
                        "In Sway, fields can be accessed on:".to_string(),
                        format!("{}- structs. E.g., `my_struct.field`.", Indent::Single),
                        format!("{}- references, direct or indirect, to structs. E.g., `(&my_struct).field` or `(&&&my_struct).field`.", Indent::Single),
                    ]
                }
            },
	    SymbolWithMultipleBindings { name, paths, span } => Diagnostic {
		reason: Some(Reason::new(code(1), "Multiple bindings for symbol in this scope".to_string())),
		issue: Issue::error(
		    source_engine,
		    span.clone(),
		    format!("The following paths are all valid bindings for symbol \"{}\": {}", name, paths.iter().map(|path| format!("{path}::{name}")).collect::<Vec<_>>().join(", ")),
		),
		hints: paths.iter().map(|path| Hint::info(source_engine, Span::dummy(), format!("{path}::{}", name.as_str()))).collect(),
		help: vec![format!("Consider using a fully qualified name, e.g., {}::{}", paths[0], name.as_str())],
	    },
            StorageFieldDoesNotExist { field_name, available_fields, storage_decl_span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Storage field does not exist".to_string())),
                issue: Issue::error(
                    source_engine,
                    field_name.span(),
                    format!("Storage field \"{field_name}\" does not exist in the storage.")
                ),
                hints: {
                    let (hint, show_storage_decl) = if available_fields.is_empty() {
                        ("The storage is empty. It doesn't have any fields.".to_string(), false)
                    } else {
                        const NUM_OF_FIELDS_TO_DISPLAY: usize = 4;
                        match &available_fields[..] {
                            [field] => (format!("Only available storage field is \"{field}\"."), false),
                            _ => (format!("Available storage fields are {}.", sequence_to_str(available_fields, Enclosing::DoubleQuote, NUM_OF_FIELDS_TO_DISPLAY)),
                                    available_fields.len() > NUM_OF_FIELDS_TO_DISPLAY
                                ),
                        }
                    };

                    let mut hints = vec![];

                    hints.push(Hint::help(source_engine, field_name.span(), hint));

                    if show_storage_decl {
                        hints.push(Hint::info(
                            source_engine,
                            storage_decl_span.clone(),
                            format!("Storage is declared here, and has {} fields.",
                                number_to_str(available_fields.len())
                            )
                        ));
                    }

                    hints
                },
                help: vec![],
            },
            TupleIndexOutOfBounds { index, count, tuple_type, span, prefix_span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Tuple index is out of bounds".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("Tuple index {index} is out of bounds. The tuple has only {count} element{}.", plural_s(*count))
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        prefix_span.clone(),
                        format!("This expression has type \"{tuple_type}\".")
                    ),
                ],
                help: vec![],
            },
            TupleElementAccessOnNonTuple { actually, span, index, index_span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Tuple element access requires a tuple".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("This expression has type \"{actually}\", which is not a tuple or a reference to a tuple.")
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        index_span.clone(),
                        format!("Tuple element access happens here, on the index {index}.")
                    )
                ],
                help: vec![
                    "In Sway, tuple elements can be accessed on:".to_string(),
                    format!("{}- tuples. E.g., `my_tuple.1`.", Indent::Single),
                    format!("{}- references, direct or indirect, to tuples. E.g., `(&my_tuple).1` or `(&&&my_tuple).1`.", Indent::Single),
                ],
            },
            RefMutCannotReferenceConstant { constant, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "References to mutable values cannot reference constants".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("\"{constant}\" is a constant. `&mut` cannot reference constants.")
                ),
                hints: vec![],
                help: vec![
                    "Consider:".to_string(),
                    format!("{}- taking a reference without `mut`: `&{constant}`.", Indent::Single),
                    format!("{}- referencing a mutable copy of the constant, by returning it from a block: `&mut {{ {constant} }}`.", Indent::Single)
                ],
            },
            RefMutCannotReferenceImmutableVariable { decl_name, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "References to mutable values cannot reference immutable variables".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("\"{decl_name}\" is an immutable variable. `&mut` cannot reference immutable variables.")
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        decl_name.span(),
                        format!("Variable \"{decl_name}\" is declared here as immutable.")
                    ),
                ],
                help: vec![
                    "Consider:".to_string(),
                    // TODO-IG: Once desugaring information becomes available, do not show the first suggestion if declaring variable as mutable is not possible.
                    format!("{}- declaring \"{decl_name}\" as mutable.", Indent::Single),
                    format!("{}- taking a reference without `mut`: `&{decl_name}`.", Indent::Single),
                    format!("{}- referencing a mutable copy of \"{decl_name}\", by returning it from a block: `&mut {{ {decl_name} }}`.", Indent::Single)
                ],
            },
            ConflictingImplsForTraitAndType { trait_name, type_implementing_for, existing_impl_span, second_impl_span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Trait is already implemented for type".to_string())),
                issue: Issue::error(
                    source_engine,
                    second_impl_span.clone(),
                    format!("Trait \"{trait_name}\" is already implemented for type \"{type_implementing_for}\".")
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        existing_impl_span.clone(),
                        format!("This is the already existing implementation of \"{}\" for \"{type_implementing_for}\".",
                            call_path_suffix_with_args(trait_name)
                        )
                    ),
                ],
                help: vec![
                    "In Sway, there can be at most one implementation of a trait for any given type.".to_string(),
                    "This property is called \"trait coherence\".".to_string(),
                ],
            },
            AssignmentToNonMutableVariable { lhs_span, decl_name } => Diagnostic {
                reason: Some(Reason::new(code(1), "Immutable variables cannot be assigned to".to_string())),
                issue: Issue::error(
                    source_engine,
                    lhs_span.clone(),
                    // "x" cannot be assigned to, because it is an immutable variable.
                    //  or
                    // This expression cannot be assigned to, because "x" is an immutable variable.
                    format!("{} cannot be assigned to, because {} is an immutable variable.",
                        if decl_name.as_str() == lhs_span.as_str() { // We have just a single variable in the expression.
                            format!("\"{decl_name}\"")
                        } else {
                            "This expression".to_string()
                        },
                        if decl_name.as_str() == lhs_span.as_str() {
                            "it".to_string()
                        } else {
                            format!("\"{decl_name}\"")
                        }
                    )
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        decl_name.span(),
                        format!("Variable \"{decl_name}\" is declared here as immutable.")
                    ),
                ],
                help: vec![
                    // TODO-IG: Once desugaring information becomes available, do not show this suggestion if declaring variable as mutable is not possible.
                    format!("Consider declaring \"{decl_name}\" as mutable."),
                ],
            },
            AssignmentToConstantOrConfigurable { lhs_span, is_configurable, decl_name } => Diagnostic {
                reason: Some(Reason::new(code(1), format!("{} cannot be assigned to",
                    if *is_configurable {
                        "Configurables"
                    } else {
                        "Constants"
                    }
                ))),
                issue: Issue::error(
                    source_engine,
                    lhs_span.clone(),
                    // "x" cannot be assigned to, because it is a constant/configurable.
                    //  or
                    // This expression cannot be assigned to, because "x" is a constant/configurable.
                    format!("{} cannot be assigned to, because {} is a {}.",
                        if decl_name.as_str() == lhs_span.as_str() { // We have just the constant in the expression.
                            format!("\"{decl_name}\"")
                        } else {
                            "This expression".to_string()
                        },
                        if decl_name.as_str() == lhs_span.as_str() {
                            "it".to_string()
                        } else {
                            format!("\"{decl_name}\"")
                        },
                        if *is_configurable {
                            "configurable"
                        } else {
                            "constant"
                        }
                    )
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        decl_name.span(),
                        format!("{} \"{decl_name}\" is declared here.",
                            if *is_configurable {
                                "Configurable"
                            } else {
                                "Constant"
                            }
                        )
                    ),
                ],
                help: vec![],
            },
            DeclAssignmentTargetCannotBeAssignedTo { decl_name, decl_friendly_type_name, lhs_span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Assignment target cannot be assigned to".to_string())),
                issue: Issue::error(
                    source_engine,
                    lhs_span.clone(),
                    // "x" cannot be assigned to, because it is a trait/function/ etc and not a mutable variable.
                    //  or
                    // This cannot be assigned to, because "x" is a trait/function/ etc and not a mutable variable.
                    format!("{} cannot be assigned to, because {} is {}{decl_friendly_type_name} and not a mutable variable.",
                        match decl_name {
                            Some(decl_name) if decl_name.as_str() == lhs_span.as_str() => // We have just the decl name in the expression.
                                format!("\"{decl_name}\""),
                            _ => "This".to_string(),
                        },
                        match decl_name {
                            Some(decl_name) if decl_name.as_str() == lhs_span.as_str() =>
                                "it".to_string(),
                            Some(decl_name) => format!("\"{}\"", decl_name.as_str()),
                            _ => "it".to_string(),
                        },
                        a_or_an(decl_friendly_type_name)
                    )
                ),
                hints: vec![
                    match decl_name {
                        Some(decl_name) => Hint::info(
                            source_engine,
                            decl_name.span(),
                            format!("{} \"{decl_name}\" is declared here.", ascii_sentence_case(&decl_friendly_type_name.to_string()))
                        ),
                        _ => Hint::none(),
                    }
                ],
                help: vec![],
            },
            AssignmentViaNonMutableReference { decl_reference_name, decl_reference_rhs, decl_reference_type, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Reference is not a reference to a mutable value (`&mut`)".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    // This reference expression is not a reference to a mutable value (`&mut`).
                    //  or
                    // Reference "ref_xyz" is not a reference to a mutable value (`&mut`).
                    format!("{} is not a reference to a mutable value (`&mut`).",
                        match decl_reference_name {
                            Some(decl_reference_name) => format!("Reference \"{decl_reference_name}\""),
                            _ => "This reference expression".to_string(),
                        }
                    )
                ),
                hints: vec![
                    match decl_reference_name {
                        Some(decl_reference_name) => Hint::info(
                            source_engine,
                            decl_reference_name.span(),
                            format!("Reference \"{decl_reference_name}\" is declared here as a reference to immutable value.")
                        ),
                        _ => Hint::none(),
                    },
                    match decl_reference_rhs {
                        Some(decl_reference_rhs) => Hint::info(
                            source_engine,
                            decl_reference_rhs.clone(),
                            format!("This expression has type \"{decl_reference_type}\" instead of \"&mut {}\".",
                                &decl_reference_type[1..]
                            )
                        ),
                        _ => Hint::info(
                            source_engine,
                            span.clone(),
                            format!("It has type \"{decl_reference_type}\" instead of \"&mut {}\".",
                                &decl_reference_type[1..]
                            )
                        ),
                    },
                    match decl_reference_rhs {
                        Some(decl_reference_rhs) if decl_reference_rhs.as_str().starts_with('&') => Hint::help(
                            source_engine,
                            decl_reference_rhs.clone(),
                            format!("Consider taking here a reference to a mutable value: `&mut {}`.",
                                first_line(decl_reference_rhs.as_str()[1..].trim(), true)
                            )
                        ),
                        _ => Hint::none(),
                    },
                ],
                help: vec![
                    format!("{} dereferenced in assignment targets must {} references to mutable values (`&mut`).",
                        if decl_reference_name.is_some() {
                            "References"
                        } else {
                            "Reference expressions"
                        },
                        if decl_reference_name.is_some() {
                            "be"
                        } else {
                            "result in"
                        }
                    ),
                ],
            },
            Unimplemented { feature, help, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Used feature is currently not implemented".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("{feature} is currently not implemented.")
                ),
                hints: vec![],
                help: help.clone(),
            },
            MatchedValueIsNotValid { supported_types_message, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Matched value is not valid".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    "This cannot be matched.".to_string()
                ),
                hints: vec![],
                help: {
                    let mut help = vec![];

                    help.push("Matched value must be an expression whose result is of one of the types supported in pattern matching.".to_string());
                    help.push(Diagnostic::help_empty_line());
                    for msg in supported_types_message {
                        help.push(msg.to_string());
                    }

                    help
                }
            },
            TypeIsNotValidAsImplementingFor { invalid_type, trait_name, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Self type of an impl block is not valid".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    format!("{invalid_type} is not a valid type in the self type of {} impl block.",
                        match trait_name {
                            Some(_) => "a trait",
                            None => "an",
                        }
                    )
                ),
                hints: vec![
                    if matches!(invalid_type, InvalidImplementingForType::SelfType) {
                        Hint::help(
                            source_engine,
                            span.clone(),
                            format!("Replace {invalid_type} with the actual type that you want to implement for.")
                        )
                    } else {
                        Hint::none()
                    }
                ],
                help: {
                    if matches!(invalid_type, InvalidImplementingForType::Placeholder) {
                        vec![
                            format!("Are you trying to implement {} for any type?",
                                match trait_name {
                                    Some(trait_name) => format!("trait \"{trait_name}\""),
                                    None => "functionality".to_string(),
                                }
                            ),
                            Diagnostic::help_empty_line(),
                            "If so, use generic type parameters instead.".to_string(),
                            "E.g., instead of:".to_string(),
                            // The trait `trait_name` could represent an arbitrary complex trait.
                            // E.g., `with generic arguments, etc. So we don't want to deal
                            // with the complexity of representing it properly
                            // but rather use a simplified but clearly instructive
                            // sample trait name here, `SomeTrait`.
                            // impl _
                            //   or
                            // impl SomeTrait for _
                            format!("{}impl {}_",
                                Indent::Single,
                                match trait_name {
                                    Some(_) => "SomeTrait for ",
                                    None => "",
                                }
                            ),
                            "use:".to_string(),
                            format!("{}impl<T> {}T",
                                Indent::Single,
                                match trait_name {
                                    Some(_) => "SomeTrait for ",
                                    None => "",
                                }
                            ),
                        ]
                    } else {
                        vec![]
                    }
                }
            },
            ModulePathIsNotAnExpression { module_path, span } => Diagnostic {
                reason: Some(Reason::new(code(1), "Module path is not an expression".to_string())),
                issue: Issue::error(
                    source_engine,
                    span.clone(),
                    "This is a module path, and not an expression.".to_string()
                ),
                hints: vec![
                    Hint::help(
                        source_engine,
                        span.clone(),
                        "An expression is expected at this location, but a module path is found.".to_string()
                    ),
                ],
                help: vec![
                    "In expressions, module paths can only be used to fully qualify names with a path.".to_string(),
                    format!("E.g., `{module_path}::SOME_CONSTANT` or `{module_path}::some_function()`."),
                ]
            },
            Parse { error } => {
                match &error.kind {
                    ParseErrorKind::UnassignableExpression { erroneous_expression_kind, erroneous_expression_span } => Diagnostic {
                        reason: Some(Reason::new(code(1), "Expression cannot be assigned to".to_string())),
                        // A bit of a special handling for parentheses, because they are the only
                        // expression kind whose friendly name is in plural. Having it in singular
                        // or without this simple special handling gives very odd sounding sentences.
                        // Therefore, just a bit of a special handling.
                        issue: Issue::error(
                            source_engine,
                            error.span.clone(),
                            format!("This expression cannot be assigned to, because it {} {}{}.",
                                if &error.span == erroneous_expression_span { // If the whole expression is erroneous.
                                    "is"
                                } else {
                                    "contains"
                                },
                                if *erroneous_expression_kind == "parentheses" {
                                    ""
                                } else {
                                    a_or_an(erroneous_expression_kind)
                                },
                                erroneous_expression_kind
                            )
                        ),
                        hints: vec![
                            if &error.span != erroneous_expression_span {
                                Hint::info(
                                    source_engine,
                                    erroneous_expression_span.clone(),
                                    format!("{} the contained {erroneous_expression_kind}.",
                                        if *erroneous_expression_kind == "parentheses" {
                                            "These are"
                                        } else {
                                            "This is"
                                        }
                                    )
                                )
                            } else {
                                Hint::none()
                            },
                        ],
                        help: vec![
                            format!("{} cannot be {}an assignment target.",
                                ascii_sentence_case(&erroneous_expression_kind.to_string()),
                                if &error.span == erroneous_expression_span {
                                    ""
                                } else {
                                    "a part of "
                                }
                            ),
                            Diagnostic::help_empty_line(),
                            "In Sway, assignment targets must be one of the following:".to_string(),
                            format!("{}- Expressions starting with a mutable variable, optionally having", Indent::Single),
                            format!("{}  array or tuple element accesses, struct field accesses,", Indent::Single),
                            format!("{}  or arbitrary combinations of those.", Indent::Single),
                            format!("{}  E.g., `mut_var` or `mut_struct.field` or `mut_array[x + y].field.1`.", Indent::Single),
                            Diagnostic::help_empty_line(),
                            format!("{}- Dereferencing of an arbitrary expression that results", Indent::Single),
                            format!("{}  in a reference to a mutable value.", Indent::Single),
                            format!("{}  E.g., `*ref_to_mutable_value` or `*max_mut(&mut x, &mut y)`.", Indent::Single),
                        ]
                    },
                    _ => Diagnostic {
                                // TODO: Temporary we use self here to achieve backward compatibility.
                                //       In general, self must not be used and will not be used once we
                                //       switch to our own #[error] macro. All the values for the formatting
                                //       of a diagnostic must come from the enum variant parameters.
                                issue: Issue::error(source_engine, self.span(), format!("{}", self)),
                                ..Default::default()
                        },
                }
            },
           _ => Diagnostic {
                    // TODO: Temporary we use self here to achieve backward compatibility.
                    //       In general, self must not be used and will not be used once we
                    //       switch to our own #[error] macro. All the values for the formatting
                    //       of a diagnostic must come from the enum variant parameters.
                    issue: Issue::error(source_engine, self.span(), format!("{}", self)),
                    ..Default::default()
            }
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeNotAllowedReason {
    #[error(
        "Returning a type containing `raw_slice` from `main()` is not allowed. \
            Consider converting it into a flat `raw_slice` first."
    )]
    NestedSliceReturnNotAllowedInMain,

    #[error("The type \"{ty}\" is not allowed in storage.")]
    TypeNotAllowedInContractStorage { ty: String },

    #[error("`str` or a type containing `str` on `main()` arguments is not allowed.")]
    StringSliceInMainParameters,

    #[error("Returning `str` or a type containing `str` from `main()` is not allowed.")]
    StringSliceInMainReturn,

    #[error("`str` or a type containing `str` on `configurables` is not allowed.")]
    StringSliceInConfigurables,

    #[error("`str` or a type containing `str` on `const` is not allowed.")]
    StringSliceInConst,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StructFieldUsageContext {
    StructInstantiation { struct_can_be_instantiated: bool },
    StorageDeclaration { struct_can_be_instantiated: bool },
    StorageAccess,
    PatternMatching { has_rest_pattern: bool },
    StructFieldAccess,
    // TODO: Distinguish between struct field access and destructing
    //       once https://github.com/FuelLabs/sway/issues/5478 is implemented
    //       and provide specific suggestions for these two cases.
    //       (Destructing desugars to plain struct field access.)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InvalidImplementingForType {
    SelfType,
    Placeholder,
    Other,
}

impl fmt::Display for InvalidImplementingForType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvalidImplementingForType::SelfType => f.write_str("\"Self\""),
            InvalidImplementingForType::Placeholder => f.write_str("Placeholder `_`"),
            InvalidImplementingForType::Other => f.write_str("This"),
        }
    }
}
