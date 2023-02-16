fn main() {
    let cmd = forc_tx::Command::try_parse().unwrap();
    let tx = fuel_tx::Transaction::try_from(cmd.tx).unwrap();
    match cmd.output_path {
        None => {
            let string = serde_json::to_string_pretty(&tx).unwrap();
            println!("{string}");
        }
        Some(path) => {
            let file = std::fs::File::create(path).unwrap();
            let writer = std::io::BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &tx).unwrap();
        }
    }
}
