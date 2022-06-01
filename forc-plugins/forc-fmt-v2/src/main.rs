//! A `forc` plugin for running the Sway code formatter.

use forc_util::init_tracing_subscriber;
use sway_fmt_v2::App;
use tracing::error;

fn main() {
    init_tracing_subscriber();
    if let Err(err) = App::run() {
        error!("Error: {:?}", err);
        std::process::exit(1);
    }
}
