mod aws;
mod business;
mod config;

use config::Config;

use color_eyre::Result;
use std::env;
use tracing_subscriber::EnvFilter;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ecr::config::Region;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("aws_image_tag=info".parse()?))
        .init();
    let config = Config::new()?;
    let tags = business::get_tags(&config.environment, &config.tag).await;

    let region =
        RegionProviderChain::first_try(env::var("AWS_DEFAULT_REGION").ok().map(Region::new))
            .or_default_provider()
            .or_else(Region::new("eu-north-1"));
    let shared_config = aws_config::from_env().region(region).load().await;
    let client = aws_sdk_ecr::Client::new(&shared_config);

    let manifest = aws::get_manifest(&client, &config.application, &config.tag).await?;

    aws::tag_image(&client, &config.application, &manifest, tags).await?;
    Ok(())
}
