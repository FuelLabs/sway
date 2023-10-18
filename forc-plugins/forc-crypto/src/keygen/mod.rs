use clap::ValueEnum;
use std::io::{Read, Write};

pub mod new_key;
pub mod parse_secret;

pub const BLOCK_PRODUCTION: &str = "block-production";
pub const P2P: &str = "p2p";

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum KeyType {
    #[default]
    BlockProduction,
    Peering,
}

fn wait_for_keypress() {
    let mut single_key = [0u8];
    std::io::stdin().read_exact(&mut single_key).unwrap();
}

pub(crate) fn display_string_discreetly(
    discreet_string: &str,
    continue_message: &str,
) -> anyhow::Result<()> {
    use termion::screen::IntoAlternateScreen;
    let mut screen = std::io::stdout().into_alternate_screen()?;
    writeln!(screen, "{discreet_string}")?;
    screen.flush()?;
    println!("{continue_message}");
    wait_for_keypress();
    Ok(())
}
