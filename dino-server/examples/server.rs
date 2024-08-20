use anyhow::Result;
use dashmap::DashMap;
use dino_server::{start_server, ProjectConfig, SwappableAppRouter};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = include_str!("../fixtures/config.yml");
    let config: ProjectConfig = serde_yaml::from_str(config)?;

    let router = DashMap::new();
    router.insert(
        "localhost".to_string(),
        SwappableAppRouter::try_new(config.routes)?,
    );
    start_server(8080, router).await?;
    Ok(())
}
