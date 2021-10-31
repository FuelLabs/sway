use fuels_rs::json_abi::parse_param;
use fuels_rs::json_abi::ABIParser;
use fuels_rs::types::{ParamType, Property};
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
/// Sway/Fuel ABI coder.
enum Opt {
    /// Encode ABI call.
    Encode(Encode),
    /// Decode ABI call result.
    Decode(Decode),
}

#[derive(StructOpt, Debug)]
enum Encode {
    /// Load function from JSON ABI file.
    Function {
        abi_path: String,
        function_name: String,
        #[structopt(short, number_of_values = 1)]
        params: Vec<String>,
    },
    /// Specify types of input params inline.
    Params {
        /// Pairs of types directly followed by params in the form:
        ///
        /// -v <type1> <param1> -v <type2> <param2> ...
        #[structopt(
            short = "v",
            name = "type-or-param",
            number_of_values = 2,
            allow_hyphen_values = true
        )]
        params: Vec<String>,
    },
}

#[derive(StructOpt, Debug)]
enum Decode {
    /// Load function from JSON ABI file.
    Function {
        abi_path: String,
        function_name: String,
        data: String,
    },
    /// Specify types of input params inline.
    Params {
        #[structopt(short, name = "type", number_of_values = 1)]
        types: Vec<String>,
        data: String,
    },
}

fn execute<I>(args: I) -> anyhow::Result<String>
where
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString> + Clone,
{
    let opt = Opt::from_iter(args);

    match opt {
        Opt::Encode(Encode::Function {
            abi_path,
            function_name,
            params,
        }) => encode_input(&abi_path, &function_name, &params),
        Opt::Encode(Encode::Params { params }) => encode_params(&params),
        Opt::Decode(Decode::Params { types, data }) => decode_params(&types, &data),

        Opt::Decode(Decode::Function {
            abi_path,
            function_name,
            data,
        }) => decode_call_output(&abi_path, &function_name, &data),
    }
}

fn encode_params(params: &[String]) -> anyhow::Result<String> {
    let abi_coder = ABIParser::new();

    Ok(abi_coder.encode_params(params)?)
}

fn encode_input(path: &str, function_name: &str, values: &[String]) -> anyhow::Result<String> {
    if values.len() == 0 {
        anyhow::bail!("Values to be encoded shouldn't be empty")
    }

    let contract = fs::read_to_string(path)?;

    let mut abi_coder = ABIParser::new();

    let result = abi_coder.encode_with_function_selector(&contract, function_name, values)?;

    Ok(result)
}

fn decode_params(types: &[String], data: &str) -> anyhow::Result<String> {
    let abi_coder = ABIParser::new();

    let types: Result<Vec<ParamType>, _> = types
        .iter()
        .map(|s| {
            parse_param(&Property {
                name: "".into(),
                type_field: s.to_owned(),
                components: None,
            })
        })
        .collect();

    let data: Vec<u8> = hex::decode(&data)?;

    let decoded = abi_coder.decode_params(&types.unwrap(), &data)?;

    let mut result: String = String::new();
    for token in decoded {
        let format = format!("{}\n", token.to_string());
        result.push_str(&format);
    }

    Ok(result)
}

fn decode_call_output(path: &str, function_name: &str, data: &str) -> anyhow::Result<String> {
    let contract = fs::read_to_string(path)?;

    let abi_coder = ABIParser::new();

    let decoded = abi_coder.decode(&contract, function_name, &data.as_bytes())?;

    let mut result: String = String::new();
    for res in decoded {
        let format = format!("{}\n", res.to_string());
        result.push_str(&format);
    }

    Ok(result)
}

fn main() -> anyhow::Result<()> {
    println!("{}", execute(std::env::args())?);

    Ok(())
}
