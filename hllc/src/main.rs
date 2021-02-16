#![allow(warnings)]
use line_col::LineColLookup;
use parser::parse;
use source_span::{
    fmt::{Color, Formatter, Style},
    Position, SourceBuffer, Span, DEFAULT_METRICS,
};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::path::PathBuf;
use structopt::StructOpt;
use termcolor::{BufferWriter, Color as TermColor, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(short = "o", parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let opt = Opt::from_args();
    let content = fs::read_to_string(opt.input.clone())?;

    let res = parse(&content);

    match res {
        Ok((compiled, warnings)) => {
            if let Some(output) = opt.output {
                let mut file = File::create(output)?;
                file.write_all(format!("{:#?}", compiled).as_bytes())?;
            } else {
                println!("{:#?}", compiled);
            }
            for ref warning in warnings.iter() {
                format_warning(&content, warning);
            }
            if warnings.is_empty() {
                write_green(&format!("Successfully compiled {:?}", opt.input));
            } else {
                write_yellow(&format!("Compiled {:?} with warnings.", opt.input));
            }
        }
        Err(e) => format_err(&content, e),
    }

    Ok(())
}
fn format_warning(input: &str, err: &parser::CompileWarning) {
    let metrics = DEFAULT_METRICS;
    let chars = input.chars().map(|x| -> Result<_, ()> { Ok(x) });

    let metrics = source_span::DEFAULT_METRICS;
    let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);

    let mut fmt = Formatter::with_margin_color(Color::Blue);

    for c in buffer.iter() {
        let c = c.unwrap(); // report eventual errors.
    }

    let (start_pos, end_pos) = err.span();
    let lookup = LineColLookup::new(input);
    let (start_line, start_col) = lookup.get(start_pos);
    let (end_line, end_col) = lookup.get(end_pos - 1);

    let err_start = Position::new(start_line - 1, start_col - 1);
    let err_end = Position::new(end_line - 1, end_col - 1);
    let err_span = Span::new(err_start, err_end, err_end.next_column());
    fmt.add(
        err_span,
        Some(err.to_friendly_warning_string()),
        Style::Warning,
    );

    let formatted = fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap();
    fmt.add(
        buffer.span(),
        Some("this is the whole program\nwhat a nice program!".to_string()),
        Style::Error,
    );

    println!("{}", formatted);
}

fn format_err(input: &str, err: parser::CompileError) {
    let metrics = DEFAULT_METRICS;
    let chars = input.chars().map(|x| -> Result<_, ()> { Ok(x) });

    let metrics = source_span::DEFAULT_METRICS;
    let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);

    let mut fmt = Formatter::with_margin_color(Color::Blue);

    for c in buffer.iter() {
        let c = c.unwrap(); // report eventual errors.
    }

    let (start_pos, end_pos) = err.span();
    let lookup = LineColLookup::new(input);
    let (start_line, start_col) = lookup.get(start_pos);
    let (end_line, end_col) = lookup.get(end_pos - 1);

    let err_start = Position::new(start_line - 1, start_col - 1);
    let err_end = Position::new(end_line - 1, end_col - 1);
    let err_span = Span::new(err_start, err_end, err_end.next_column());
    fmt.add(err_span, Some(err.to_friendly_error_string()), Style::Error);

    let formatted = fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap();
    fmt.add(
        buffer.span(),
        Some("this is the whole program\nwhat a nice program!".to_string()),
        Style::Error,
    );

    println!("{}", formatted);
    write_red("Aborting due to previous error.").unwrap();
}

fn write_red(txt: &str) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Red)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)
}

fn write_green(txt: &str) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Green)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)
}

fn write_yellow(txt: &str) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Yellow)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)
}
