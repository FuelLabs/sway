pub mod assignable;
pub mod attribute;
pub mod brackets;
pub mod dependency;
mod error;
pub mod expr;
pub mod generics;
pub mod intrinsics;
mod item;
pub mod keywords;
mod literal;
pub mod module;
pub mod parse;
pub mod parser;
pub mod path;
pub mod pattern;
mod priv_prelude;
pub mod punctuated;
pub mod statement;
pub mod token;
pub mod ty;
pub mod where_clause;

pub use crate::{
    assignable::Assignable,
    attribute::AttributeDecl,
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
    intrinsics::*,
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
        FnArg, FnArgs, FnSignature, Item, ItemKind, TypeField,
    },
    keywords::{DoubleColonToken, PubToken},
    literal::{LitInt, LitIntType, Literal},
    module::{Module, ModuleKind},
    parse::Parse,
    parser::Parser,
    path::{PathExpr, PathExprSegment, PathType, PathTypeSegment, QualifiedPathRoot},
    pattern::{Pattern, PatternStructField},
    statement::{Statement, StatementLet},
    token::{lex, lex_commented},
    token::LexError,
    ty::Ty,
    where_clause::{WhereBound, WhereClause},
};

use crate::priv_prelude::*;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, PartialEq, Hash, Error)]
pub enum ParseFileError {
    #[error(transparent)]
    Lex(LexError),
    #[error("Unable to parse: {}", .0.iter().map(|x| x.kind.to_string()).collect::<Vec<String>>().join("\n"))]
    Parse(Vec<ParseError>),
}

pub fn parse_file(src: Arc<str>, path: Option<Arc<PathBuf>>) -> Result<Module, ParseFileError> {
    let token_stream = match lex(&src, 0, src.len(), path) {
        Ok(token_stream) => token_stream,
        Err(error) => return Err(ParseFileError::Lex(error)),
    };
    let mut errors = Vec::new();
    let parser = Parser::new(&token_stream, &mut errors);
    let module = match parser.parse_to_end() {
        Ok((module, _parser_consumed)) => module,
        Err(_error_emitted) => return Err(ParseFileError::Parse(errors)),
    };
    Ok(module)
}
