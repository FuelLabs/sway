use crate::convert_parse_tree_error::ConvertParseTreeError;
use crate::diagnostic::{Code, Diagnostic, Hint, Issue, Reason, ToDiagnostic};
use crate::lex_error::LexError;
use crate::parser_error::ParseError;
use crate::type_error::TypeError;

use core::fmt;
use sway_types::constants::STORAGE_PURITY_ATTRIBUTE_NAME;
use sway_types::{BaseIdent, Ident, SourceEngine, Span, Spanned};
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
    Unimplemented(&'static str, Span),
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
    InternalOwned(String, Span),
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
        constant_decl: Span,
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
    #[error("\"Self\" is not valid in the self type of an impl block")]
    SelfIsNotValidAsImplementingFor { span: Span },

    #[error("Unitialized register is being read before being written")]
    UninitRegisterInAsmBlockBeingRead { span: Span },
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
            UnknownVariable { span, .. } => span.clone(),
            NotAVariable { span, .. } => span.clone(),
            Unimplemented(_, span) => span.clone(),
            UnimplementedWithHelp(_, _, span) => span.clone(),
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
            AssignmentToNonMutable { span, .. } => span.clone(),
            MutableParameterNotSupported { span, .. } => span.clone(),
            ImmutableArgumentToMutableParameter { span } => span.clone(),
            RefMutableNotAllowedInContractAbi { span, .. } => span.clone(),
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
            StructMissingField { span, .. } => span.clone(),
            StructDoesNotHaveField { span, .. } => span.clone(),
            MethodNotFound { span, .. } => span.clone(),
            ModuleNotFound { span, .. } => span.clone(),
            NotATuple { span, .. } => span.clone(),
            NotAStruct { span, .. } => span.clone(),
            NotIndexable { span, .. } => span.clone(),
            FieldAccessOnNonStruct { span, .. } => span.clone(),
            FieldNotFound { span, .. } => span.clone(),
            SymbolNotFound { span, .. } => span.clone(),
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
            AmbiguousPath { span, .. } => span.clone(),
            UnknownType { span, .. } => span.clone(),
            UnknownTypeName { span, .. } => span.clone(),
            FileCouldNotBeRead { span, .. } => span.clone(),
            ImportMustBeLibrary { span, .. } => span.clone(),
            MoreThanOneEnumInstantiator { span, .. } => span.clone(),
            UnnecessaryEnumInstantiator { span, .. } => span.clone(),
            UnitVariantWithParenthesesEnumInstantiator { span, .. } => span.clone(),
            TraitNotFound { span, .. } => span.clone(),
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
            MatchArmVariableNotDefinedInAllAlternatives { variable, .. } => variable.span(),
            MatchArmVariableMismatchedType { variable, .. } => variable.span(),
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
            StorageFieldDoesNotExist { span, .. } => span.clone(),
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
            NonConstantDeclValue { span } => span.clone(),
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
            SelfIsNotValidAsImplementingFor { span } => span.clone(),
            UninitRegisterInAsmBlockBeingRead { span } => span.clone(),
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
