use crate::{
    fmt::{Format, FormattedCode, Formatter, FormatterError},
    utils::bracket::{CurlyBrace, Parenthesis},
};
use std::fmt::Write;
use sway_parse::{token::Delimiter, CodeBlockContents, FnArg, FnArgs, FnSignature, ItemFn};
use sway_types::Spanned;

impl Format for ItemFn {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.fn_signature.format(formatted_code, formatter)?;
        Self::open_curly_brace(formatted_code, formatter)?;
        self.body
            .clone()
            .into_inner()
            .format(formatted_code, formatter)?;
        Self::close_curly_brace(formatted_code, formatter)?;
        Ok(())
    }
}

impl Format for CodeBlockContents {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}

impl CurlyBrace for ItemFn {
    fn open_curly_brace(
        _line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
    fn close_curly_brace(
        _line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}

pub(crate) trait FormatSig {
    fn format(
        &self,
        formatted_code: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

impl FormatSig for FnSignature {
    fn format(
        &self,
        formatted_code: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // `pub `
        if let Some(visibility_token) = &self.visibility {
            write!(formatted_code, "{} ", visibility_token.span().as_str())?;
        }
        // `fn ` + name
        write!(
            formatted_code,
            "{} {}",
            self.fn_token.span().as_str(),
            self.name.as_str()
        )?;
        // `<T>`
        if let Some(generics) = &self.generics.clone() {
            generics.format(formatted_code, formatter)?;
        }
        // `(`
        Self::open_parenthesis(formatted_code, formatter)?;
        // FnArgs
        match self.arguments.clone().into_inner() {
            FnArgs::Static(args) => {
                // TODO: Refactor into `Punctuated::format()`
                args.format(formatted_code, formatter)?;
            }
            FnArgs::NonStatic {
                self_token,
                mutable_self,
                args_opt,
            } => {
                // mut
                if let Some(mut_token) = mutable_self {
                    write!(formatted_code, "{} ", mut_token.span().as_str())?;
                }
                // self
                formatted_code.push_str(self_token.span().as_str());
                // args
                if let Some(args) = args_opt {
                    // `, `
                    write!(formatted_code, "{} ", args.0.span().as_str())?;
                    args.1.format(formatted_code, formatter)?;
                }
            }
        }
        // `)`
        Self::close_parenthesis(formatted_code, formatter)?;
        // return_type_opt
        if let Some(return_type) = &self.return_type_opt {
            // TODO: fix when `ty` formatting is done
            write!(
                formatted_code,
                " {} {}",
                return_type.0.span().as_str(), // `->`
                return_type.1.span().as_str()  // `Ty`
            )?;
        }
        // `WhereClause`
        if let Some(where_clause) = &self.where_clause_opt {
            where_clause.format(formatted_code, formatter)?;
        }
        Ok(())
    }
}

// We will need to add logic to handle the case of long fn arguments, and break into new line
impl Parenthesis for FnSignature {
    fn open_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Parenthesis.as_open_char());
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Parenthesis.as_close_char());
        Ok(())
    }
}

// TODO: Use this in `Punctuated::format()`
impl Format for FnArg {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Ok(())
    }
}
