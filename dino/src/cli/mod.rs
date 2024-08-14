mod build;
mod init;
mod run;

pub use build::BuildOpts;
use clap::Parser;
use enum_dispatch::enum_dispatch;
pub use init::InitOpts;
pub use run::RunOpts;

// rcli csv -i input.csv -o output.csv --header -d ','
#[derive(Debug, Parser)]
#[command(name = "dino", version, author, about, long_about=None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}
#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum SubCommand {
    #[command(name = "init", about = "Initialize a new dino project")]
    Init(InitOpts),
    #[command(name = "build", about = "Build dino project")]
    Build(BuildOpts),
    #[command(name = "run", about = "Run user's dino project")]
    Run(RunOpts),
}
