#[macro_use]
extern crate pest_derive;

mod error;
mod parser;
pub use error::CompileError;
use parser::HllParser;
use parser::Rule;
use pest::Parser;

type AST = ();

pub fn parse(input: &str) -> Result<AST, CompileError> {
    let parsed = HllParser::parse(Rule::program, input)?;
    dbg!(parsed);
    todo!()
}

#[test]
fn test_basic_prog() {
    let prog = parse(
        r#"
    use stdlib::println

    fn prints_number_five() {
        let x = 5
        println(x)
x.to_string()
    }"#,
    );
    dbg!(&prog);
    prog.unwrap()
}
