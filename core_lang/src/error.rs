use crate::span::Span;
use crate::{parser::Rule, types::MaybeResolvedType};
use inflector::cases::classcase::to_class_case;
use inflector::cases::snakecase::to_snake_case;
use line_col::LineColLookup;
use source_span::{
    fmt::{Formatter, Style},
    Position,
};
use std::fmt;
use thiserror::Error;

macro_rules! check {
    ($fn_expr: expr, $error_recovery: expr, $warnings: ident, $errors: ident) => {{
        let mut res = $fn_expr;
        $warnings.append(&mut res.warnings);
        $errors.append(&mut res.errors);
        match res.value {
            None => $error_recovery,
            Some(value) => value,
        }
    }};
}

macro_rules! assert_or_warn {
    ($bool_expr: expr, $warnings: ident, $span: expr, $warning: expr) => {
        if !$bool_expr {
            use crate::error::CompileWarning;
            $warnings.push(CompileWarning {
                warning_content: $warning,
                span: $span,
            });
        }
    };
}

/// Denotes a non-recoverable state
pub(crate) fn err<'sc, T>(
    warnings: Vec<CompileWarning<'sc>>,
    errors: Vec<CompileError<'sc>>,
) -> CompileResult<'sc, T> {
    CompileResult {
        value: None,
        warnings,
        errors,
    }
}

/// Denotes a recovered or non-error state
pub(crate) fn ok<'sc, T>(
    value: T,
    warnings: Vec<CompileWarning<'sc>>,
    errors: Vec<CompileError<'sc>>,
) -> CompileResult<'sc, T> {
    CompileResult {
        value: Some(value),
        warnings,
        errors,
    }
}

#[derive(Debug, Clone)]
pub struct CompileResult<'sc, T> {
    pub value: Option<T>,
    pub warnings: Vec<CompileWarning<'sc>>,
    pub errors: Vec<CompileError<'sc>>,
}

impl<'sc, T> CompileResult<'sc, T> {
    pub fn ok(
        mut self,
        warnings: &mut Vec<CompileWarning<'sc>>,
        errors: &mut Vec<CompileError<'sc>>,
    ) -> Option<T> {
        warnings.append(&mut self.warnings);
        errors.append(&mut self.errors);
        self.value
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> CompileResult<'sc, U> {
        match self.value {
            None => err(self.warnings, self.errors),
            Some(value) => ok(f(value), self.warnings, self.errors),
        }
    }

    pub fn unwrap(
        self,
        warnings: &mut Vec<CompileWarning<'sc>>,
        errors: &mut Vec<CompileError<'sc>>,
    ) -> T {
        let panic_msg = format!("Unwrapped an err {:?}", self.errors);
        self.unwrap_or_else(warnings, errors, || panic!("{}", panic_msg))
    }

    pub fn unwrap_or_else<F: FnOnce() -> T>(
        self,
        warnings: &mut Vec<CompileWarning<'sc>>,
        errors: &mut Vec<CompileError<'sc>>,
        or_else: F,
    ) -> T {
        self.ok(warnings, errors).unwrap_or_else(or_else)
    }
}

#[derive(Debug, Clone)]
pub struct CompileWarning<'sc> {
    pub span: Span<'sc>,
    pub warning_content: Warning<'sc>,
}

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

impl<'sc> CompileWarning<'sc> {
    pub fn to_friendly_warning_string(&self) -> String {
        self.warning_content.to_string()
    }

    pub fn span(&self) -> (usize, usize) {
        (self.span.start(), self.span.end())
    }

    pub fn path(&self) -> String {
        self.span.path()
    }

    /// Returns the line and column start and end
    pub fn line_col(&self) -> (LineCol, LineCol) {
        (
            self.span.start_pos().line_col().into(),
            self.span.end_pos().line_col().into(),
        )
    }

    pub fn format(&self, fmt: &mut Formatter) -> source_span::fmt::Formatted {
        let input = self.span.input();
        let chars = input.chars().map(|x| -> Result<_, ()> { Ok(x) });

        let metrics = source_span::DEFAULT_METRICS;
        let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);

        for c in buffer.iter() {
            let _ = c.unwrap(); // report eventual errors.
        }

        let (start_pos, end_pos) = self.span();
        let lookup = LineColLookup::new(input);
        let (start_line, start_col) = lookup.get(start_pos);
        let (end_line, end_col) = lookup.get(end_pos - 1);

        let err_start = Position::new(start_line - 1, start_col - 1);
        let err_end = Position::new(end_line - 1, end_col - 1);
        let err_span = source_span::Span::new(err_start, err_end, err_end.next_column());
        fmt.add(
            err_span,
            Some(self.to_friendly_warning_string()),
            Style::Warning,
        );

        fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum Warning<'sc> {
    NonClassCaseStructName {
        struct_name: &'sc str,
    },
    NonClassCaseTraitName {
        name: &'sc str,
    },
    NonClassCaseEnumName {
        enum_name: &'sc str,
    },
    NonClassCaseEnumVariantName {
        variant_name: &'sc str,
    },
    NonSnakeCaseStructFieldName {
        field_name: &'sc str,
    },
    NonSnakeCaseFunctionName {
        name: &'sc str,
    },
    LossOfPrecision {
        initial_type: Box<MaybeResolvedType<'sc>>,
        cast_to: MaybeResolvedType<'sc>,
    },
    UnusedReturnValue {
        r#type: MaybeResolvedType<'sc>,
    },
    SimilarMethodFound {
        lib: &'sc str,
        module: &'sc str,
        name: &'sc str,
    },
    OverridesOtherSymbol {
        name: String,
    },
    OverridingTraitImplementation,
    DeadDeclaration,
    DeadFunctionDeclaration,
    DeadStructDeclaration,
    DeadTrait,
    UnreachableCode,
    DeadEnumVariant {
        variant_name: String,
    },
    DeadMethod,
    StructFieldNeverRead,
    ShadowingReservedRegister {
        reg_name: &'sc str,
    },
}

impl<'sc> fmt::Display for Warning<'sc> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Warning::*;
        match self {
            NonClassCaseStructName { struct_name } => {
                write!(f,
                "Struct name \"{}\" is not idiomatic. Structs should have a ClassCase name, like \
                 \"{}\".",
                struct_name,
                to_class_case(struct_name)
            )
            }
            NonClassCaseTraitName { name } => {
                write!(f,
                "Trait name \"{}\" is not idiomatic. Traits should have a ClassCase name, like \
                 \"{}\".",
                name,
                to_class_case(name)
            )
            }
            NonClassCaseEnumName { enum_name } => write!(
                f,
                "Enum \"{}\"'s capitalization is not idiomatic. Enums should have a ClassCase \
                 name, like \"{}\".",
                enum_name,
                to_class_case(enum_name)
            ),
            NonSnakeCaseStructFieldName { field_name } => write!(
                f,
                "Struct field name \"{}\" is not idiomatic. Struct field names should have a \
                 snake_case name, like \"{}\".",
                field_name,
                to_snake_case(field_name)
            ),
            NonClassCaseEnumVariantName { variant_name } => write!(
                f,
                "Enum variant name \"{}\" is not idiomatic. Enum variant names should be \
                 ClassCase, like \"{}\".",
                variant_name,
                to_class_case(variant_name)
            ),
            NonSnakeCaseFunctionName { name } => {
                write!(f,
                "Function name \"{}\" is not idiomatic. Function names should be snake_case, like \
                 \"{}\".",
                name,
                to_snake_case(name)
            )
            }
            LossOfPrecision {
                initial_type,
                cast_to,
            } => write!(
                f,
                "This cast, from type {} to type {}, will lose precision.",
                initial_type.friendly_type_str(),
                cast_to.friendly_type_str()
            ),
            UnusedReturnValue { r#type } => write!(
                f,
                "This returns a value of type {}, which is not assigned to anything and is \
                 ignored.",
                r#type.friendly_type_str()
            ),
            SimilarMethodFound { lib, module, name } => write!(
                f,
                "A method with the same name was found for type {} in dependency \"{}::{}\". \
                 Traits must be in scope in order to access their methods. ",
                name, lib, module
            ),
            OverridesOtherSymbol { name } => write!(
                f,
                "This import would override another symbol with the same name \"{}\" in this \
                 namespace.",
                name
            ),
            OverridingTraitImplementation => write!(
                f,
                "This trait implementation overrides another one that was previously defined."
            ),
            DeadDeclaration => write!(f, "This declaration is never used."),
            DeadStructDeclaration => write!(f, "This struct is never instantiated."),
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
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum CompileError<'sc> {
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariable { var_name: String, span: Span<'sc> },
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariablePath { var_name: &'sc str, span: Span<'sc> },
    #[error("Function \"{name}\" does not exist in this scope.")]
    UnknownFunction { name: &'sc str, span: Span<'sc> },
    #[error("Identifier \"{name}\" was used as a variable, but it is actually a {what_it_is}.")]
    NotAVariable {
        name: String,
        span: Span<'sc>,
        what_it_is: &'static str,
    },
    #[error(
        "Identifier \"{name}\" was called as if it was a function, but it is actually a \
         {what_it_is}."
    )]
    NotAFunction {
        name: String,
        span: Span<'sc>,
        what_it_is: &'static str,
    },
    #[error("Unimplemented feature: {0}")]
    Unimplemented(&'static str, Span<'sc>),
    #[error("{0}")]
    TypeError(TypeError<'sc>),
    #[error("Error parsing input: expected {err:?}")]
    ParseFailure {
        span: Span<'sc>,
        err: pest::error::Error<Rule>,
    },
    #[error(
        "Invalid top-level item: {0:?}. A program should consist of a contract, script, or \
         predicate at the top level."
    )]
    InvalidTopLevelItem(Rule, Span<'sc>),
    #[error(
        "Internal compiler error: {0}\nPlease file an issue on the repository and include the \
         code that triggered this error."
    )]
    Internal(&'static str, Span<'sc>),
    #[error("Unimplemented feature: {0:?}")]
    UnimplementedRule(Rule, Span<'sc>),
    #[error(
        "Byte literal had length of {byte_length}. Byte literals must be either one byte long (8 \
         binary digits or 2 hex digits) or 32 bytes long (256 binary digits or 64 hex digits)"
    )]
    InvalidByteLiteralLength { byte_length: usize, span: Span<'sc> },
    #[error("Expected an expression to follow operator \"{op}\"")]
    ExpectedExprAfterOp { op: &'sc str, span: Span<'sc> },
    #[error("Expected an operator, but \"{op}\" is not a recognized operator. ")]
    ExpectedOp { op: &'sc str, span: Span<'sc> },
    #[error(
        "Where clause was specified but there are no generic type parameters. Where clauses can \
         only be applied to generic type parameters."
    )]
    UnexpectedWhereClause(Span<'sc>),
    #[error(
        "Specified generic type in where clause \"{type_name}\" not found in generic type \
         arguments of function."
    )]
    UndeclaredGenericTypeInWhereClause {
        type_name: &'sc str,
        span: Span<'sc>,
    },
    #[error(
        "Program contains multiple contracts. A valid program should only contain at most one \
         contract."
    )]
    MultipleContracts(Span<'sc>),
    #[error(
        "Program contains multiple scripts. A valid program should only contain at most one \
         script."
    )]
    MultipleScripts(Span<'sc>),
    #[error(
        "Program contains multiple predicates. A valid program should only contain at most one \
         predicate."
    )]
    MultiplePredicates(Span<'sc>),
    #[error(
        "Trait constraint was applied to generic type that is not in scope. Trait \
         \"{trait_name}\" cannot constrain type \"{type_name}\" because that type does not exist \
         in this scope."
    )]
    ConstrainedNonExistentType {
        trait_name: &'sc str,
        type_name: &'sc str,
        span: Span<'sc>,
    },
    #[error(
        "Predicate definition contains multiple main functions. Multiple functions in the same \
         scope cannot have the same name."
    )]
    MultiplePredicateMainFunctions(Span<'sc>),
    #[error(
        "Predicate declaration contains no main function. Predicates require a main function."
    )]
    NoPredicateMainFunction(Span<'sc>),
    #[error("A predicate's main function must return a boolean.")]
    PredicateMainDoesNotReturnBool(Span<'sc>),
    #[error("Script declaration contains no main function. Scripts require a main function.")]
    NoScriptMainFunction(Span<'sc>),
    #[error(
        "Script definition contains multiple main functions. Multiple functions in the same scope \
         cannot have the same name."
    )]
    MultipleScriptMainFunctions(Span<'sc>),
    #[error(
        "Attempted to reassign to a symbol that is not a variable. Symbol {name} is not a mutable \
         variable, it is a {kind}."
    )]
    ReassignmentToNonVariable {
        name: &'sc str,
        kind: &'sc str,
        decl_span: Span<'sc>,
        usage_span: Span<'sc>,
    },
    #[error("Assignment to immutable variable. Variable {name} is not declared as mutable.")]
    AssignmentToNonMutable {
        name: &'sc str,
        decl_span: Span<'sc>,
        usage_span: Span<'sc>,
    },
    #[error(
        "Generic type \"{name}\" is not in scope. Perhaps you meant to specify type parameters in \
         the function signature? For example: \n`fn \
         {fn_name}<{comma_separated_generic_params}>({args}) -> ... `"
    )]
    TypeParameterNotInTypeScope {
        name: &'sc str,
        span: Span<'sc>,
        comma_separated_generic_params: String,
        fn_name: &'sc str,
        args: String,
    },
    #[error(
        "Asm opcode has multiple immediates specified, when any opcode has at most one immediate."
    )]
    MultipleImmediates(Span<'sc>),
    #[error(
        "Expected type {expected}, but found type {given}. The definition of this function must \
         match the one in the trait declaration."
    )]
    MismatchedTypeInTrait {
        span: Span<'sc>,
        given: String,
        expected: String,
    },
    #[error("\"{name}\" is not a trait, so it cannot be \"impl'd\". ")]
    NotATrait { span: Span<'sc>, name: &'sc str },
    #[error("Trait \"{name}\" cannot be found in the current scope.")]
    UnknownTrait { span: Span<'sc>, name: &'sc str },
    #[error("Function \"{name}\" is not a part of trait \"{trait_name}\"'s interface surface.")]
    FunctionNotAPartOfInterfaceSurface {
        name: &'sc str,
        trait_name: String,
        span: Span<'sc>,
    },
    #[error("Functions are missing from this trait implementation: {missing_functions}")]
    MissingInterfaceSurfaceMethods {
        missing_functions: String,
        span: Span<'sc>,
    },
    #[error("Expected {expected} type arguments, but instead found {given}.")]
    IncorrectNumberOfTypeArguments {
        given: usize,
        expected: usize,
        span: Span<'sc>,
    },
    #[error(
        "Struct with name \"{name}\" could not be found in this scope. Perhaps you need to import \
         it?"
    )]
    StructNotFound { name: &'sc str, span: Span<'sc> },
    #[error(
        "The name \"{name}\" does not refer to a struct, but this is an attempted struct \
         declaration."
    )]
    DeclaredNonStructAsStruct { name: &'sc str, span: Span<'sc> },
    #[error(
        "Attempted to access field \"{field_name}\" of non-struct \"{name}\". Field accesses are \
         only valid on structs."
    )]
    AccessedFieldOfNonStruct {
        field_name: &'sc str,
        name: &'sc str,
        span: Span<'sc>,
    },
    #[error(
        "Attempted to access a method on something that has no methods. \"{name}\" is a {thing}, \
         not a type with methods."
    )]
    MethodOnNonValue {
        name: &'sc str,
        thing: &'sc str,
        span: Span<'sc>,
    },
    #[error("Initialization of struct \"{struct_name}\" is missing field \"{field_name}\".")]
    StructMissingField {
        field_name: &'sc str,
        struct_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Struct \"{struct_name}\" does not have field \"{field_name}\".")]
    StructDoesNotHaveField {
        field_name: &'sc str,
        struct_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("No method named \"{method_name}\" found for type \"{type_name}\".")]
    MethodNotFound {
        span: Span<'sc>,
        method_name: String,
        type_name: String,
    },
    #[error("The asterisk, if present, must be the last part of a path. E.g., `use foo::bar::*`.")]
    NonFinalAsteriskInPath { span: Span<'sc> },
    #[error("Module \"{name}\" could not be found.")]
    ModuleNotFound { span: Span<'sc>, name: String },
    #[error("\"{name}\" is a {actually}, not a struct. Fields can only be accessed on structs.")]
    NotAStruct {
        name: String,
        span: Span<'sc>,
        actually: String,
    },
    #[error(
        "Field \"{field_name}\" not found on struct \"{struct_name}\". Available fields are:\n \
         {available_fields}"
    )]
    FieldNotFound {
        field_name: &'sc str,
        available_fields: String,
        struct_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Could not find symbol \"{name}\" in this scope.")]
    SymbolNotFound { span: Span<'sc>, name: &'sc str },
    #[error(
        "Because this if expression's value is used, an \"else\" branch is required and it must \
         return type \"{r#type}\""
    )]
    NoElseBranch { span: Span<'sc>, r#type: String },
    #[error("Use of type `Self` outside of a context in which `Self` refers to a type.")]
    UnqualifiedSelfType { span: Span<'sc> },
    #[error(
        "Symbol \"{name}\" does not refer to a type, it refers to a {actually_is}. It cannot be \
         used in this position."
    )]
    NotAType {
        span: Span<'sc>,
        name: String,
        actually_is: &'sc str,
    },
    #[error(
        "This enum variant requires an instantiation expression. Try initializing it with \
         arguments in parentheses."
    )]
    MissingEnumInstantiator { span: Span<'sc> },
    #[error(
        "This path must return a value of type \"{ty}\" from function \"{function_name}\", but it \
         does not."
    )]
    PathDoesNotReturn {
        span: Span<'sc>,
        ty: String,
        function_name: &'sc str,
    },
    #[error("Expected block to implicitly return a value of type \"{ty}\".")]
    ExpectedImplicitReturnFromBlockWithType { span: Span<'sc>, ty: String },
    #[error("Expected block to implicitly return a value.")]
    ExpectedImplicitReturnFromBlock { span: Span<'sc> },
    #[error(
        "This register was not initialized in the initialization section of the ASM expression. \
         Initialized registers are: {initialized_registers}"
    )]
    UnknownRegister {
        span: Span<'sc>,
        initialized_registers: String,
    },
    #[error("This opcode takes an immediate value but none was provided.")]
    MissingImmediate { span: Span<'sc> },
    #[error("This immediate value is invalid.")]
    InvalidImmediateValue { span: Span<'sc> },
    #[error(
        "This expression was expected to return a value but no return register was specified. \
         Provide a register in the implicit return position of this asm expression to return it."
    )]
    InvalidAssemblyMismatchedReturn { span: Span<'sc> },
    #[error("Variant \"{variant_name}\" does not exist on enum \"{enum_name}\"")]
    UnknownEnumVariant {
        enum_name: &'sc str,
        variant_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Unknown opcode: \"{op_name}\".")]
    UnrecognizedOp { op_name: &'sc str, span: Span<'sc> },
    #[error("Unknown type \"{ty}\".")]
    TypeMustBeKnown { ty: String, span: Span<'sc> },
    #[error("The value \"{val}\" is too large to fit in this 6-bit immediate spot.")]
    Immediate06TooLarge { val: u64, span: Span<'sc> },
    #[error("The value \"{val}\" is too large to fit in this 12-bit immediate spot.")]
    Immediate12TooLarge { val: u64, span: Span<'sc> },
    #[error("The value \"{val}\" is too large to fit in this 18-bit immediate spot.")]
    Immediate18TooLarge { val: u64, span: Span<'sc> },
    #[error("The value \"{val}\" is too large to fit in this 24-bit immediate spot.")]
    Immediate24TooLarge { val: u64, span: Span<'sc> },
    #[error("The opcode \"jnei\" is not valid in inline assembly. Use an enclosing if expression instead.")]
    DisallowedJnei { span: Span<'sc> },
    #[error(
        "The opcode \"ji\" is not valid in inline assembly. Try using function calls instead."
    )]
    DisallowedJi { span: Span<'sc> },
    #[error(
        "The opcode \"lw\" is not valid in inline assembly. Try assigning a static value to a variable instead."
    )]
    DisallowedLw { span: Span<'sc> },
    #[error(
        "This op expects {expected} register(s) as arguments, but you provided {received} register(s)."
    )]
    IncorrectNumberOfAsmRegisters {
        span: Span<'sc>,
        expected: usize,
        received: usize,
    },
    #[error("This op does not take an immediate value.")]
    UnnecessaryImmediate { span: Span<'sc> },
    #[error("This reference is ambiguous, and could refer to either a module or an enum of the same name. Try qualifying the name with a path.")]
    AmbiguousPath { span: Span<'sc> },
    #[error("This value is not valid within a \"str\" type.")]
    InvalidStrType { raw: String, span: Span<'sc> },
    #[error("Unknown type name.")]
    UnknownType { span: Span<'sc> },
    #[error("Bytecode can only support programs with up to 2^12 words worth of opcodes. Try refactoring into contract calls? This is a temporary error and will be implemented in the future.")]
    TooManyInstructions { span: Span<'sc> },
    #[error(
        "No valid {} file (.{}) was found at {file_path}",
        crate::constants::LANGUAGE_NAME,
        crate::constants::DEFAULT_FILE_EXTENSION
    )]
    FileNotFound { span: Span<'sc>, file_path: String },
    #[error("The file {file_path} could not be read: {stringified_error}")]
    FileCouldNotBeRead {
        span: Span<'sc>,
        file_path: String,
        stringified_error: String,
    },
    #[error("This imported file must be a library. It must start with \"library <name>\", where \"name\" is the name of the library this file contains.")]
    ImportMustBeLibrary { span: Span<'sc> },
    #[error("An enum instantiaton cannot contain more than one value. This should be a single value of type {ty}.")]
    MoreThanOneEnumInstantiator { span: Span<'sc>, ty: String },
    #[error("This enum variant represents the unit type, so it should not be instantiated with any value.")]
    UnnecessaryEnumInstantiator { span: Span<'sc> },
    #[error("Trait \"{name}\" does not exist in this scope.")]
    TraitNotFound { name: &'sc str, span: Span<'sc> },
    #[error("This expression is not valid on the left hand side of a reassignment.")]
    InvalidExpressionOnLhs { span: Span<'sc> },
    #[error(
        "Function \"{method_name}\" expects {expected} arguments but you provided {received}."
    )]
    TooManyArgumentsForFunction {
        decl_span: Span<'sc>,
        usage_span: Span<'sc>,
        method_name: &'sc str,
        expected: usize,
        received: usize,
    },
    #[error(
        "Function \"{method_name}\" expects {expected} arguments but you provided {received}."
    )]
    TooFewArgumentsForFunction {
        decl_span: Span<'sc>,
        usage_span: Span<'sc>,
        method_name: &'sc str,
        expected: usize,
        received: usize,
    },
    #[error("This type is invalid in a function selector. A contract ABI function selector must be a known sized type, not generic.")]
    InvalidAbiType { span: Span<'sc> },
    #[error("An ABI function must accept exactly four arguments.")]
    InvalidNumberOfAbiParams { span: Span<'sc> },
    #[error("This is a {actually_is}, not an ABI. An ABI cast requires a valid ABI to cast the address to.")]
    NotAnAbi {
        span: Span<'sc>,
        actually_is: &'static str,
    },
    #[error("An ABI can only be implemented for the `Contract` type, so this implementation of an ABI for type \"{ty}\" is invalid.")]
    ImplAbiForNonContract { span: Span<'sc>, ty: String },
    #[error("The trait function \"{fn_name}\" in trait \"{trait_name}\" expects {num_args} arguments, but the provided implementation only takes {provided_args} arguments.")]
    IncorrectNumberOfInterfaceSurfaceFunctionParameters {
        fn_name: &'sc str,
        trait_name: &'sc str,
        num_args: usize,
        provided_args: usize,
        span: Span<'sc>,
    },
    #[error("For now, ABI functions must take exactly four parameters, in this order: gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, <your_function_parameter>: ?")]
    AbiFunctionRequiresSpecificSignature { span: Span<'sc> },
    #[error("This parameter was declared as type {should_be}, but argument of type {provided} was provided.")]
    ArgumentParameterTypeMismatch {
        span: Span<'sc>,
        should_be: String,
        provided: String,
    },
    #[error("Function {fn_name} is recursive, which is unsupported at this time.")]
    RecursiveCall { fn_name: &'sc str, span: Span<'sc> },
    #[error(
        "Function {fn_name} is recursive via {call_chain}, which is unsupported at this time."
    )]
    RecursiveCallChain {
        fn_name: &'sc str,
        call_chain: String, // Pretty list of symbols, e.g., "a, b and c".
        span: Span<'sc>,
    },
    #[error("File {file_path} generates an infinite dependency cycle.")]
    InfiniteDependencies { file_path: String, span: Span<'sc> },
}

impl<'sc> std::convert::From<TypeError<'sc>> for CompileError<'sc> {
    fn from(other: TypeError<'sc>) -> CompileError<'sc> {
        CompileError::TypeError(other)
    }
}

#[derive(Error, Debug, Clone)]
pub enum TypeError<'sc> {
    #[error(
        "Mismatched types: Expected type {expected} but found type {received}. Type {received} is \
         not castable to type {expected}.\n help: {help_text}"
    )]
    MismatchedType {
        expected: String,
        received: String,
        help_text: String,
        span: Span<'sc>,
    },
}

impl<'sc> TypeError<'sc> {
    pub(crate) fn internal_span(&self) -> &Span<'sc> {
        use TypeError::*;
        match self {
            MismatchedType { span, .. } => span,
        }
    }
}

impl<'sc> CompileError<'sc> {
    pub fn to_friendly_error_string(&self) -> String {
        match self {
            CompileError::ParseFailure { err, .. } => format!(
                "Error parsing input: {}",
                match &err.variant {
                    pest::error::ErrorVariant::ParsingError {
                        positives,
                        negatives,
                    } => {
                        let mut buf = String::new();
                        if !positives.is_empty() {
                            buf.push_str(&format!(
                                "expected one of [{}]",
                                positives
                                    .iter()
                                    .map(|x| format!("{:?}", x))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                        if !negatives.is_empty() {
                            buf.push_str(&format!(
                                "did not expect any of [{}]",
                                negatives
                                    .iter()
                                    .map(|x| format!("{:?}", x))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                        buf
                    }
                    pest::error::ErrorVariant::CustomError { message } => message.to_string(),
                }
            ),
            a => format!("{}", a),
        }
    }

    pub fn span(&self) -> (usize, usize) {
        let sp = self.internal_span();
        (sp.start(), sp.end())
    }

    pub fn path(&self) -> String {
        self.internal_span().path()
    }

    pub fn internal_span(&self) -> &Span<'sc> {
        use CompileError::*;
        match self {
            UnknownVariable { span, .. } => span,
            UnknownVariablePath { span, .. } => span,
            UnknownFunction { span, .. } => span,
            NotAVariable { span, .. } => span,
            NotAFunction { span, .. } => span,
            Unimplemented(_, span) => span,
            TypeError(err) => err.internal_span(),
            ParseFailure { span, .. } => span,
            InvalidTopLevelItem(_, span) => span,
            Internal(_, span) => span,
            UnimplementedRule(_, span) => span,
            InvalidByteLiteralLength { span, .. } => span,
            ExpectedExprAfterOp { span, .. } => span,
            ExpectedOp { span, .. } => span,
            UnexpectedWhereClause(span) => span,
            UndeclaredGenericTypeInWhereClause { span, .. } => span,
            MultiplePredicates(span) => span,
            MultipleScripts(span) => span,
            MultipleContracts(span) => span,
            ConstrainedNonExistentType { span, .. } => span,
            MultiplePredicateMainFunctions(span) => span,
            NoPredicateMainFunction(span) => span,
            PredicateMainDoesNotReturnBool(span) => span,
            NoScriptMainFunction(span) => span,
            MultipleScriptMainFunctions(span) => span,
            ReassignmentToNonVariable { usage_span, .. } => usage_span,
            AssignmentToNonMutable { usage_span, .. } => usage_span,
            TypeParameterNotInTypeScope { span, .. } => span,
            MultipleImmediates(span) => span,
            MismatchedTypeInTrait { span, .. } => span,
            NotATrait { span, .. } => span,
            UnknownTrait { span, .. } => span,
            FunctionNotAPartOfInterfaceSurface { span, .. } => span,
            MissingInterfaceSurfaceMethods { span, .. } => span,
            IncorrectNumberOfTypeArguments { span, .. } => span,
            StructNotFound { span, .. } => span,
            DeclaredNonStructAsStruct { span, .. } => span,
            AccessedFieldOfNonStruct { span, .. } => span,
            MethodOnNonValue { span, .. } => span,
            StructMissingField { span, .. } => span,
            StructDoesNotHaveField { span, .. } => span,
            MethodNotFound { span, .. } => span,
            NonFinalAsteriskInPath { span, .. } => span,
            ModuleNotFound { span, .. } => span,
            NotAStruct { span, .. } => span,
            FieldNotFound { span, .. } => span,
            SymbolNotFound { span, .. } => span,
            NoElseBranch { span, .. } => span,
            UnqualifiedSelfType { span, .. } => span,
            NotAType { span, .. } => span,
            MissingEnumInstantiator { span, .. } => span,
            PathDoesNotReturn { span, .. } => span,
            ExpectedImplicitReturnFromBlockWithType { span, .. } => span,
            ExpectedImplicitReturnFromBlock { span, .. } => span,
            UnknownRegister { span, .. } => span,
            MissingImmediate { span, .. } => span,
            InvalidImmediateValue { span, .. } => span,
            InvalidAssemblyMismatchedReturn { span, .. } => span,
            UnknownEnumVariant { span, .. } => span,
            UnrecognizedOp { span, .. } => span,
            TypeMustBeKnown { span, .. } => span,
            Immediate06TooLarge { span, .. } => span,
            Immediate12TooLarge { span, .. } => span,
            Immediate18TooLarge { span, .. } => span,
            Immediate24TooLarge { span, .. } => span,
            DisallowedJnei { span, .. } => span,
            DisallowedJi { span, .. } => span,
            DisallowedLw { span, .. } => span,
            IncorrectNumberOfAsmRegisters { span, .. } => span,
            UnnecessaryImmediate { span, .. } => span,
            AmbiguousPath { span, .. } => span,
            UnknownType { span, .. } => span,
            InvalidStrType { span, .. } => span,
            TooManyInstructions { span, .. } => span,
            FileNotFound { span, .. } => span,
            FileCouldNotBeRead { span, .. } => span,
            ImportMustBeLibrary { span, .. } => span,
            MoreThanOneEnumInstantiator { span, .. } => span,
            UnnecessaryEnumInstantiator { span, .. } => span,
            TraitNotFound { span, .. } => span,
            InvalidExpressionOnLhs { span, .. } => span,
            TooManyArgumentsForFunction { usage_span, .. } => usage_span,
            TooFewArgumentsForFunction { usage_span, .. } => usage_span,
            InvalidAbiType { span, .. } => span,
            InvalidNumberOfAbiParams { span, .. } => span,
            NotAnAbi { span, .. } => span,
            ImplAbiForNonContract { span, .. } => span,
            IncorrectNumberOfInterfaceSurfaceFunctionParameters { span, .. } => span,
            AbiFunctionRequiresSpecificSignature { span, .. } => span,
            ArgumentParameterTypeMismatch { span, .. } => span,
            RecursiveCall { span, .. } => span,
            RecursiveCallChain { span, .. } => span,
            InfiniteDependencies { span, .. } => span,
        }
    }

    /// Returns the line and column start and end
    pub fn line_col(&self) -> (LineCol, LineCol) {
        (
            self.internal_span().start_pos().line_col().into(),
            self.internal_span().end_pos().line_col().into(),
        )
    }

    pub fn format(&self, fmt: &mut Formatter) -> source_span::fmt::Formatted {
        match self {
            CompileError::AssignmentToNonMutable {
                name,
                decl_span,
                usage_span,
            } => self.format_one_hint_one_err(
                fmt,
                decl_span,
                format!(
                    "Variable {} not declared as mutable. Try adding 'mut'.",
                    name
                ),
                usage_span,
                format!("Assignment to immutable variable {}.", name),
            ),
            CompileError::TooFewArgumentsForFunction {
                decl_span,
                usage_span,
                method_name,
                expected,
                received,
            } => self.format_one_hint_one_err(
                fmt,
                decl_span,
                format!("Function {} declared here.", method_name),
                usage_span,
                format!(
                    "Function {} expected {} arguments and recieved {}.",
                    method_name, expected, received
                ),
            ),
            CompileError::TooManyArgumentsForFunction {
                decl_span,
                usage_span,
                method_name,
                expected,
                received,
            } => self.format_one_hint_one_err(
                fmt,
                decl_span,
                format!("Function {} declared here.", method_name),
                usage_span,
                format!(
                    "Function {} expected {} arguments and recieved {}.",
                    method_name, expected, received
                ),
            ),
            CompileError::ReassignmentToNonVariable {
                name,
                kind,
                decl_span,
                usage_span,
            } => self.format_one_hint_one_err(
                fmt,
                decl_span,
                format!("Symbol {} declared here.", name),
                usage_span,
                format!("Attempted to reassign to a symbol that is not a variable. Symbol {} is not a mutable \
                variable, it is a {}.", name, kind)
            ),
            _ => self.format_err_simple(fmt),
        }
    }

    fn format_err_simple(&self, fmt: &mut Formatter) -> source_span::fmt::Formatted {
        let input = self.internal_span().input();
        let chars = input.chars().map(Result::<_, String>::Ok);

        let metrics = source_span::DEFAULT_METRICS;
        let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);

        for c in buffer.iter() {
            let _ = c.unwrap(); // report eventual errors.
        }

        let (start_pos, end_pos) = self.span();
        let lookup = LineColLookup::new(input);
        let (start_line, start_col) = lookup.get(start_pos);
        let (end_line, end_col) = lookup.get(if end_pos == 0 { 0 } else { end_pos - 1 });

        let err_start = Position::new(start_line - 1, start_col - 1);
        let err_end = Position::new(end_line - 1, end_col - 1);
        let err_span = source_span::Span::new(err_start, err_end, err_end.next_column());
        fmt.add(
            err_span,
            Some(self.to_friendly_error_string()),
            Style::Error,
        );

        fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap()
    }

    fn format_one_hint_one_err(
        &self,
        fmt: &mut Formatter,
        hint_span: &Span<'sc>,
        hint_message: String,
        err_span: &Span<'sc>,
        err_message: String,
    ) -> source_span::fmt::Formatted {
        self.format_one(fmt, hint_span.clone(), Style::Note, hint_message);
        self.format_one(fmt, err_span.clone(), Style::Error, err_message);

        let span = crate::utils::join_spans(hint_span.clone(), err_span.clone());
        let input = span.input();
        let chars = input.chars().map(Result::<_, String>::Ok);
        let metrics = source_span::DEFAULT_METRICS;
        let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);
        for c in buffer.iter() {
            let _ = c.unwrap(); // report eventual errors.
        }

        fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap()
    }

    fn format_one(
        &self,
        fmt: &mut Formatter,
        span: Span<'sc>,
        style: source_span::fmt::Style,
        friendly_string: String,
    ) {
        let input = span.input();
        let (start_pos, end_pos) = (span.start(), span.end());
        let lookup = LineColLookup::new(input);
        let (start_line, start_col) = lookup.get(start_pos);
        let (end_line, end_col) = lookup.get(if end_pos == 0 { 0 } else { end_pos - 1 });

        let err_start = Position::new(start_line - 1, start_col - 1);
        let err_end = Position::new(end_line - 1, end_col - 1);
        let err_span = source_span::Span::new(err_start, err_end, err_end.next_column());
        fmt.add(err_span, Some(friendly_string), style);
    }
}
