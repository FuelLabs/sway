use anyhow::Result;

pub(crate) fn exec() -> Result<()> {
    for path in crate::cli::plugin::find_all() {
        println!("{}", path.display());
    }
    Ok(())
}
