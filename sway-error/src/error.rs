use crate::convert_parse_tree_error::ConvertParseTreeError;
use crate::diagnostic::{Code, Diagnostic, Hint, Issue, Reason, ToDiagnostic};
use crate::lex_error::LexError;
use crate::parser_error::ParseError;
use crate::type_error::TypeError;

use core::fmt;
use sway_types::constants::STORAGE_PURITY_ATTRIBUTE_NAME;
use sway_types::{Ident, MaybeSpanned, SourceEngine, Span, Spanned};
use thiserror::Error;

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

// TODO: since moving to using Idents instead of strings, there are a lot of redundant spans in
// this type.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompileError {
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariable { var_name: Ident, span: Span },
    #[error("Identifier \"{name}\" was used as a variable, but it is actually a {what_it_is}.")]
    NotAVariable {
        name: Ident,
        what_it_is: &'static str,
        span: Span,
    },
    #[error("Unimplemented feature: {0}")]
    Unimplemented(&'static str, Option<Span>),
    #[error(
        "Unimplemented feature: {0}\n\
         help: {1}.\n\
         "
    )]
    UnimplementedWithHelp(&'static str, &'static str, Span),
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
    InternalOwned(String, Option<Span>),
    #[error(
        "Predicate declaration contains no main function. Predicates require a main function."
    )]
    NoPredicateMainFunction(Span),
    #[error("A predicate's main function must return a boolean.")]
    PredicateMainDoesNotReturnBool(Span),
    #[error("Script declaration contains no main function. Scripts require a main function.")]
    NoScriptMainFunction(Span),
    #[error("Function \"{name}\" was already defined in scope.")]
    MultipleDefinitionsOfFunction { name: Ident, span: Span },
    #[error("Name \"{name}\" is defined multiple times.")]
    MultipleDefinitionsOfName { name: Ident, span: Span },
    #[error("Constant \"{name}\" was already defined in scope.")]
    MultipleDefinitionsOfConstant { name: Ident, span: Span },
    #[error("Assignment to immutable variable. Variable {name} is not declared as mutable.")]
    AssignmentToNonMutable { name: Ident, span: Span },
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
    #[error("Constants are missing from this trait implementation: {missing_constants}")]
    MissingInterfaceSurfaceConstants {
        missing_constants: String,
        span: Span,
    },
    #[error("Functions are missing from this trait implementation: {missing_functions}")]
    MissingInterfaceSurfaceMethods {
        missing_functions: String,
        span: Span,
    },
    #[error("Expected {} type {}, but instead found {}.", expected, if *expected == 1usize { "argument" } else { "arguments" }, given)]
    IncorrectNumberOfTypeArguments {
        given: usize,
        expected: usize,
        span: Span,
    },
    #[error("\"{name}\" does not take type arguments.")]
    DoesNotTakeTypeArguments { name: Ident, span: Span },
    #[error("\"{name}\" does not take type arguments as prefix.")]
    DoesNotTakeTypeArgumentsAsPrefix { name: Ident, span: Option<Span> },
    #[error("Type arguments are not allowed for this type.")]
    TypeArgumentsNotAllowed { span: Span },
    #[error("\"{name}\" needs type arguments.")]
    NeedsTypeArguments { name: Ident, span: Span },
    #[error(
        "Enum with name \"{name}\" could not be found in this scope. Perhaps you need to import \
         it?"
    )]
    EnumNotFound { name: Ident, span: Span },
    #[error("Initialization of struct \"{struct_name}\" is missing field \"{field_name}\".")]
    StructMissingField {
        field_name: Ident,
        struct_name: Ident,
        span: Span,
    },
    #[error("Struct \"{struct_name}\" does not have field \"{field_name}\".")]
    StructDoesNotHaveField {
        field_name: Ident,
        struct_name: Ident,
        span: Span,
    },
    #[error("No method named \"{method_name}\" found for type \"{type_name}\".")]
    MethodNotFound {
        method_name: Ident,
        type_name: String,
        span: Span,
    },
    #[error("Module \"{name}\" could not be found.")]
    ModuleNotFound { span: Span, name: String },
    #[error("This is a {actually}, not a struct. Fields can only be accessed on structs.")]
    FieldAccessOnNonStruct { actually: String, span: Span },
    #[error("\"{name}\" is a {actually}, not a tuple. Elements can only be access on tuples.")]
    NotATuple {
        name: String,
        span: Span,
        actually: String,
    },
    #[error("\"{name}\" is a {actually}, which is not an indexable expression.")]
    NotIndexable {
        name: String,
        span: Span,
        actually: String,
    },
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
    #[error(
        "Field \"{field_name}\" not found on struct \"{struct_name}\". Available fields are:\n \
         {available_fields}"
    )]
    FieldNotFound {
        field_name: Ident,
        available_fields: String,
        struct_name: Ident,
        span: Span,
    },
    #[error("Could not find symbol \"{name}\" in this scope.")]
    SymbolNotFound { name: Ident, span: Span },
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
    #[error("This expression is not valid on the left hand side of a reassignment.")]
    InvalidExpressionOnLhs { span: Span },
    #[error("This code cannot be evaluated to a constant")]
    CannotBeEvaluatedToConst { span: Span },
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
    #[error("Tuple index out of bounds; the arity is {count} but the index is {index}.")]
    TupleIndexOutOfBounds {
        index: usize,
        count: usize,
        span: Span,
    },
    #[error("Constants cannot be shadowed. {variable_or_constant} \"{name}\" shadows constant with the same name.")]
    ConstantsCannotBeShadowed {
        variable_or_constant: String,
        name: Ident,
        constant_span: Span,
        constant_decl: Option<Span>,
        is_alias: bool,
    },
    #[error("Constants cannot shadow variables. The constant \"{name}\" shadows variable with the same name.")]
    ConstantShadowsVariable { name: Ident, variable_span: Span },
    #[error("The imported symbol \"{name}\" shadows another symbol with the same name.")]
    ShadowsOtherSymbol { name: Ident },
    #[error("The name \"{name}\" is already used for a generic parameter in this scope.")]
    GenericShadowsGeneric { name: Ident },
    #[error("Non-exhaustive match expression. Missing patterns {missing_patterns}")]
    MatchExpressionNonExhaustive {
        missing_patterns: String,
        span: Span,
    },
    #[error("Pattern does not mention {}: {}",
        if missing_fields.len() == 1 { "field" } else { "fields" },
        missing_fields.join(", "))]
    MatchStructPatternMissingFields {
        missing_fields: Vec<String>,
        span: Span,
    },
    #[error("Variable \"{var}\" is not bound in all patterns")]
    MatchVariableNotBoundInAllPatterns { var: Ident, span: Span },
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
    #[error("Storage field {name} does not exist")]
    StorageFieldDoesNotExist { name: Ident, span: Span },
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
    #[error("The type \"{ty}\" is not allowed in storage.")]
    TypeNotAllowedInContractStorage { ty: String, span: Span },
    #[error("ref mut parameter not allowed for main()")]
    RefMutableNotAllowedInMain { param_name: Ident, span: Span },
    #[error(
        "Returning a type containing `raw_slice` from `main()` is not allowed. \
            Consider converting it into a flat `raw_slice` first."
    )]
    NestedSliceReturnNotAllowedInMain { span: Span },
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
        for (index, as_trait) in as_traits.iter().enumerate() {
            candidates = format!("{candidates}\n  Disambiguate the associated function for candidate #{index}\n    <{type_name} as {as_trait}>::{method_name}(");
        }
        candidates
    })]
    MultipleApplicableItemsInScope {
        span: Span,
        type_name: String,
        method_name: String,
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
    #[error("Cannot call ABI supertrait's method as a contract method: \"{fn_name}\"")]
    AbiSupertraitMethodCallAsContractCall { fn_name: Ident, span: Span },
}

impl std::convert::From<TypeError> for CompileError {
    fn from(other: TypeError) -> CompileError {
        CompileError::TypeError(other)
    }
}

impl MaybeSpanned for CompileError {
    fn try_span(&self) -> Option<Span> {
        use CompileError::*;
        match self {
            UnknownVariable { span, .. } => Some(span.clone()),
            NotAVariable { span, .. } => Some(span.clone()),
            Unimplemented(_, span) => span.clone(),
            UnimplementedWithHelp(_, _, span) => Some(span.clone()),
            TypeError(err) => Some(err.span()),
            ParseError { span, .. } => Some(span.clone()),
            Internal(_, span) => Some(span.clone()),
            InternalOwned(_, span) => span.clone(),
            NoPredicateMainFunction(span) => Some(span.clone()),
            PredicateMainDoesNotReturnBool(span) => Some(span.clone()),
            NoScriptMainFunction(span) => Some(span.clone()),
            MultipleDefinitionsOfFunction { span, .. } => Some(span.clone()),
            MultipleDefinitionsOfName { span, .. } => Some(span.clone()),
            MultipleDefinitionsOfConstant { span, .. } => Some(span.clone()),
            AssignmentToNonMutable { span, .. } => Some(span.clone()),
            MutableParameterNotSupported { span, .. } => Some(span.clone()),
            ImmutableArgumentToMutableParameter { span } => Some(span.clone()),
            RefMutableNotAllowedInContractAbi { span, .. } => Some(span.clone()),
            MethodRequiresMutableSelf { span, .. } => Some(span.clone()),
            AssociatedFunctionCalledAsMethod { span, .. } => Some(span.clone()),
            TypeParameterNotInTypeScope { span, .. } => Some(span.clone()),
            MismatchedTypeInInterfaceSurface { span, .. } => Some(span.clone()),
            UnknownTrait { span, .. } => Some(span.clone()),
            FunctionNotAPartOfInterfaceSurface { span, .. } => Some(span.clone()),
            ConstantNotAPartOfInterfaceSurface { span, .. } => Some(span.clone()),
            MissingInterfaceSurfaceConstants { span, .. } => Some(span.clone()),
            MissingInterfaceSurfaceMethods { span, .. } => Some(span.clone()),
            IncorrectNumberOfTypeArguments { span, .. } => Some(span.clone()),
            DoesNotTakeTypeArguments { span, .. } => Some(span.clone()),
            DoesNotTakeTypeArgumentsAsPrefix { span, .. } => span.clone(),
            TypeArgumentsNotAllowed { span } => Some(span.clone()),
            NeedsTypeArguments { span, .. } => Some(span.clone()),
            StructMissingField { span, .. } => Some(span.clone()),
            StructDoesNotHaveField { span, .. } => Some(span.clone()),
            MethodNotFound { span, .. } => Some(span.clone()),
            ModuleNotFound { span, .. } => Some(span.clone()),
            NotATuple { span, .. } => Some(span.clone()),
            NotAStruct { span, .. } => Some(span.clone()),
            NotIndexable { span, .. } => Some(span.clone()),
            FieldAccessOnNonStruct { span, .. } => Some(span.clone()),
            FieldNotFound { span, .. } => Some(span.clone()),
            SymbolNotFound { span, .. } => Some(span.clone()),
            ImportPrivateSymbol { span, .. } => Some(span.clone()),
            ImportPrivateModule { span, .. } => Some(span.clone()),
            NoElseBranch { span, .. } => Some(span.clone()),
            NotAType { span, .. } => Some(span.clone()),
            MissingEnumInstantiator { span, .. } => Some(span.clone()),
            PathDoesNotReturn { span, .. } => Some(span.clone()),
            ExpectedModuleDocComment { span } => Some(span.clone()),
            UnknownRegister { span, .. } => Some(span.clone()),
            MissingImmediate { span, .. } => Some(span.clone()),
            InvalidImmediateValue { span, .. } => Some(span.clone()),
            UnknownEnumVariant { span, .. } => Some(span.clone()),
            UnrecognizedOp { span, .. } => Some(span.clone()),
            UnableToInferGeneric { span, .. } => Some(span.clone()),
            UnconstrainedGenericParameter { span, .. } => Some(span.clone()),
            TraitConstraintNotSatisfied { span, .. } => Some(span.clone()),
            TraitConstraintMissing { span, .. } => Some(span.clone()),
            Immediate06TooLarge { span, .. } => Some(span.clone()),
            Immediate12TooLarge { span, .. } => Some(span.clone()),
            Immediate18TooLarge { span, .. } => Some(span.clone()),
            Immediate24TooLarge { span, .. } => Some(span.clone()),
            IncorrectNumberOfAsmRegisters { span, .. } => Some(span.clone()),
            UnnecessaryImmediate { span, .. } => Some(span.clone()),
            AmbiguousPath { span, .. } => Some(span.clone()),
            UnknownType { span, .. } => Some(span.clone()),
            UnknownTypeName { span, .. } => Some(span.clone()),
            FileCouldNotBeRead { span, .. } => Some(span.clone()),
            ImportMustBeLibrary { span, .. } => Some(span.clone()),
            MoreThanOneEnumInstantiator { span, .. } => Some(span.clone()),
            UnnecessaryEnumInstantiator { span, .. } => Some(span.clone()),
            UnitVariantWithParenthesesEnumInstantiator { span, .. } => Some(span.clone()),
            TraitNotFound { span, .. } => Some(span.clone()),
            InvalidExpressionOnLhs { span, .. } => Some(span.clone()),
            TooManyArgumentsForFunction { span, .. } => Some(span.clone()),
            TooFewArgumentsForFunction { span, .. } => Some(span.clone()),
            MissingParenthesesForFunction { span, .. } => Some(span.clone()),
            InvalidAbiType { span, .. } => Some(span.clone()),
            NotAnAbi { span, .. } => Some(span.clone()),
            ImplAbiForNonContract { span, .. } => Some(span.clone()),
            ConflictingImplsForTraitAndType {
                second_impl_span, ..
            } => Some(second_impl_span.clone()),
            DuplicateDeclDefinedForType { span, .. } => Some(span.clone()),
            IncorrectNumberOfInterfaceSurfaceFunctionParameters { span, .. } => Some(span.clone()),
            ArgumentParameterTypeMismatch { span, .. } => Some(span.clone()),
            RecursiveCall { span, .. } => Some(span.clone()),
            RecursiveCallChain { span, .. } => Some(span.clone()),
            RecursiveType { span, .. } => Some(span.clone()),
            RecursiveTypeChain { span, .. } => Some(span.clone()),
            GMFromExternalContext { span, .. } => Some(span.clone()),
            MintFromExternalContext { span, .. } => Some(span.clone()),
            BurnFromExternalContext { span, .. } => Some(span.clone()),
            ContractStorageFromExternalContext { span, .. } => Some(span.clone()),
            InvalidOpcodeFromPredicate { span, .. } => Some(span.clone()),
            ArrayOutOfBounds { span, .. } => Some(span.clone()),
            ConstantsCannotBeShadowed { name, .. } => Some(name.span()),
            ConstantShadowsVariable { name, .. } => Some(name.span()),
            ShadowsOtherSymbol { name } => Some(name.span()),
            GenericShadowsGeneric { name } => Some(name.span()),
            MatchExpressionNonExhaustive { span, .. } => Some(span.clone()),
            MatchStructPatternMissingFields { span, .. } => Some(span.clone()),
            MatchVariableNotBoundInAllPatterns { span, .. } => Some(span.clone()),
            NotAnEnum { span, .. } => Some(span.clone()),
            StorageAccessMismatch { span, .. } => Some(span.clone()),
            TraitDeclPureImplImpure { span, .. } => Some(span.clone()),
            TraitImplPurityMismatch { span, .. } => Some(span.clone()),
            DeclIsNotAnEnum { span, .. } => Some(span.clone()),
            DeclIsNotAStruct { span, .. } => Some(span.clone()),
            DeclIsNotAFunction { span, .. } => Some(span.clone()),
            DeclIsNotAVariable { span, .. } => Some(span.clone()),
            DeclIsNotAnAbi { span, .. } => Some(span.clone()),
            DeclIsNotATrait { span, .. } => Some(span.clone()),
            DeclIsNotAnImplTrait { span, .. } => Some(span.clone()),
            DeclIsNotATraitFn { span, .. } => Some(span.clone()),
            DeclIsNotStorage { span, .. } => Some(span.clone()),
            DeclIsNotAConstant { span, .. } => Some(span.clone()),
            DeclIsNotATypeAlias { span, .. } => Some(span.clone()),
            ImpureInNonContract { span, .. } => Some(span.clone()),
            ImpureInPureContext { span, .. } => Some(span.clone()),
            ParameterRefMutabilityMismatch { span, .. } => Some(span.clone()),
            IntegerTooLarge { span, .. } => Some(span.clone()),
            IntegerTooSmall { span, .. } => Some(span.clone()),
            IntegerContainsInvalidDigit { span, .. } => Some(span.clone()),
            AbiAsSupertrait { span, .. } => Some(span.clone()),
            SupertraitImplRequired { span, .. } => Some(span.clone()),
            ContractCallParamRepeated { span, .. } => Some(span.clone()),
            UnrecognizedContractParam { span, .. } => Some(span.clone()),
            CallParamForNonContractCallMethod { span, .. } => Some(span.clone()),
            StorageFieldDoesNotExist { span, .. } => Some(span.clone()),
            InvalidStorageOnlyTypeDecl { span, .. } => Some(span.clone()),
            NoDeclaredStorage { span, .. } => Some(span.clone()),
            MultipleStorageDeclarations { span, .. } => Some(span.clone()),
            UnexpectedDeclaration { span, .. } => Some(span.clone()),
            ContractAddressMustBeKnown { span, .. } => Some(span.clone()),
            ConvertParseTree { error } => error.try_span(),
            Lex { error } => Some(error.span()),
            Parse { error } => Some(error.span.clone()),
            EnumNotFound { span, .. } => Some(span.clone()),
            TupleIndexOutOfBounds { span, .. } => Some(span.clone()),
            NonConstantDeclValue { span } => Some(span.clone()),
            StorageDeclarationInNonContract { span, .. } => Some(span.clone()),
            IntrinsicUnsupportedArgType { span, .. } => Some(span.clone()),
            IntrinsicIncorrectNumArgs { span, .. } => Some(span.clone()),
            IntrinsicIncorrectNumTArgs { span, .. } => Some(span.clone()),
            BreakOutsideLoop { span } => Some(span.clone()),
            ContinueOutsideLoop { span } => Some(span.clone()),
            ContractIdConstantNotAConstDecl { span } => Some(span.clone()),
            ContractIdValueNotALiteral { span } => Some(span.clone()),
            TypeNotAllowedInContractStorage { span, .. } => Some(span.clone()),
            RefMutableNotAllowedInMain { span, .. } => Some(span.clone()),
            NestedSliceReturnNotAllowedInMain { span } => Some(span.clone()),
            InitializedRegisterReassignment { span, .. } => Some(span.clone()),
            DisallowedControlFlowInstruction { span, .. } => Some(span.clone()),
            CallingPrivateLibraryMethod { span, .. } => Some(span.clone()),
            DisallowedIntrinsicInPredicate { span, .. } => Some(span.clone()),
            CoinsPassedToNonPayableMethod { span, .. } => Some(span.clone()),
            TraitImplPayabilityMismatch { span, .. } => Some(span.clone()),
            ConfigurableInLibrary { span } => Some(span.clone()),
            MultipleApplicableItemsInScope { span, .. } => Some(span.clone()),
            NonStrGenericType { span } => Some(span.clone()),
            CannotBeEvaluatedToConst { span } => Some(span.clone()),
            ContractCallsItsOwnMethod { span } => Some(span.clone()),
            AbiShadowsSuperAbiMethod { span, .. } => Some(span.clone()),
            ConflictingSuperAbiMethods { span, .. } => Some(span.clone()),
            AbiSupertraitMethodCallAsContractCall { span, .. } => Some(span.clone()),
        }
    }
}

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
                    Some(name.span()),
                    format!(
                        // Variable "x" shadows constant with the same name
                        //  or
                        // Constant "x" shadows imported constant with the same name
                        //  or
                        // ...
                        "{variable_or_constant} \"{name}\" shadows {}constant with the same name",
                        if constant_decl.is_some() { "imported " } else { "" }
                    )
                ),
                hints: {
                    let mut hints = vec![
                        Hint::info(
                            source_engine,
                            constant_span.clone(),
                            format!(
                                // Constant "x" is declared here.
                                //  or
                                // Constant "x" gets imported here.
                                "Constant \"{name}\" {} here{}.",
                                if constant_decl.is_some() { "gets imported" } else { "is declared" },
                                if *is_alias { " as alias" } else { "" }
                            )
                        ),
                    ];
                    if let Some(constant_decl) = constant_decl {
                        hints.push(Hint::info(
                            source_engine,
                            constant_decl.clone(),
                            format!("This is the original declaration of the imported constant \"{name}\".")
                        ));
                    }
                    hints.push(Hint::error(
                        source_engine,
                        name.span(),
                        format!(
                            "Shadowing via {} \"{name}\" happens here.", 
                            if variable_or_constant == "Variable" { "variable" } else { "new constant" }
                        )
                    ));
                    hints
                },
                help: vec![
                    format!("Unlike variables, constants cannot be shadowed by other constants or variables."),
                    match (variable_or_constant.as_str(), constant_decl.is_some()) {
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
                    Some(name.span()),
                    format!("Constant \"{name}\" shadows variable with the same name")
                ),
                hints: vec![
                    Hint::info(
                        source_engine,
                        variable_span.clone(),
                        format!("This is the shadowed variable \"{name}\".")
                    ),
                    Hint::error(
                        source_engine,
                        name.span(),
                        format!("This is the constant \"{name}\" that shadows the variable.")
                    ),
                ],
                help: vec![
                    format!("Variables can shadow other variables, but constants cannot."),
                    format!("Consider renaming either the variable or the constant."),
                ],
            },
           _ => Diagnostic {
                    // TODO: Temporary we use self here to achieve backward compatibility.
                    //       In general, self must not be used and will not be used once we
                    //       switch to our own #[error] macro. All the values for the formating
                    //       of a diagnostic must come from the enum variant parameters.
                    issue: Issue::error(source_engine, self.try_span(), format!("{}", self)),
                    reason: None,
                    hints: vec![],
                    help: vec![],
                }
        }
    }
}
