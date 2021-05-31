use fuel_tx::Transaction;
use fuel_core::interpreter::Interpreter;
use structopt::{self, StructOpt};

use crate::ops::forc_build;

#[derive(Debug, StructOpt)]
pub(crate) struct Command {
    #[structopt(short = "d", long = "data")]
    pub data: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    let input_data = &command.data.unwrap_or("".into());
    let data = format_hex_data(input_data);
    let script_data = hex::decode(data).expect("Invalid hex");

    let project_path = "../example_project/fuel_project".into();

    match forc_build::build(Some(project_path)) {
        Ok(script) => {
            let tx = create_tx_with_script_and_data(script, script_data);
            let vm = Interpreter::execute_tx(tx).expect("Invalid tx");
            println!("{:?}", vm.log());
        }
        Err(e) => println!("{}", e),
    }
    Ok(())
}

fn create_tx_with_script_and_data(script: Vec<u8>, script_data: Vec<u8>) -> Transaction {
    let gas_price = 0;
    let gas_limit = 10000000;
    let maturity = 0;
    let inputs = vec![];
    let outputs = vec![];
    let witnesses = vec![];

    Transaction::script(
        gas_price,
        gas_limit,
        maturity,
        script,
        script_data,
        inputs,
        outputs,
        witnesses,
    )
}

fn format_hex_data(data: &str) -> &str {
    if &data[..2] == "0x" {
        &data[2..]
    } else {
        &data
    }
}
