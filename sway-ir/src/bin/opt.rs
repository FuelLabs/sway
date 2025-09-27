use std::{
    io::{BufReader, BufWriter, Read, Write},
    process::exit,
};

use anyhow::anyhow;
use sway_features::ExperimentalFeatures;
use sway_ir::{
    insert_after_each, register_known_passes, Backtrace, PassGroup, PassManager,
    MODULE_PRINTER_NAME, MODULE_VERIFIER_NAME,
};
use sway_types::SourceEngine;

// -------------------------------------------------------------------------------------------------

fn main() -> Result<(), anyhow::Error> {
    // Maintain a list of named pass functions for delegation.
    let mut pass_mgr = PassManager::default();
    register_known_passes(&mut pass_mgr);

    // Build the config from the command line.
    let config = ConfigBuilder::build(&pass_mgr, std::env::args())?;

    // Read the input file, or standard in.
    let input_str = read_from_input(&config.input_path)?;

    let source_engine = SourceEngine::default();

    // Parse it. XXX Improve this error message too.
    let mut ir = sway_ir::parser::parse(
        &input_str,
        &source_engine,
        ExperimentalFeatures::default(),
        Backtrace::default(),
    )?;

    // Perform optimisation passes in order.
    let mut passes = PassGroup::default();
    for pass in config.passes {
        passes.append_pass(pass);
    }
    if config.print_after_each {
        passes = insert_after_each(passes, MODULE_PRINTER_NAME);
    }
    if config.verify_after_each {
        passes = insert_after_each(passes, MODULE_VERIFIER_NAME);
    }
    pass_mgr.run(&mut ir, &passes)?;

    // Write the output file or standard out.
    write_to_output(ir, &config.output_path)?;

    Ok(())
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
// Using a bespoke CLI parser since the order in which passes are specified is important.

#[derive(Default)]
struct Config {
    input_path: Option<String>,
    output_path: Option<String>,

    verify_after_each: bool,
    print_after_each: bool,
    _time_passes: bool,
    _stats: bool,

    passes: Vec<&'static str>,
}

// This is a little clumsy in that it needs to consume items from the iterator carefully in each
// method to ensure we don't enter a weird state.
struct ConfigBuilder<'a, I: Iterator<Item = String>> {
    next: Option<String>,
    rest: I,
    cfg: Config,
    pass_mgr: &'a PassManager,
}

impl<I: Iterator<Item = String>> ConfigBuilder<'_, I> {
    fn build(pass_mgr: &PassManager, mut rest: I) -> Result<Config, anyhow::Error> {
        rest.next(); // Skip the first arg which is the binary name.
        let next = rest.next();
        ConfigBuilder {
            next,
            rest,
            cfg: Config::default(),
            pass_mgr,
        }
        .build_root()
    }

    fn build_root(mut self) -> Result<Config, anyhow::Error> {
        match self.next {
            None => Ok(self.cfg),
            Some(opt) => {
                self.next = self.rest.next();
                match opt.as_str() {
                    "-i" => self.build_input(),
                    "-o" => self.build_output(),
                    "-verify-after-each" => {
                        self.cfg.verify_after_each = true;
                        self.build_root()
                    }
                    "-print-after-each" => {
                        self.cfg.print_after_each = true;
                        self.build_root()
                    }
                    "-h" => {
                        print!(
                            "Usage: opt [passname...] -i input_file -o output_file\n\n{}",
                            self.pass_mgr.help_text()
                        );
                        print!("\n\nIn the absence of -i or -o options, input is taken from stdin and output is printed to stdout.\n");
                        exit(0);
                    }

                    name => {
                        if matches!(opt.chars().next(), Some('-')) {
                            Err(anyhow!("Unrecognised option '{opt}'."))
                        } else {
                            self.build_pass(name)
                        }
                    }
                }
            }
        }
    }

    fn build_input(mut self) -> Result<Config, anyhow::Error> {
        match self.next {
            None => Err(anyhow!("-i option requires an argument.")),
            Some(path) => {
                self.cfg.input_path = Some(path);
                self.next = self.rest.next();
                self.build_root()
            }
        }
    }

    fn build_output(mut self) -> Result<Config, anyhow::Error> {
        match self.next {
            None => Err(anyhow!("-o option requires an argument.")),
            Some(path) => {
                self.cfg.output_path = Some(path);
                self.next = self.rest.next();
                self.build_root()
            }
        }
    }

    fn build_pass(mut self, name: &str) -> Result<Config, anyhow::Error> {
        if let Some(pass) = self.pass_mgr.lookup_registered_pass(name) {
            self.cfg.passes.push(pass.name);
            self.build_root()
        } else {
            Err(anyhow!(
                "Unrecognised pass name '{name}'.\n\n{}",
                self.pass_mgr.help_text()
            ))
        }
    }
}

// -------------------------------------------------------------------------------------------------
