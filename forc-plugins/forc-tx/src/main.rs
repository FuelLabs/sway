fn main() -> anyhow::Result<()> {
    let cmd = forc_tx::Command::parse();
    let tx = fuel_tx::Transaction::try_from(cmd.tx)?;
    match cmd.output_path {
        None => {
            let string = serde_json::to_string_pretty(&tx)?;
            println!("{string}");
        }
        Some(path) => {
            let file = std::fs::File::create(path)?;
            let writer = std::io::BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &tx)?;
        }
    }
    Ok(())
}
