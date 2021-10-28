use crate::cli::AbiSpecCommand;

pub fn generate_abi_spec(command: AbiSpecCommand) -> Result<Vec<u8>, String> {
    let AbiSpecCommand {
        path,
        offline_mode,
        silent_mode,
        json_outfile,
    } = command;

    unimplemented!()
}
