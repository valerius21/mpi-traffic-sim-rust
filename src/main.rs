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
use core::panic;
use std::collections::HashMap;

use crate::graph::get_path_length;
use crate::graph::graph::{GPartition, GUtils, OSMGraph};
use crate::mmpi::*;
use crate::models::graph_input::GraphInput;
use crate::models::vehicle::{Moveable, Vehicle};
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

    /// Set Debug Logging
    #[arg(short, long, default_value = "false")]
    debug: bool,

    /// Error rate for the simulation
    #[arg(short, long, default_value = "0.0")]
    error_rate: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let number_of_vehicles = args.number_of_vehicles;
    let error_rate = args.error_rate;
    if args.debug {
        simple_logger::init_with_level(Level::Debug).unwrap();
    } else {
        simple_logger::init_with_level(Level::Info).unwrap();
    }

    // Avoiding overflows
    const MAX_NUMBER_OF_VEHICLES: usize = usize::MAX / 2;
    if number_of_vehicles > MAX_NUMBER_OF_VEHICLES {
        panic!(
            "Number of vehicles must be smaller than {}, but is {}!",
            MAX_NUMBER_OF_VEHICLES, number_of_vehicles
        );
    }

    // finishing threshold
    let finishing_threshold = ((number_of_vehicles as f64) * (1.0 - error_rate)) as usize;

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
            let mut finished_vehicle_counter = 0;
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

            if node_to_rank.len() != my_graph.nodes().len() {
                panic!(
                    "Node to rank mapping is incomplete! {} != {}",
                    node_to_rank.len(),
                    my_graph.nodes().len()
                );
            }

            log::debug!("[{}] Sending vehicles", rank);
            // FIXME: This approach avoids invalid vertices in the path, e.g. on graph islands
            let mut vehicle_counter = 0;
            // send vehicles
            while vehicle_counter < number_of_vehicles {
                let v = Vehicle::generate_default(&my_graph).unwrap();

                match map_vehicle_to_rank(v, &node_to_rank, rank, world) {
                    Ok(_) => {}
                    Err(_) => {
                        log::warn!("[{}] Failed to send vehicle", rank);
                        continue;
                    }
                };
                vehicle_counter += 1;
            }
            log::debug!("[{}] Sent {} vehicles to ranks", rank, vehicle_counter,);
            log::debug!("[{}] Listening for incoming connections", rank);
            loop {
                let (msg, status) = world.any_process().receive_vec::<u8>();
                match status.tag() {
                    LEAF_ROOT_VEHICLE => {
                        log::debug!(
                            "[{}] Received vehicle from rank {}",
                            rank,
                            status.source_rank()
                        );
                        let v = Vehicle::from_bytes(msg).unwrap();
                        match map_vehicle_to_rank(v, &node_to_rank, rank, world) {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "[{}] Failed to send vehicle after receive: {:?}",
                                    rank,
                                    err
                                );
                                continue;
                            }
                        }
                    }
                    EDGE_LENGTH_REQUEST => {
                        log::debug!("Received edge length request");
                        let el_req = EdgeLengthRequest::from_bytes(msg).unwrap();
                        let from = el_req.from;
                        let to = el_req.to;

                        let edges = my_graph.edges_directed(from, petgraph::Direction::Outgoing);

                        let el: f64 = match edges.filter(|e| e.1 == to).next() {
                            Some(e) => e.2.clone(),
                            None => {
                                log::error!("[{}] No edge found for from={} to={}", rank, from, to);
                                // NOTE: handle possible currupt algorithmic/path finding error
                                // recalculating the way, and send the distance of the path instead of 0
                                let cost = get_path_length(from, to, my_graph.clone());
                                cost
                            }
                        };
                        let v = vec![el.clone()];
                        world
                            .process_at_rank(status.source_rank())
                            .send_with_tag(&v[..], EDGE_LENGTH_RESPONSE);
                    }
                    LEAF_ROOT_VEHICLE_FINISH => {
                        finished_vehicle_counter += 1;
                        if finished_vehicle_counter >= finishing_threshold {
                            log::info!(
                                "[{}] Finished {} vehicles, terminating",
                                rank,
                                finished_vehicle_counter
                            );
                            for r in 1..size {
                                world
                                    .process_at_rank(r)
                                    .send_with_tag(&[1], ROOT_LEAF_TERMINATE);
                            }
                            break;
                        }
                    }
                    _ => {
                        log::error!(
                            "[{}] Received unknown message with unknown tag -> {} -> {:#?}",
                            rank,
                            status.tag(),
                            msg
                        );
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

                        if v.is_parked || v.prev_id == v.next_id {
                            // vehicle is done
                            log::info!("[{}] - 1 Vehicle {} is done driving", rank, v.id);
                            continue;
                        }

                        // II.3
                        v.marked_for_deletion = false;

                        // ask root for edge length
                        let el_req = EdgeLengthRequest {
                            from: v.prev_id,
                            to: v.next_id,
                        };
                        let buf = EdgeLengthRequest::to_bytes(el_req.clone()).unwrap();
                        log::debug!(
                            "[{}] Sending edge length request, {:?} @ {:?}",
                            rank,
                            el_req,
                            v
                        );
                        world
                            .process_at_rank(0)
                            .send_with_tag(&buf[..], EDGE_LENGTH_REQUEST);

                        // get edge length
                        let (el_msg, _) = world
                            .process_at_rank(0)
                            .receive_vec_with_tag::<f64>(EDGE_LENGTH_RESPONSE);

                        // II.5
                        v.delta += el_msg[0];

                        log::debug!(
                            "[{}] Vehicle {} is driving from {} to {}",
                            rank,
                            v.id,
                            v.prev_id,
                            v.next_id
                        );

                        loop {
                            if v.is_parked {
                                // v is done
                                log::info!("[{}] - 2 Vehicle {} is done driving", rank, v.id);
                                // create buffer containing the number 1
                                let buf = vec![1];
                                world
                                    .process_at_rank(0)
                                    .send_with_tag(&buf[..], LEAF_ROOT_VEHICLE_FINISH);
                                break;
                            } else if v.marked_for_deletion {
                                log::debug!("[{}] Sending vehicle {} to root", rank, v.id);
                                // send vehicle to root
                                world.process_at_rank(0).send_with_tag(
                                    &Vehicle::to_bytes(v).unwrap()[..],
                                    LEAF_ROOT_VEHICLE,
                                );
                                break;
                            }
                            v.step(&part);
                        }
                    }
                    ROOT_LEAF_TERMINATE => {
                        log::info!("[{}] Received termination notification", rank);
                        break;
                    }
                    _ => {
                        log::error!("[{}] Received unknown message with unknown tag", rank);
                    }
                }
            }
        }
    };
    Ok(())
}
