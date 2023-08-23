mod graph;
mod models;
use crate::graph::graph::{GUtils, OSMGraph};
use crate::models::graph_input::GraphInput;
use clap::Parser;
use petgraph::dot::{Config, Dot};

/// Traffic Simulation with MPI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the JSON file for the Graph
    #[arg(short, long)]
    path: String,
}

fn main() {
    let args = Args::parse();

    let json = std::fs::read_to_string(args.path).unwrap();
    let model: GraphInput = serde_json::from_str(&json).unwrap();
    let osm_graph = OSMGraph::new(model.graph);
    let my_graph = osm_graph.graph.clone();

    println!("{:?}", Dot::with_config(&my_graph, &[Config::EdgeNoLabel]));
}
