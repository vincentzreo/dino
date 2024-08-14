use std::fs;

use clap::Parser;

use crate::{build_project, CmdExecutor, JsWorker};

#[derive(Debug, Parser)]

pub struct RunOpts {}

impl CmdExecutor for RunOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let content = fs::read_to_string(&filename)?;
        let worker = JsWorker::try_new(&content)?;
        // TODO: normally this should run axum server
        worker.run("await handlers.hello()")?;
        Ok(())
    }
}
