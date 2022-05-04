pub mod assignable;
pub mod brackets;
pub mod dependency;
mod error;
pub mod expr;
pub mod generics;
mod item;
pub mod keywords;
mod literal;
pub mod parse;
pub mod parser;
pub mod path;
pub mod pattern;
mod priv_prelude;
pub mod program;
pub mod punctuated;
pub mod statement;
mod token;
pub mod ty;
pub mod where_clause;

pub use crate::{
    assignable::Assignable,
    brackets::{AngleBrackets, Braces},
    dependency::Dependency,
    error::{ParseError, ParseErrorKind},
    expr::{
        asm::{AsmBlock, AsmRegisterDeclaration},
        op_code::Instruction,
        AbiCastArgs, CodeBlockContents, Expr, ExprArrayDescriptor, ExprStructField,
        ExprTupleDescriptor, IfCondition, IfExpr, MatchBranch, MatchBranchKind,
    },
    generics::{GenericArgs, GenericParams},
    item::{
        item_abi::ItemAbi,
        item_const::ItemConst,
        item_enum::ItemEnum,
        item_fn::ItemFn,
        item_impl::ItemImpl,
        item_storage::{ItemStorage, StorageField},
        item_struct::ItemStruct,
        item_trait::{ItemTrait, Traits},
        item_use::{ItemUse, UseTree},
        FnArg, FnArgs, FnSignature, Item, TypeField,
    },
    keywords::{DoubleColonToken, ImpureToken, PubToken},
    literal::{LitInt, LitIntType, Literal},
    parse::Parse,
    parser::Parser,
    path::{PathExpr, PathExprSegment, PathType, PathTypeSegment, QualifiedPathRoot},
    pattern::{Pattern, PatternStructField},
    program::{Program, ProgramKind},
    statement::{Statement, StatementLet},
    token::lex,
    token::LexError,
    ty::Ty,
    where_clause::{WhereBound, WhereClause},
};

use std::{path::PathBuf, sync::Arc};

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
