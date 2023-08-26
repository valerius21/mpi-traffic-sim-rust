mod graph;
mod models;
mod streets;
mod utils;
use crate::graph::graph::{GPartition, GUtils, OSMGraph};
use crate::models::graph_input::GraphInput;
use clap::Parser;

/// Traffic Simulation with MPI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the JSON file for the Graph
    #[arg(short, long)]
    path: String,

    #[arg(short, long, default_value = 0)]
    partitions: usize,
}

fn main() {
    let args = Args::parse();
    let partitions = args.partitions; // =: WorldSize - 1
    let root_rank = 0;

    let json = std::fs::read_to_string(args.path).unwrap();
    let model: GraphInput = serde_json::from_str(&json).unwrap();
    let osm_graph = OSMGraph::new(root_rank, model.graph);
    let my_graph = osm_graph.graph.clone();

    println!(
        "Root Size ({},{})",
        my_graph.node_count(),
        my_graph.edge_count()
    );

    println!("Making {} partitions", partitions);

    for i in 0..partitions {
        let part = osm_graph.partition(partitions, i, i + 1);
        println!(
            "Part {} Size ({},{})",
            i,
            part.graph.node_count(),
            part.graph.edge_count()
        );
    }
}
