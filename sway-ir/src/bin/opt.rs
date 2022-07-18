use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Error, ErrorKind, Read, Write},
};

use sway_ir::{error::IrError, function::Function, optimize, Context};

// -------------------------------------------------------------------------------------------------

fn main() -> std::io::Result<()> {
    fn to_err<S: std::fmt::Display>(err: S) -> Error {
        Error::new(ErrorKind::Other, err.to_string())
    }

    // Build the config from the command line.
    let config = ConfigBuilder::build(std::env::args()).map_err(&to_err)?;

    // Read the input file, or standard in.
    let input_str = read_from_input(&config.input_path)?;

    // Parse it. XXX Improve this error message too.
    let mut ir = sway_ir::parser::parse(&input_str).map_err(&to_err)?;

    // Perform optimisation passes in order.
    for pass in config.passes {
        match pass.name.as_ref() {
            "inline" => perform_inline(&mut ir).map_err(&to_err)?,
            "constcombine" => perform_combine_constants(&mut ir).map_err(&to_err)?,
            _otherwise => unreachable!("Unknown pass name: {}", pass.name),
        };
    }

    // Write the output file or standard out.
    write_to_output(ir, &config.output_path)
}

fn read_from_input(path_str: &Option<String>) -> std::io::Result<String> {
    let mut input = Vec::new();
    match path_str {
        None => {
            BufReader::new(std::io::stdin()).read_to_end(&mut input)?;
        }
        Some(path_str) => {
            let file = std::fs::File::open(path_str)?;
            BufReader::new(file).read_to_end(&mut input)?;
        }
    }
    Ok(String::from_utf8_lossy(&input).to_string())
}

fn write_to_output<S: Into<String>>(ir_str: S, path_str: &Option<String>) -> std::io::Result<()> {
    match path_str {
        None => {
            println!("{}", ir_str.into());
            Ok(())
        }
        Some(path_str) => {
            let file = std::fs::File::create(path_str)?;
            BufWriter::new(file)
                .write(ir_str.into().as_ref())
                .map(|_| ())
        }
    }
}

// -------------------------------------------------------------------------------------------------

fn perform_inline(ir: &mut Context) -> Result<bool, IrError> {
    // For now we inline everything into `main()`.  Eventually we can be more selective.
    let main_fn = ir
        .functions
        .iter()
        .find_map(|(idx, fc)| if fc.name == "main" { Some(idx) } else { None })
        .unwrap();
    optimize::inline_all_function_calls(ir, &Function(main_fn))
}

// -------------------------------------------------------------------------------------------------

fn perform_combine_constants(ir: &mut Context) -> Result<bool, IrError> {
    let funcs = ir.functions.iter().map(|(idx, _)| idx).collect::<Vec<_>>();
    let mut modified = false;
    for idx in funcs {
        if optimize::combine_constants(ir, &Function(idx))? {
            modified = true;
        }
    }
    Ok(modified)
}

// -------------------------------------------------------------------------------------------------
// Using a bespoke CLI parser since the order in which passes are specified is important.

#[derive(Default)]
struct Config {
    input_path: Option<String>,
    output_path: Option<String>,

    _verify_each: bool,
    _time_passes: bool,
    _stats: bool,

    passes: Vec<Pass>,
}

#[derive(Default)]
struct Pass {
    name: String,
    #[allow(dead_code)]
    opts: HashMap<String, String>,
}

impl From<&str> for Pass {
    fn from(name: &str) -> Self {
        Pass {
            name: name.to_owned(),
            opts: HashMap::new(),
        }
    }
}

// This is a little clumsy in that it needs to consume items from the iterator carefully in each
// method to ensure we don't enter a weird state.
struct ConfigBuilder<I: Iterator<Item = String>> {
    next: Option<String>,
    rest: I,
    cfg: Config,
}

impl<I: Iterator<Item = String>> ConfigBuilder<I> {
    fn build(mut rest: I) -> Result<Config, String> {
        rest.next(); // Skip the first arg which is the binary name.
        let next = rest.next();
        ConfigBuilder {
            next,
            rest,
            cfg: Config::default(),
        }
        .build_root()
    }

    fn build_root(mut self) -> Result<Config, String> {
        match self.next {
            None => Ok(self.cfg),
            Some(opt) => {
                self.next = self.rest.next();
                match opt.as_str() {
                    "-i" => self.build_input(),
                    "-o" => self.build_output(),

                    "inline" => self.build_inline_pass(),
                    "constcombine" => self.build_const_combine_pass(),

                    _otherwise => Err(format!("Unrecognised option '{}'.", opt)),
                }
            }
        }
    }

    fn build_input(mut self) -> Result<Config, String> {
        match self.next {
            None => Err("-i option requires an argument.".to_owned()),
            Some(path) => {
                self.cfg.input_path = Some(path);
                self.next = self.rest.next();
                self.build_root()
            }
        }
    }

    fn build_output(mut self) -> Result<Config, String> {
        match self.next {
            None => Err("-o option requires an argument.".to_owned()),
            Some(path) => {
                self.cfg.output_path = Some(path);
                self.next = self.rest.next();
                self.build_root()
            }
        }
    }

    fn build_inline_pass(mut self) -> Result<Config, String> {
        // No args yet.  Eventually we should allow specifying which functions are to be inlined
        // or which functions are to have all embedded calls inlined.
        self.cfg.passes.push("inline".into());
        self.next = self.rest.next();
        self.build_root()
    }

    fn build_const_combine_pass(mut self) -> Result<Config, String> {
        // No args yet.  Eventually we should allow specifying which functions should have consts
        // combined.
        self.cfg.passes.push("constcombine".into());
        self.next = self.rest.next();
        self.build_root()
    }
}

// -------------------------------------------------------------------------------------------------
