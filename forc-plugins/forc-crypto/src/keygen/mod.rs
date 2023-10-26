use atty::Stream;
use clap::ValueEnum;
use std::io::{stdin, stdout, Read, Write};
use termion::screen::IntoAlternateScreen;

pub mod new_key;
pub mod parse_secret;

pub(crate) const BLOCK_PRODUCTION: &str = "block-production";
pub(crate) const P2P: &str = "p2p";

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum KeyType {
    #[default]
    BlockProduction,
    Peering,
}

fn wait_for_keypress() {
    let mut single_key = [0u8];
    stdin().read_exact(&mut single_key).unwrap();
}

pub(crate) fn display_string_discreetly(
    discreet_string: &str,
    continue_message: &str,
) -> anyhow::Result<()> {
    if atty::is(Stream::Stdout) {
        let mut screen = stdout().into_alternate_screen()?;
        writeln!(screen, "{discreet_string}")?;
        screen.flush()?;
        println!("{continue_message}");
        wait_for_keypress();
    } else {
        print!("{discreet_string}");
    }
    Ok(())
}
