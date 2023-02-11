use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Read, Write},
};

use anyhow::anyhow;
use sway_ir::{
    create_const_combine_pass, create_dce_pass, create_inline_pass, create_mem2reg_pass,
    create_simplify_cfg_pass, PassManager, PassManagerConfig,
};

// -------------------------------------------------------------------------------------------------

fn main() -> Result<(), anyhow::Error> {
    // Maintain a list of named pass functions for delegation.
    let mut pass_mgr = PassManager::default();

    pass_mgr.register(create_const_combine_pass());
    pass_mgr.register(create_inline_pass());
    pass_mgr.register(create_simplify_cfg_pass());
    pass_mgr.register(create_dce_pass());
    pass_mgr.register(create_mem2reg_pass());

    // Build the config from the command line.
    let config = ConfigBuilder::build(&pass_mgr, std::env::args())?;

    // Read the input file, or standard in.
    let input_str = read_from_input(&config.input_path)?;

    // Parse it. XXX Improve this error message too.
    let mut ir = sway_ir::parser::parse(&input_str)?;

    // Perform optimisation passes in order.
    let pm_config = PassManagerConfig {
        to_run: config.passes.iter().map(|pass| pass.name.clone()).collect(),
    };
    pass_mgr.run(&mut ir, &pm_config)?;

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
struct ConfigBuilder<'a, I: Iterator<Item = String>> {
    next: Option<String>,
    rest: I,
    cfg: Config,
    pass_mgr: &'a PassManager,
}

impl<'a, I: Iterator<Item = String>> ConfigBuilder<'a, I> {
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
        if self.pass_mgr.is_registered(name) {
            self.cfg.passes.push(name.into());
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
