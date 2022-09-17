use std::{
    convert::TryInto,
    io::{stdout, Write},
};

use anyhow::Result;
use cli::pick_option;
use engine::{ResourceGetter, StacksGetter};

use crate::engine::Resource;

mod cli;
mod engine;

#[tokio::main]
async fn main() -> Result<()> {
    let engine = engine::Engine::new().await;

    let stacks = engine.get_stacks().await?;
    let picked_stack = pick_option(stacks, "Pick a stack")?;

    let resources_for_stack = engine.get_resources(&picked_stack).await?;
    let picked_resource_logical_id = pick_option(resources_for_stack, "Pick a resource")?;

    let picked_resource: Resource = picked_resource_logical_id.try_into()?;
    let url_path = picked_resource.to_console_url_path(&picked_stack)?;

    stdout().write_all(url_path.as_bytes())?;

    return Ok(());
}
