use std::fs;

use clap::Parser;

use crate::{build_project, CmdExecutor, JsWorker, Req};

#[derive(Debug, Parser)]

pub struct RunOpts {}

impl CmdExecutor for RunOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let content = fs::read_to_string(&filename)?;
        let worker = JsWorker::try_new(&content)?;
        // TODO: normally this should run axum server
        let req = Req::builder()
            .method("GET".to_string())
            .url("http://localhost:3000".to_string())
            .headers(Default::default())
            .build();
        let ret = worker.run("hello", req)?;
        println!("Response: {:?}", ret);
        Ok(())
    }
}
