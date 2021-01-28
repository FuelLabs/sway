use std::collections::HashMap;
use either::Either;
use crate::ast::Expression;

struct FunctionDeclaration<'sc> {
    name: &'sc str,
    body: CodeBlock<'sc>,
    parameters: Vec<FunctionParameter<'sc>>,
    span: pest::Span<'sc>,
}

struct FunctionParameter<'sc> {
    name: &'sc str,
    r#type: TypeInfo,
}

/// Type information without an associated value, used for type inferencing and definition.
enum TypeInfo {
    String,
    Integer,
    Boolean
}

struct CodeBlock<'sc> {
    scope: HashMap<&'sc str, Declaration<'sc>>,
    body: Vec<Box<dyn Assemblable>>
}

type Declaration<'sc> = ();

/// Represents a type's ability to be rendered into the target ASM
trait Assemblable {
    fn to_asm(&self) -> Asm;
}


type Asm = String; // TODO
