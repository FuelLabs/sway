//! Tools related to handling/recovering from Sway compile errors and reporting them to the user.

use crate::{
    constants::STORAGE_PURITY_ATTRIBUTE_NAME,
    convert_parse_tree::ConvertParseTreeError,
    style::{to_screaming_snake_case, to_snake_case, to_upper_camel_case},
    type_system::*,
    CallPath, VariableDeclaration,
};
use sway_types::{ident::Ident, span::Span, Spanned};

use std::{
    fmt::{self, Display},
    path::PathBuf,
    sync::Arc,
};
use thiserror::Error;

macro_rules! check {
    ($fn_expr: expr, $error_recovery: expr, $warnings: ident, $errors: ident $(,)?) => {{
        let mut res = $fn_expr;
        $warnings.append(&mut res.warnings);
        $errors.append(&mut res.errors);
        #[allow(clippy::manual_unwrap_or)]
        match res.value {
            None => $error_recovery,
            Some(value) => value,
        }
    }};
}

macro_rules! assert_or_warn {
    ($bool_expr: expr, $warnings: ident, $span: expr, $warning: expr $(,)?) => {{
        if !$bool_expr {
            use crate::error::CompileWarning;
            $warnings.push(CompileWarning {
                warning_content: $warning,
                span: $span,
            });
        }
    }};
}

/// Denotes a non-recoverable state
pub(crate) fn err<T>(warnings: Vec<CompileWarning>, errors: Vec<CompileError>) -> CompileResult<T> {
    CompileResult {
        value: None,
        warnings,
        errors,
    }
}

/// Denotes a recovered or non-error state
pub(crate) fn ok<T>(
    value: T,
    warnings: Vec<CompileWarning>,
    errors: Vec<CompileError>,
) -> CompileResult<T> {
    CompileResult {
        value: Some(value),
        warnings,
        errors,
    }
}

/// Acts as the result of parsing `Declaration`s, `Expression`s, etc.
/// Some `Expression`s need to be able to create `VariableDeclaration`s,
/// so this struct is used to "bubble up" those declarations to a viable
/// place in the AST.
#[derive(Debug, Clone)]
pub struct ParserLifter<T> {
    pub var_decls: Vec<VariableDeclaration>,
    pub value: T,
}

impl<T> ParserLifter<T> {
    #[allow(dead_code)]
    pub(crate) fn empty(value: T) -> Self {
        ParserLifter {
            var_decls: vec![],
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileResult<T> {
    pub value: Option<T>,
    pub warnings: Vec<CompileWarning>,
    pub errors: Vec<CompileError>,
}

impl<T> From<Result<T, CompileError>> for CompileResult<T> {
    fn from(o: Result<T, CompileError>) -> Self {
        match o {
            Ok(o) => CompileResult {
                value: Some(o),
                warnings: vec![],
                errors: vec![],
            },
            Err(e) => CompileResult {
                value: None,
                warnings: vec![],
                errors: vec![e],
            },
        }
    }
}

impl<T> CompileResult<T> {
    pub fn is_ok(&self) -> bool {
        self.value.is_some() && self.errors.is_empty()
    }

    pub fn is_ok_no_warn(&self) -> bool {
        self.value.is_some() && self.warnings.is_empty() && self.errors.is_empty()
    }

    pub fn new(value: Option<T>, warnings: Vec<CompileWarning>, errors: Vec<CompileError>) -> Self {
        CompileResult {
            value,
            warnings,
            errors,
        }
    }

    pub fn ok(
        mut self,
        warnings: &mut Vec<CompileWarning>,
        errors: &mut Vec<CompileError>,
    ) -> Option<T> {
        warnings.append(&mut self.warnings);
        errors.append(&mut self.errors);
        self.value
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> CompileResult<U> {
        match self.value {
            None => err(self.warnings, self.errors),
            Some(value) => ok(f(value), self.warnings, self.errors),
        }
    }

    pub fn flat_map<U, F: FnOnce(T) -> CompileResult<U>>(self, f: F) -> CompileResult<U> {
        match self.value {
            None => err(self.warnings, self.errors),
            Some(value) => {
                let res = f(value);
                CompileResult {
                    value: res.value,
                    warnings: [self.warnings, res.warnings].concat(),
                    errors: [self.errors, res.errors].concat(),
                }
            }
        }
    }

    pub fn unwrap(self, warnings: &mut Vec<CompileWarning>, errors: &mut Vec<CompileError>) -> T {
        let panic_msg = format!("Unwrapped an err {:?}", self.errors);
        self.unwrap_or_else(warnings, errors, || panic!("{}", panic_msg))
    }

    pub fn unwrap_or_else<F: FnOnce() -> T>(
        self,
        warnings: &mut Vec<CompileWarning>,
        errors: &mut Vec<CompileError>,
        or_else: F,
    ) -> T {
        self.ok(warnings, errors).unwrap_or_else(or_else)
    }
}

impl<'a, T> CompileResult<&'a T>
where
    T: Clone,
{
    /// Converts a `CompileResult` around a reference value to an owned value by cloning the type
    /// behind the reference.
    pub fn cloned(self) -> CompileResult<T> {
        let CompileResult {
            value,
            warnings,
            errors,
        } = self;
        let value = value.cloned();
        CompileResult {
            value,
            warnings,
            errors,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hint {
    msg: Option<String>,
}

impl Hint {
    pub fn empty() -> Hint {
        Hint { msg: None }
    }
    pub fn new(msg: String) -> Hint {
        Hint { msg: Some(msg) }
    }
}

impl Display for Hint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hint: {}", &self.msg.as_ref().unwrap_or(&"".to_string()))
    }
}

// TODO: since moving to using Idents instead of strings the warning_content will usually contain a
// duplicate of the span.
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

    /// Returns the line and column start and end
    pub fn line_col(&self) -> (LineCol, LineCol) {
        (
            self.span.start_pos().line_col().into(),
            self.span.end_pos().line_col().into(),
        )
    }
}

#[derive(Clone, Copy)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

impl From<(usize, usize)> for LineCol {
    fn from(o: (usize, usize)) -> Self {
        LineCol {
            line: o.0,
            col: o.1,
        }
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
        r#type: Box<TypeInfo>,
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
}

impl fmt::Display for Warning {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        }
    }
}

// TODO: since moving to using Idents instead of strings, there are a lot of redundant spans in
// this type.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompileError {
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariable { var_name: Ident },
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariablePath { var_name: Ident, span: Span },
    #[error("Function \"{name}\" does not exist in this scope.")]
    UnknownFunction { name: Ident, span: Span },
    #[error("Identifier \"{name}\" was used as a variable, but it is actually a {what_it_is}.")]
    NotAVariable {
        name: Ident,
        what_it_is: &'static str,
    },
    #[error(
        "Identifier \"{name}\" was called as if it was a function, but it is actually a \
         {what_it_is}."
    )]
    NotAFunction {
        name: CallPath,
        what_it_is: &'static str,
    },
    #[error("Unimplemented feature: {0}")]
    Unimplemented(&'static str, Span),
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
        "Byte literal had length of {byte_length}. Byte literals must be either one byte long (8 \
         binary digits or 2 hex digits) or 32 bytes long (256 binary digits or 64 hex digits)"
    )]
    InvalidByteLiteralLength { byte_length: usize, span: Span },
    #[error("Expected an expression to follow operator \"{op}\"")]
    ExpectedExprAfterOp { op: String, span: Span },
    #[error("Expected an operator, but \"{op}\" is not a recognized operator. ")]
    ExpectedOp { op: String, span: Span },
    #[error(
        "Program contains multiple contracts. A valid program should only contain at most one \
         contract."
    )]
    MultipleContracts(Span),
    #[error(
        "Program contains multiple scripts. A valid program should only contain at most one \
         script."
    )]
    MultipleScripts(Span),
    #[error(
        "Program contains multiple predicates. A valid program should only contain at most one \
         predicate."
    )]
    MultiplePredicates(Span),
    #[error(
        "Predicate declaration contains no main function. Predicates require a main function."
    )]
    NoPredicateMainFunction(Span),
    #[error("A predicate's main function must return a boolean.")]
    PredicateMainDoesNotReturnBool(Span),
    #[error("Script declaration contains no main function. Scripts require a main function.")]
    NoScriptMainFunction(Span),
    #[error("Function \"{name}\" was already defined in scope.")]
    MultipleDefinitionsOfFunction { name: Ident },
    #[error(
        "Attempted to reassign to a symbol that is not a variable. Symbol {name} is not a mutable \
         variable, it is a {kind}."
    )]
    ReassignmentToNonVariable {
        name: Ident,
        kind: &'static str,
        span: Span,
    },
    #[error("Assignment to immutable variable. Variable {name} is not declared as mutable.")]
    AssignmentToNonMutable { name: Ident },
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
    MutableParameterNotSupported { param_name: Ident },
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
        "Asm opcode has multiple immediates specified, when any opcode has at most one immediate."
    )]
    MultipleImmediates(Span),
    #[error(
        "Expected: {expected} \n\
         found:    {given}. The definition of this function must \
         match the one in the trait declaration."
    )]
    MismatchedTypeInTrait {
        span: Span,
        given: String,
        expected: String,
    },
    #[error("\"{name}\" is not a trait, so it cannot be \"impl'd\".")]
    NotATrait { span: Span, name: Ident },
    #[error("Trait \"{name}\" cannot be found in the current scope.")]
    UnknownTrait { span: Span, name: Ident },
    #[error("Function \"{name}\" is not a part of trait \"{trait_name}\"'s interface surface.")]
    FunctionNotAPartOfInterfaceSurface {
        name: Ident,
        trait_name: Ident,
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
    #[error("Type arguments are not allowed for this type.")]
    TypeArgumentsNotAllowed { span: Span },
    #[error("\"{name}\" needs type arguments.")]
    NeedsTypeArguments { name: Ident, span: Span },
    #[error(
        "Struct with name \"{name}\" could not be found in this scope. Perhaps you need to import \
         it?"
    )]
    StructNotFound { name: Ident, span: Span },
    #[error(
        "Enum with name \"{name}\" could not be found in this scope. Perhaps you need to import \
         it?"
    )]
    EnumNotFound { name: Ident, span: Span },
    #[error(
        "The name \"{name}\" does not refer to a struct, but this is an attempted struct \
         declaration."
    )]
    DeclaredNonStructAsStruct { name: Ident, span: Span },
    #[error(
        "Attempted to access field \"{field_name}\" of non-struct \"{name}\". Field accesses are \
         only valid on structs."
    )]
    AccessedFieldOfNonStruct {
        field_name: Ident,
        name: Ident,
        span: Span,
    },
    #[error(
        "Attempted to access a method on something that has no methods. \"{name}\" is a {thing}, \
         not a type with methods."
    )]
    MethodOnNonValue {
        name: Ident,
        thing: Ident,
        span: Span,
    },
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
    #[error(
        "Field \"{field_name}\" not found on struct \"{struct_name}\". Available fields are:\n \
         {available_fields}"
    )]
    FieldNotFound {
        field_name: Ident,
        available_fields: String,
        struct_name: Ident,
    },
    #[error("Could not find symbol \"{name}\" in this scope.")]
    SymbolNotFound { name: Ident },
    #[error("Symbol \"{name}\" is private.")]
    ImportPrivateSymbol { name: Ident },
    #[error(
        "Because this if expression's value is used, an \"else\" branch is required and it must \
         return type \"{r#type}\""
    )]
    NoElseBranch { span: Span, r#type: String },
    #[error("Use of type `Self` outside of a context in which `Self` refers to a type.")]
    UnqualifiedSelfType { span: Span },
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
    #[error("Expected block to implicitly return a value of type \"{ty}\".")]
    ExpectedImplicitReturnFromBlockWithType { span: Span, ty: String },
    #[error("Expected block to implicitly return a value.")]
    ExpectedImplicitReturnFromBlock { span: Span },
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
    #[error(
        "This expression was expected to return a value but no return register was specified. \
         Provide a register in the implicit return position of this asm expression to return it."
    )]
    InvalidAssemblyMismatchedReturn { span: Span },
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
    #[error("The value \"{val}\" is too large to fit in this 6-bit immediate spot.")]
    Immediate06TooLarge { val: u64, span: Span },
    #[error("The value \"{val}\" is too large to fit in this 12-bit immediate spot.")]
    Immediate12TooLarge { val: u64, span: Span },
    #[error("The value \"{val}\" is too large to fit in this 18-bit immediate spot.")]
    Immediate18TooLarge { val: u64, span: Span },
    #[error("The value \"{val}\" is too large to fit in this 24-bit immediate spot.")]
    Immediate24TooLarge { val: u64, span: Span },
    #[error(
        "The opcode \"ji\" is not valid in inline assembly. Try using function calls instead."
    )]
    DisallowedJi { span: Span },
    #[error("The opcode \"jnei\" is not valid in inline assembly. Use an enclosing if expression instead.")]
    DisallowedJnei { span: Span },
    #[error("The opcode \"jnzi\" is not valid in inline assembly. Use an enclosing if expression instead.")]
    DisallowedJnzi { span: Span },
    #[error(
        "The opcode \"lw\" is not valid in inline assembly. Try assigning a static value to a variable instead."
    )]
    DisallowedLw { span: Span },
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
    #[error("This value is not valid within a \"str\" type.")]
    InvalidStrType { raw: String, span: Span },
    #[error("Unknown type name.")]
    UnknownType { span: Span },
    #[error("Unknown type name \"{name}\".")]
    UnknownTypeName { name: String, span: Span },
    #[error("Bytecode can only support programs with up to 2^12 words worth of opcodes. Try refactoring into contract calls? This is a temporary error and will be implemented in the future.")]
    TooManyInstructions { span: Span },
    #[error(
        "No valid {} file (.{}) was found at {file_path}",
        crate::constants::LANGUAGE_NAME,
        crate::constants::DEFAULT_FILE_EXTENSION
    )]
    FileNotFound { span: Span, file_path: String },
    #[error("The file {file_path} could not be read: {stringified_error}")]
    FileCouldNotBeRead {
        span: Span,
        file_path: String,
        stringified_error: String,
    },
    #[error("This imported file must be a library. It must start with \"library <name>\", where \"name\" is the name of the library this file contains.")]
    ImportMustBeLibrary { span: Span },
    #[error("An enum instantiaton cannot contain more than one value. This should be a single value of type {ty}.")]
    MoreThanOneEnumInstantiator { span: Span, ty: String },
    #[error("This enum variant represents the unit type, so it should not be instantiated with any value.")]
    UnnecessaryEnumInstantiator { span: Span },
    #[error("Cannot find trait \"{name}\" in this scope.")]
    TraitNotFound { name: CallPath },
    #[error("This expression is not valid on the left hand side of a reassignment.")]
    InvalidExpressionOnLhs { span: Span },
    #[error(
        "Function \"{method_name}\" expects {expected} arguments but you provided {received}."
    )]
    TooManyArgumentsForFunction {
        span: Span,
        method_name: Ident,
        expected: usize,
        received: usize,
    },
    #[error(
        "Function \"{method_name}\" expects {expected} arguments but you provided {received}."
    )]
    TooFewArgumentsForFunction {
        span: Span,
        method_name: Ident,
        expected: usize,
        received: usize,
    },
    #[error("This type is invalid in a function selector. A contract ABI function selector must be a known sized type, not generic.")]
    InvalidAbiType { span: Span },
    #[error("This is a {actually_is}, not an ABI. An ABI cast requires a valid ABI to cast the address to.")]
    NotAnAbi {
        span: Span,
        actually_is: &'static str,
    },
    #[error("An ABI can only be implemented for the `Contract` type, so this implementation of an ABI for type \"{ty}\" is invalid.")]
    ImplAbiForNonContract { span: Span, ty: String },
    #[error("The function \"{fn_name}\" in trait \"{trait_name}\" is defined with {num_parameters} parameters, but the provided implementation has {provided_parameters} parameters.")]
    IncorrectNumberOfInterfaceSurfaceFunctionParameters {
        fn_name: Ident,
        trait_name: Ident,
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
    #[error(
        "The size of this type is not known. Try putting it on the heap or changing the type."
    )]
    TypeWithUnknownSize { span: Span },
    #[error("File {file_path} generates an infinite dependency cycle.")]
    InfiniteDependencies { file_path: String, span: Span },
    #[error("The GM (get-metadata) opcode, when called from an external context, will cause the VM to panic.")]
    GMFromExternalContract { span: Span },
    #[error("The MINT opcode cannot be used in an external context.")]
    MintFromExternalContext { span: Span },
    #[error("The BURN opcode cannot be used in an external context.")]
    BurnFromExternalContext { span: Span },
    #[error("Contract storage cannot be used in an external context.")]
    ContractStorageFromExternalContext { span: Span },
    #[error("Array index out of bounds; the length is {count} but the index is {index}.")]
    ArrayOutOfBounds { index: u64, count: u64, span: Span },
    #[error("Tuple index out of bounds; the arity is {count} but the index is {index}.")]
    TupleIndexOutOfBounds {
        index: usize,
        count: usize,
        span: Span,
    },
    #[error("The name \"{name}\" shadows another symbol with the same name.")]
    ShadowsOtherSymbol { name: Ident },
    #[error("The name \"{name}\" is already used for a generic parameter in this scope.")]
    GenericShadowsGeneric { name: Ident },
    #[error("The name \"{name}\" imported through `*` shadows another symbol with the same name.")]
    StarImportShadowsOtherSymbol { name: Ident },
    #[error(
        "Match expression arm has mismatched types.\n\
         expected: {expected}\n\
         "
    )]
    MatchWrongType { expected: TypeId, span: Span },
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
    #[error(
        "Storage attribute access mismatch. Try giving the surrounding function more access by \
        adding \"#[{STORAGE_PURITY_ATTRIBUTE_NAME}({attrs})]\" to the function declaration."
    )]
    StorageAccessMismatch { attrs: String, span: Span },
    #[error(
        "The trait function \"{fn_name}\" in trait \"{trait_name}\" is pure, but this \
        implementation is not.  The \"{STORAGE_PURITY_ATTRIBUTE_NAME}\" annotation must be \
        removed, or the trait declaration must be changed to \
        \"#[{STORAGE_PURITY_ATTRIBUTE_NAME}({attrs})]\"."
    )]
    TraitDeclPureImplImpure {
        fn_name: Ident,
        trait_name: Ident,
        attrs: String,
        span: Span,
    },
    #[error(
        "Storage attribute access mismatch. The trait function \"{fn_name}\" in trait \
        \"{trait_name}\" requires the storage attribute(s) #[{STORAGE_PURITY_ATTRIBUTE_NAME}({attrs})]."
    )]
    TraitImplPurityMismatch {
        fn_name: Ident,
        trait_name: Ident,
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
    #[error("Literal value is too large for type {ty}.")]
    IntegerTooLarge { span: Span, ty: String },
    #[error("Literal value underflows type {ty}.")]
    IntegerTooSmall { span: Span, ty: String },
    #[error("Literal value contains digits which are not valid for type {ty}.")]
    IntegerContainsInvalidDigit { span: Span, ty: String },
    #[error("Unexpected alias after an asterisk in an import statement.")]
    AsteriskWithAlias { span: Span },
    #[error("A trait cannot be a subtrait of an ABI.")]
    AbiAsSupertrait { span: Span },
    #[error("The trait \"{supertrait_name}\" is not implemented for type \"{type_name}\"")]
    SupertraitImplMissing {
        supertrait_name: CallPath,
        type_name: String,
        span: Span,
    },
    #[error(
        "Implementation of trait \"{supertrait_name}\" is required by this bound in \"{trait_name}\""
    )]
    SupertraitImplRequired {
        supertrait_name: CallPath,
        trait_name: Ident,
        span: Span,
    },
    #[error("Cannot use `if let` on a non-enum type.")]
    IfLetNonEnum { span: Span },
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
    StorageFieldDoesNotExist { name: Ident },
    #[error("No storage has been declared")]
    NoDeclaredStorage { span: Span },
    #[error("Multiple storage declarations were found")]
    MultipleStorageDeclarations { span: Span },
    #[error("Type {ty} can only be declared directly as a storage field")]
    InvalidStorageOnlyTypeDecl { ty: String, span: Span },
    #[error("Expected identifier, found keyword \"{name}\" ")]
    InvalidVariableName { name: Ident },
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
    Lex { error: sway_parse::LexError },
    #[error("{}", error)]
    Parse { error: sway_parse::ParseError },
    #[error("\"where\" clauses are not yet supported")]
    WhereClauseNotYetSupported { span: Span },
    #[error("Could not evaluate initializer to a const declaration.")]
    NonConstantDeclValue { span: Span },
    #[error("Declaring storage in a {program_kind} is not allowed.")]
    StorageDeclarationInNonContract { program_kind: String, span: Span },
    #[error("Unsupported argument type to intrinsic \"{name}\". {hint}")]
    IntrinsicUnsupportedArgType {
        name: String,
        span: Span,
        hint: Hint,
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
    #[error("arguments to \"main()\" are not yet supported. See the issue here: github.com/FuelLabs/sway/issues/845")]
    MainArgsNotYetSupported { span: Span },
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
            UnknownVariable { var_name } => var_name.span(),
            UnknownVariablePath { span, .. } => span.clone(),
            UnknownFunction { span, .. } => span.clone(),
            NotAVariable { name, .. } => name.span(),
            NotAFunction { name, .. } => name.span(),
            Unimplemented(_, span) => span.clone(),
            TypeError(err) => err.span(),
            ParseError { span, .. } => span.clone(),
            Internal(_, span) => span.clone(),
            InternalOwned(_, span) => span.clone(),
            InvalidByteLiteralLength { span, .. } => span.clone(),
            ExpectedExprAfterOp { span, .. } => span.clone(),
            ExpectedOp { span, .. } => span.clone(),
            MultiplePredicates(span) => span.clone(),
            MultipleScripts(span) => span.clone(),
            MultipleContracts(span) => span.clone(),
            NoPredicateMainFunction(span) => span.clone(),
            PredicateMainDoesNotReturnBool(span) => span.clone(),
            NoScriptMainFunction(span) => span.clone(),
            MultipleDefinitionsOfFunction { name } => name.span(),
            ReassignmentToNonVariable { span, .. } => span.clone(),
            AssignmentToNonMutable { name } => name.span(),
            MutableParameterNotSupported { param_name } => param_name.span(),
            MethodRequiresMutableSelf { span, .. } => span.clone(),
            AssociatedFunctionCalledAsMethod { span, .. } => span.clone(),
            TypeParameterNotInTypeScope { span, .. } => span.clone(),
            MultipleImmediates(span) => span.clone(),
            MismatchedTypeInTrait { span, .. } => span.clone(),
            NotATrait { span, .. } => span.clone(),
            UnknownTrait { span, .. } => span.clone(),
            FunctionNotAPartOfInterfaceSurface { span, .. } => span.clone(),
            MissingInterfaceSurfaceMethods { span, .. } => span.clone(),
            IncorrectNumberOfTypeArguments { span, .. } => span.clone(),
            DoesNotTakeTypeArguments { span, .. } => span.clone(),
            TypeArgumentsNotAllowed { span } => span.clone(),
            NeedsTypeArguments { span, .. } => span.clone(),
            StructNotFound { span, .. } => span.clone(),
            DeclaredNonStructAsStruct { span, .. } => span.clone(),
            AccessedFieldOfNonStruct { span, .. } => span.clone(),
            MethodOnNonValue { span, .. } => span.clone(),
            StructMissingField { span, .. } => span.clone(),
            StructDoesNotHaveField { span, .. } => span.clone(),
            MethodNotFound { method_name, .. } => method_name.span(),
            ModuleNotFound { span, .. } => span.clone(),
            NotATuple { span, .. } => span.clone(),
            NotAStruct { span, .. } => span.clone(),
            FieldAccessOnNonStruct { span, .. } => span.clone(),
            FieldNotFound { field_name, .. } => field_name.span(),
            SymbolNotFound { name, .. } => name.span(),
            ImportPrivateSymbol { name } => name.span(),
            NoElseBranch { span, .. } => span.clone(),
            UnqualifiedSelfType { span, .. } => span.clone(),
            NotAType { span, .. } => span.clone(),
            MissingEnumInstantiator { span, .. } => span.clone(),
            PathDoesNotReturn { span, .. } => span.clone(),
            ExpectedImplicitReturnFromBlockWithType { span, .. } => span.clone(),
            ExpectedImplicitReturnFromBlock { span, .. } => span.clone(),
            UnknownRegister { span, .. } => span.clone(),
            MissingImmediate { span, .. } => span.clone(),
            InvalidImmediateValue { span, .. } => span.clone(),
            InvalidAssemblyMismatchedReturn { span, .. } => span.clone(),
            UnknownEnumVariant { span, .. } => span.clone(),
            UnrecognizedOp { span, .. } => span.clone(),
            UnableToInferGeneric { span, .. } => span.clone(),
            UnconstrainedGenericParameter { span, .. } => span.clone(),
            Immediate06TooLarge { span, .. } => span.clone(),
            Immediate12TooLarge { span, .. } => span.clone(),
            Immediate18TooLarge { span, .. } => span.clone(),
            Immediate24TooLarge { span, .. } => span.clone(),
            DisallowedJi { span, .. } => span.clone(),
            DisallowedJnei { span, .. } => span.clone(),
            DisallowedJnzi { span, .. } => span.clone(),
            DisallowedLw { span, .. } => span.clone(),
            IncorrectNumberOfAsmRegisters { span, .. } => span.clone(),
            UnnecessaryImmediate { span, .. } => span.clone(),
            AmbiguousPath { span, .. } => span.clone(),
            UnknownType { span, .. } => span.clone(),
            UnknownTypeName { span, .. } => span.clone(),
            InvalidStrType { span, .. } => span.clone(),
            TooManyInstructions { span, .. } => span.clone(),
            FileNotFound { span, .. } => span.clone(),
            FileCouldNotBeRead { span, .. } => span.clone(),
            ImportMustBeLibrary { span, .. } => span.clone(),
            MoreThanOneEnumInstantiator { span, .. } => span.clone(),
            UnnecessaryEnumInstantiator { span, .. } => span.clone(),
            TraitNotFound { name } => name.span(),
            InvalidExpressionOnLhs { span, .. } => span.clone(),
            TooManyArgumentsForFunction { span, .. } => span.clone(),
            TooFewArgumentsForFunction { span, .. } => span.clone(),
            InvalidAbiType { span, .. } => span.clone(),
            NotAnAbi { span, .. } => span.clone(),
            ImplAbiForNonContract { span, .. } => span.clone(),
            IncorrectNumberOfInterfaceSurfaceFunctionParameters { span, .. } => span.clone(),
            ArgumentParameterTypeMismatch { span, .. } => span.clone(),
            RecursiveCall { span, .. } => span.clone(),
            RecursiveCallChain { span, .. } => span.clone(),
            RecursiveType { span, .. } => span.clone(),
            RecursiveTypeChain { span, .. } => span.clone(),
            TypeWithUnknownSize { span, .. } => span.clone(),
            InfiniteDependencies { span, .. } => span.clone(),
            GMFromExternalContract { span, .. } => span.clone(),
            MintFromExternalContext { span, .. } => span.clone(),
            BurnFromExternalContext { span, .. } => span.clone(),
            ContractStorageFromExternalContext { span, .. } => span.clone(),
            ArrayOutOfBounds { span, .. } => span.clone(),
            ShadowsOtherSymbol { name } => name.span(),
            GenericShadowsGeneric { name } => name.span(),
            StarImportShadowsOtherSymbol { name } => name.span(),
            MatchWrongType { span, .. } => span.clone(),
            MatchExpressionNonExhaustive { span, .. } => span.clone(),
            MatchStructPatternMissingFields { span, .. } => span.clone(),
            NotAnEnum { span, .. } => span.clone(),
            StorageAccessMismatch { span, .. } => span.clone(),
            TraitDeclPureImplImpure { span, .. } => span.clone(),
            TraitImplPurityMismatch { span, .. } => span.clone(),
            DeclIsNotAnEnum { span, .. } => span.clone(),
            DeclIsNotAStruct { span, .. } => span.clone(),
            DeclIsNotAFunction { span, .. } => span.clone(),
            DeclIsNotAVariable { span, .. } => span.clone(),
            DeclIsNotAnAbi { span, .. } => span.clone(),
            ImpureInNonContract { span, .. } => span.clone(),
            ImpureInPureContext { span, .. } => span.clone(),
            IntegerTooLarge { span, .. } => span.clone(),
            IntegerTooSmall { span, .. } => span.clone(),
            IntegerContainsInvalidDigit { span, .. } => span.clone(),
            AsteriskWithAlias { span, .. } => span.clone(),
            AbiAsSupertrait { span, .. } => span.clone(),
            SupertraitImplMissing { span, .. } => span.clone(),
            SupertraitImplRequired { span, .. } => span.clone(),
            IfLetNonEnum { span, .. } => span.clone(),
            ContractCallParamRepeated { span, .. } => span.clone(),
            UnrecognizedContractParam { span, .. } => span.clone(),
            CallParamForNonContractCallMethod { span, .. } => span.clone(),
            StorageFieldDoesNotExist { name } => name.span(),
            InvalidStorageOnlyTypeDecl { span, .. } => span.clone(),
            NoDeclaredStorage { span, .. } => span.clone(),
            MultipleStorageDeclarations { span, .. } => span.clone(),
            InvalidVariableName { name } => name.span(),
            UnexpectedDeclaration { span, .. } => span.clone(),
            ContractAddressMustBeKnown { span, .. } => span.clone(),
            ConvertParseTree { error } => error.span(),
            WhereClauseNotYetSupported { span, .. } => span.clone(),
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
            MainArgsNotYetSupported { span } => span.clone(),
        }
    }
}

impl CompileError {
    pub fn path(&self) -> Option<Arc<PathBuf>> {
        self.span().path().cloned()
    }

    /// Returns the line and column start and end
    pub fn line_col(&self) -> (LineCol, LineCol) {
        (
            self.span().start_pos().line_col().into(),
            self.span().end_pos().line_col().into(),
        )
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeError {
    #[error(
        "Mismatched types.\n\
         expected: {expected}\n\
         found:    {received}.\n\
         {help}", expected=look_up_type_id(*expected).to_string(), received=look_up_type_id(*received).to_string(), help=if !help_text.is_empty() { format!("help: {}", help_text) } else { String::new() }
    )]
    MismatchedType {
        expected: TypeId,
        received: TypeId,
        help_text: String,
        span: Span,
    },
    #[error("This type is not known. Try annotating it with a type annotation.")]
    UnknownType { span: Span },
    #[error(
        "The pattern for this match expression arm has a mismatched type.\n\
         expected: {expected}\n\
         found:    {received}.\n\
         "
    )]
    MatchArmScrutineeWrongType {
        expected: TypeId,
        received: TypeId,
        span: Span,
    },
}

impl Spanned for TypeError {
    fn span(&self) -> Span {
        use TypeError::*;
        match self {
            MismatchedType { span, .. } => span.clone(),
            UnknownType { span } => span.clone(),
            MatchArmScrutineeWrongType { span, .. } => span.clone(),
        }
    }
}
