// #![allow(unused)] TODO: remove leter
extern crate rand;
use rand::Rng;

mod error;
mod graph;
mod models;
mod prelude;
mod streets;
mod utils;
use crate::graph::graph::{GPartition, GUtils, GraphID, OSMGraph};
use crate::models::graph_input::GraphInput;
use crate::models::vehicle::{Moveable, Vehicle};
use crate::prelude::*;
use crate::streets::vehicle_builder::VehicleBuilder;
use crate::utils::get_random_vector_element;
use clap::Parser;
use petgraph::algo::astar;
use petgraph::dot::Dot;

/// Traffic Simulation with MPI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the JSON file for the Graph
    #[arg(short, long)]
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let partitions = 2; // =: WorldSize - 1

    // read input data for gprah
    let json = std::fs::read_to_string(args.path)?;
    let model: GraphInput = serde_json::from_str(&json).unwrap(); // FIXME: result handling

    // bootstrap the root graph
    let root_rank: GraphID = 0;
    let osm_graph = OSMGraph::new(model.graph)?;

    let my_graph = osm_graph.graph.clone();

    println!(
        "Root Size ({},{})",
        my_graph.node_count(),
        my_graph.edge_count()
    );

    println!("Making {} partitions", partitions);

    for i in 0..partitions {
        let part = osm_graph.partition(partitions, i)?;
        println!(
            "Part {} Size ({},{})",
            i,
            part.graph.node_count(),
            part.graph.edge_count()
        );
    }

    // println!("{:?}", Dot::with_config(&my_graph, &[]));

    for _ in 0..=100 {
        let mut v = Vehicle::generate_default(&my_graph)?;
        v.drive(&osm_graph);
        println!("{:#?} finished driving", v.id);
    }
    Ok(())
}
