use super::cmd::TestnetCmd;
use crate::{
    cmd::{ask_user_discreetly, ask_user_string, ask_user_yes_no_question},
    pkg::{create_chainconfig_dir, ChainConfig},
    run::{run_mode, Mode},
};

pub(crate) async fn run(cmd: TestnetCmd) -> anyhow::Result<()> {
    create_chainconfig_dir(ChainConfig::Testnet)?;
    let (peer_id, secret) = if let (Some(peer_id), Some(secret)) = (&cmd.peer_id, &cmd.secret) {
        (peer_id.clone(), secret.clone())
    } else {
        let has_keypair = ask_user_yes_no_question("Do you have a keypair in hand?")?;
        if has_keypair {
            // ask the keypair
            let peer_id = cmd.peer_id.unwrap_or_else(|| ask_user_string("Peer Id:"));
            let secret = ask_user_discreetly("Secret:")?;
            println!("{peer_id} : {secret}");
            (peer_id, secret)
        } else {
            // create the keypair
            todo!()
        }
    };
    let opts = TestnetOpts { peer_id, secret };
    let mode = Mode::Testnet(opts);
    // Ask if the user already have a key-pair generated.
    run_mode(mode).await?;
    Ok(())
}

#[derive(Debug)]
pub struct TestnetOpts {
    peer_id: String,
    secret: String,
}
