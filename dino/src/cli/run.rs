use std::{fs, path::Path};

use clap::Parser;
use dino_server::{start_server, ProjectConfig, SwappableAppRouter, TenentRouter};
use notify::{RecursiveMode, Watcher};
use tracing::{level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

use crate::{build_project, CmdExecutor};

#[derive(Debug, Parser)]

pub struct RunOpts {
    // port to listen
    #[arg(short, long, default_value = "3000")]
    pub port: u16,
}

impl CmdExecutor for RunOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let layer = Layer::new().with_filter(LevelFilter::INFO);
        tracing_subscriber::registry().with(layer).init();
        let filename = build_project(".")?;
        let code = fs::read_to_string(&filename)?;
        let config = ProjectConfig::load(filename.replace(".mjs", ".yml"))?;
        let router = SwappableAppRouter::try_new(&code, config.routes)?;
        let routers = vec![TenentRouter::new("localhost", router.clone())];
        // TODO: can't work
        watch_project(".", router)?;
        start_server(self.port, routers).await?;
        Ok(())
    }
}

fn watch_project(dir: &'static str, router: SwappableAppRouter) -> anyhow::Result<()> {
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            warn!("event: {:?}", event);
            let filename = build_project(dir).unwrap();
            let config = ProjectConfig::load(filename.replace(".mjs", ".yml")).unwrap();
            let code = fs::read_to_string(&filename).unwrap();
            router.swap(&code, config.routes).unwrap();
        }
        Err(e) => warn!("watch error: {:?}", e),
    })?;
    watcher.watch(Path::new(dir), RecursiveMode::Recursive)?;
    Ok(())
}
