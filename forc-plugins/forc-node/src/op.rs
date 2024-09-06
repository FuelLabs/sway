use crate::cmd::ForcNode;

pub(crate) async fn run(cmd: ForcNode) -> anyhow::Result<()> {
    match cmd {
        ForcNode::Local(local) => crate::local::op::run(local).await?,
        ForcNode::Testnet(testnet) => crate::testnet::op::run(testnet).await?,
    }
    Ok(())
}
