mod graph;
mod models;
mod streets;
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
}

fn main() {
    let args = Args::parse();
    let partitions = 2;

    let json = std::fs::read_to_string(args.path).unwrap();
    let model: GraphInput = serde_json::from_str(&json).unwrap();
    let osm_graph = OSMGraph::new(model.graph);
    let my_graph = osm_graph.graph.clone();

    println!(
        "Root Size ({},{})",
        my_graph.node_count(),
        my_graph.edge_count()
    );

    println!("Making {} partitions", partitions);

    let part_1 = osm_graph.partition(partitions, 0);
    let part_2 = osm_graph.partition(partitions, 1);

    println!(
        "Part 1 Size ({},{})",
        part_1.graph.node_count(),
        part_1.graph.edge_count()
    );

    println!(
        "Part 2 Size ({},{})",
        part_2.graph.node_count(),
        part_2.graph.edge_count()
    );
}
