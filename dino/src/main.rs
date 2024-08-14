use clap::Parser;
use dino::{CmdExecutor, Opts};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    #[allow(clippy::let_unit_value)]
    let _ = opts.cmd.execute().await?;
    Ok(())
}
