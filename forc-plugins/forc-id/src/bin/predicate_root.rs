use clap::Parser;
use forc_tracing::init_tracing_subscriber;

fn main() {
    init_tracing_subscriber(Default::default());
    let command = forc_id::cmd::PredicateRoot::parse();
    if let Err(err) = forc_id::op::predicate_root(command) {
        tracing::error!("Error: {:?}", err);
        std::process::exit(1);
    }
}
