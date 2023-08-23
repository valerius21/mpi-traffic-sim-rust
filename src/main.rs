mod graph;
mod models;
mod streets;
use crate::graph::graph::{GUtils, OSMGraph};
use crate::models::graph_input::GraphInput;
use clap::Parser;


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
    println!("{:?}\nMODEL\n", model);
    let osm_graph = OSMGraph::new(model.graph);
    println!("{:?}\nOSM\n", osm_graph);
    let my_graph = osm_graph.graph.clone();
    println!("{:?}\nMGRAPH\n", my_graph);
}
