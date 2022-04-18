mod priv_prelude;
mod literal;
mod token;
mod error;
pub mod parser;
pub mod parse;
pub mod keywords;
pub mod program;
pub mod dependency;
mod item;
pub mod brackets;
pub mod punctuated;
pub mod ty;
pub mod expr;
pub mod pattern;
pub mod path;
pub mod generics;
pub mod where_clause;
pub mod statement;
pub mod assignable;

pub use crate::{
    error::{ParseError, ParseErrorKind},
    token::lex,
    parser::Parser,
    parse::Parse,
    program::{Program, ProgramKind},
    token::LexError,
    brackets::{AngleBrackets, Braces},
    dependency::Dependency,
    item::{
        Item, TypeField, FnArgs, FnSignature,
        item_use::{ItemUse, UseTree},
        item_struct::ItemStruct,
        item_enum::ItemEnum,
        item_fn::ItemFn,
        item_trait::{ItemTrait, Traits},
        item_impl::ItemImpl,
        item_abi::ItemAbi,
        item_const::ItemConst,
        item_storage::{ItemStorage, StorageField},
    },
    keywords::{PubToken, ImpureToken, DoubleColonToken},
    literal::{Literal, LitInt, LitIntType},
    generics::{GenericParams, GenericArgs},
    where_clause::{WhereClause, WhereBound},
    ty::Ty,
    assignable::Assignable,
    expr::{
        Expr, CodeBlockContents, IfExpr, IfCondition, AbiCastArgs,
        ExprArrayDescriptor, ExprTupleDescriptor, ExprStructField,
        MatchBranch, MatchBranchKind,
        asm::{AsmBlock, AsmRegisterDeclaration},
        op_code::Instruction,
    },
    statement::{Statement, StatementLet},
    path::{QualifiedPathRoot, PathType, PathTypeSegment, PathExpr, PathExprSegment},
    pattern::{Pattern, PatternStructField},
};

use std::{
    path::PathBuf,
    sync::Arc,
};

pub fn lex_and_parse<T>(src: &Arc<str>, start: usize, end: usize, path: Option<Arc<PathBuf>>) -> T
where
    T: Parse,
{
    let token_stream = lex(src, start, end, path).unwrap();
    let mut errors = Vec::new();
    let mut parser = Parser::new(&token_stream, &mut errors);
    let ret = parser.parse().unwrap();
    if !parser.is_empty() {
        panic!("not all tokens consumed");
    }
    ret
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ParseFileError {
    Lex(LexError),
    Parse(Vec<ParseError>),
}

pub fn parse_file(src: Arc<str>, path: Option<Arc<PathBuf>>) -> Result<Program, ParseFileError> {
    let token_stream = match lex(&src, 0, src.len(), path) {
        Ok(token_stream) => token_stream,
        Err(error) => return Err(ParseFileError::Lex(error)),
    };
    let mut errors = Vec::new();
    let parser = Parser::new(&token_stream, &mut errors);
    let program = match parser.parse_to_end() {
        Ok((program, _parser_consumed)) => program,
        Err(_error_emitted) => return Err(ParseFileError::Parse(errors)),
    };
    Ok(program)
}

