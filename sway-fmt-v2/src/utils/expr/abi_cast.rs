impl Format for AbiCastArgs {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.name.format(formatted_code, formatter)?;
        write!(formatted_code, "{} ", self.comma_token.span().as_str())?;
        self.address.format(formatted_code, formatter)?;

        Ok(())
    }
}

impl Parenthesis for AbiCastArgs {
    fn open_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_open_char())?;
        Ok(())
    }

    fn close_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_close_char())?;
        Ok(())
    }
}
