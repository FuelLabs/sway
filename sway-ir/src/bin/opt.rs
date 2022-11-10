use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Read, Write},
};

use anyhow::anyhow;
use sway_ir::{error::IrError, function::Function, optimize, Context};

// -------------------------------------------------------------------------------------------------

fn main() -> Result<(), anyhow::Error> {
    // Maintain a list of named pass functions for delegation.
    let mut pass_mgr = PassManager::default();

    pass_mgr.register::<ConstCombinePass>();
    pass_mgr.register::<InlinePass>();
    pass_mgr.register::<SimplifyCfgPass>();
    pass_mgr.register::<DCEPass>();
    pass_mgr.register::<Mem2RegPass>();

    // Build the config from the command line.
    let config = ConfigBuilder::build(&pass_mgr, std::env::args())?;

    // Read the input file, or standard in.
    let input_str = read_from_input(&config.input_path)?;

    // Parse it. XXX Improve this error message too.
    let mut ir = sway_ir::parser::parse(&input_str)?;

    // Perform optimisation passes in order.
    for pass in config.passes {
        pass_mgr.run(pass.name.as_ref(), &mut ir)?;
    }

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

trait NamedPass {
    fn name() -> &'static str;
    fn descr() -> &'static str;
    fn run(ir: &mut Context) -> Result<bool, IrError>;

    fn run_on_all_fns<F: FnMut(&mut Context, &Function) -> Result<bool, IrError>>(
        ir: &mut Context,
        mut run_on_fn: F,
    ) -> Result<bool, IrError> {
        let funcs = ir
            .module_iter()
            .flat_map(|module| module.function_iter(ir))
            .collect::<Vec<_>>();
        let mut modified = false;
        for func in funcs {
            if run_on_fn(ir, &func)? {
                modified = true;
            }
        }
        Ok(modified)
    }
}

type NamePassPair = (&'static str, fn(&mut Context) -> Result<bool, IrError>);

#[derive(Default)]
struct PassManager {
    passes: HashMap<&'static str, NamePassPair>,
}

impl PassManager {
    fn register<T: NamedPass>(&mut self) {
        self.passes.insert(T::name(), (T::descr(), T::run));
    }

    fn run(&self, name: &str, ir: &mut Context) -> Result<bool, IrError> {
        self.passes.get(name).expect("Unknown pass name!").1(ir)
    }

    fn contains(&self, name: &str) -> bool {
        self.passes.contains_key(name)
    }

    fn help_text(&self) -> String {
        let summary = self
            .passes
            .iter()
            .map(|(name, (descr, _))| format!("  {name:16} - {descr}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!("Valid pass names are:\n\n{summary}",)
    }
}

// -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -

struct InlinePass;

impl NamedPass for InlinePass {
    fn name() -> &'static str {
        "inline"
    }

    fn descr() -> &'static str {
        "inline function calls."
    }

    fn run(ir: &mut Context) -> Result<bool, IrError> {
        // For now we inline everything into `main()`.  Eventually we can be more selective.
        let main_fn = ir
            .module_iter()
            .flat_map(|module| module.function_iter(ir))
            .find(|f| f.get_name(ir) == "main")
            .unwrap();
        optimize::inline_all_function_calls(ir, &main_fn)
    }
}

// -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -

struct ConstCombinePass;

impl NamedPass for ConstCombinePass {
    fn name() -> &'static str {
        "constcombine"
    }

    fn descr() -> &'static str {
        "constant folding."
    }

    fn run(ir: &mut Context) -> Result<bool, IrError> {
        Self::run_on_all_fns(ir, optimize::combine_constants)
    }
}

// -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -

struct SimplifyCfgPass;

impl NamedPass for SimplifyCfgPass {
    fn name() -> &'static str {
        "simplifycfg"
    }

    fn descr() -> &'static str {
        "merge or remove redundant blocks."
    }

    fn run(ir: &mut Context) -> Result<bool, IrError> {
        Self::run_on_all_fns(ir, optimize::simplify_cfg)
    }
}

// -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -

struct DCEPass;

impl NamedPass for DCEPass {
    fn name() -> &'static str {
        "dce"
    }

    fn descr() -> &'static str {
        "Dead code elimination."
    }

    fn run(ir: &mut Context) -> Result<bool, IrError> {
        Self::run_on_all_fns(ir, optimize::dce)
    }
}

// -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -  -

struct Mem2RegPass;

impl NamedPass for Mem2RegPass {
    fn name() -> &'static str {
        "mem2reg"
    }

    fn descr() -> &'static str {
        "Promote local memory to SSA registers."
    }

    fn run(ir: &mut Context) -> Result<bool, IrError> {
        Self::run_on_all_fns(ir, optimize::promote_to_registers)
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
        if self.pass_mgr.contains(name) {
            self.cfg.passes.push(name.into());
            self.next = self.rest.next();
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
