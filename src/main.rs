// #![allow(unused)] TODO: remove leter
extern crate mpi;
extern crate rand;

mod error;
mod graph;
mod mmpi;
mod models;
mod prelude;
mod streets;
mod utils;
use std::collections::HashMap;

use crate::graph::graph::{GPartition, GUtils, OSMGraph};
use crate::mmpi::*;
use crate::models::graph_input::GraphInput;
use crate::models::vehicle::Vehicle;
use crate::prelude::*;
use crate::utils::MpiMessageContent;
use clap::Parser;
use log::Level;
use mpi::traits::*;

/// Traffic Simulation with MPI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the JSON file for the Graph
    #[arg(short, long)]
    path: String,

    /// Number of vehicles to simulate
    #[arg(short, long, default_value = "10")]
    number_of_vehicles: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(Level::Debug).unwrap();
    let args = Args::parse();
    let number_of_vehicles = args.number_of_vehicles;

    // mpi setup
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    if size < 2 {
        panic!("Size of MPI_COMM_WORLD must be 2, but is {}!", size);
    }

    // read input data for gprah
    let json = std::fs::read_to_string(args.path)?;
    let model: GraphInput = serde_json::from_str(&json).unwrap(); // FIXME: result handling
    let partitions: usize = (size - 1).try_into().unwrap();

    // bootstrap the root graph
    let osm_graph = OSMGraph::new(model.graph)?;

    let my_graph = osm_graph.graph.clone();

    log::info!(
        "[{}] Root Size ({},{})",
        rank,
        my_graph.node_count(),
        my_graph.edge_count()
    );

    log::info!("[{}] Making {} partition(s)", rank, partitions);
    match rank {
        0 => {
            log::debug!("[{}] Creating NodeID->Rank mapping", rank);
            // create map with nodeID->rank mapping
            let mut node_to_rank = HashMap::new();

            for r in 1..size {
                let rr: usize = r.try_into().unwrap();
                let part = osm_graph.partition(partitions, rr - 1)?;

                for node in part.graph.nodes() {
                    node_to_rank.insert(node, r);
                }
            }

            log::debug!("[{}] Sending vehicles", rank);
            // FIXME: This approach avoids invalid vertices in the path, e.g. on graph islands
            let mut vehicle_counter = 0;
            // send vehicles
            while vehicle_counter < number_of_vehicles {
                let v = Vehicle::generate_default(&my_graph).unwrap();
                let node = v.prev_id;
                let r = match node_to_rank.get(&node) {
                    Some(r) => r.clone(),
                    None => {
                        log::warn!("[{}] No rank found for node {}", rank, node);
                        continue;
                    }
                };

                let vb = Vehicle::to_bytes(v).unwrap();

                world
                    .process_at_rank(r)
                    .send_with_tag(&vb[..], ROOT_LEAF_VEHICLE);
                log::debug!("[{}] Sent vehicle to rank {}", rank, r);
                vehicle_counter += 1;
            }
            log::debug!("[{}] Sent {} vehicles to ranks", rank, vehicle_counter,);
            log::debug!("[{}] Listening for incoming connections", rank);
            loop {
                let (_msg, status) = world.any_process().receive_vec::<u8>();
                match status.tag() {
                    LEAF_ROOT_VEHICLE => {
                        log::debug!(
                            "[{}] Received vehicle from rank {}",
                            rank,
                            status.source_rank()
                        );
                        todo!();
                    }
                    EDGE_LENGTH_REQUEST => {
                        let el_req = EdgeLengthRequest::from_bytes(_msg).unwrap();
                        let el = my_graph.edges(el_req.from).find(|e| e.1 == el_req.to);
                        match el {
                            Some(el) => {
                                let v = vec![el.2.clone()];
                                world
                                    .process_at_rank(status.source_rank())
                                    .send_with_tag(&v[..], EDGE_LENGTH_RESPONSE);
                            }
                            None => {
                                log::error!("[{}] Received unknown message with unknown tag", rank);
                            }
                        }
                    }
                    _ => {
                        log::error!("[{}] Received unknown message with unknown tag", rank);
                    }
                }
            }
        }
        rank_number => {
            log::debug!("[{}] Assigning leaf to rank", rank);
            let r: usize = rank_number.try_into().unwrap();
            let part = osm_graph.partition(partitions, r - 1)?;

            log::info!(
                "[{}] Rank {} -> Size ({},{})",
                rank,
                r,
                part.graph.node_count(),
                part.graph.edge_count()
            );

            loop {
                let (msg, status) = world.any_process().receive_vec::<u8>();
                match status.tag() {
                    ROOT_LEAF_VEHICLE => {
                        let mut v = Vehicle::from_bytes(msg).unwrap();
                        log::debug!(
                            "[{}] Received vehicle from rank {} ID {}",
                            rank,
                            status.source_rank(),
                            v.id
                        );

                        // II.3
                        v.marked_for_deletion = false;

                        // ask root for edge length
                        let el_req = EdgeLengthRequest {
                            from: v.prev_id,
                            to: v.next_id,
                        };
                        let buf = EdgeLengthRequest::to_bytes(el_req).unwrap();
                        world
                            .process_at_rank(0)
                            .send_with_tag(&buf[..], EDGE_LENGTH_REQUEST);

                        // get edge length
                        let (el_msg, _) = world
                            .process_at_rank(0)
                            .receive_vec_with_tag::<f64>(EDGE_LENGTH_RESPONSE);

                        // II.5
                        v.delta += el_msg[0];
                        // drive vehicle
                        todo!();
                    }
                    _ => {
                        log::error!("[{}] Received unknown message with unknown tag", rank);
                    }
                }
            }
        }
    };

    // // * NOTE: tokio has it's own scheduler
    // let mut handles = vec![];

    // for _ in 0..100 {
    //     let osm_graph_clone = osm_graph.clone();
    //     let my_graph_clone = my_graph.clone();

    //     // FIXME: error handling
    //     let handle = tokio::spawn(async move {
    //         let mut v = Vehicle::generate_default(&my_graph_clone).unwrap();
    //         v.drive(&osm_graph_clone); // Match on enum mark4del / is_parked?
    //                                    // log::info!("[{}] finished driving", v.id);
    //     });

    //     handles.push(handle);
    // }

    // // Await all tasks to complete
    // for handle in handles {
    //     handle.await.unwrap();
    // }

    Ok(())
}
