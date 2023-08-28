#[allow(clippy::all)]
extern crate mpi;
extern crate rand;

use cli::Cli;

use crate::prelude::*;
use clap::Parser;

mod cli;
mod error;
mod graph;
mod models;
mod prelude;
mod utils;
mod vmpi;
mod world;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    world::run(cli).await.unwrap();
    Ok(())
}
