extern crate mpi;

use mpi::traits::*;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use mpi::{point_to_point::Status, topology::SystemCommunicator, Rank};
use petgraph::{graphmap::GraphMap, Directed};
use tokio::task::JoinHandle;

use crate::{
    cli::{self, Cli},
    graph::osm_graph::OSMGraph,
    models::graph_input::GraphInput,
    utils::MAX_NUMBER_OF_VEHICLES,
};
use crate::{
    cli::{Parallelism, ThreadRuntime},
    graph::{
        get_path_length,
        osm_graph::{GPartition, Osmid},
    },
    models::vehicle::{Moveable, Vehicle},
    prelude::*,
    utils::MpiMessageContent,
    vmpi::*,
};

// Set up logging
fn setup_logging(level: cli::LoggingLevel) {
    let level = match level {
        cli::LoggingLevel::Debug => log::Level::Debug,
        cli::LoggingLevel::Info => log::Level::Info,
        cli::LoggingLevel::Warn => log::Level::Warn,
        cli::LoggingLevel::Error => log::Level::Error,
    };

    simple_logger::init_with_level(level).unwrap();
}

// Converts the fiel from the given path into a GraphInput struct
pub fn get_gi_from_input_file(input_file: &PathBuf) -> Result<GraphInput> {
    // open input file
    let input_file = std::fs::File::open(input_file)?;
    // read input data for gprah
    let model: GraphInput = serde_json::from_reader(input_file).unwrap();
    Ok(model)
}

// Parses the input file into a OSMGraph
pub fn parse_input(input_file: &PathBuf) -> Result<OSMGraph> {
    let model = get_gi_from_input_file(input_file)?;
    // bootstrap the root graph
    OSMGraph::new(model.graph)
}

/// Entry point for the simulation
pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        cli::Commands::GraphParts {
            input_file,
            parallelism,
            num_vehicles,
            logging_level,
            thread_runtime,
            mpi,
            error_rate,
            min_speed,
            max_speed,
        } => {
            setup_logging(logging_level);

            // Avoiding overflows
            if num_vehicles > MAX_NUMBER_OF_VEHICLES {
                panic!(
                    "Number of vehicles must be smaller than {}, but is {}!",
                    MAX_NUMBER_OF_VEHICLES, num_vehicles
                );
            }
            // finishing threshold
            let finishing_threshold = ((num_vehicles as f64) * (1.0 - error_rate)) as usize;

            if mpi && parallelism == Parallelism::SingleThreaded {
                panic!("MPI and SingleThreaded are not compatible!");
            }

            if mpi {
                log::debug!("Running with MPI");
                let (universe, _) =
                    mpi::initialize_with_threading(mpi::Threading::Multiple).unwrap();
                let world = universe.world();
                let size = world.size();
                let rank = world.rank();

                if size < 2 {
                    panic!("Size of MPI_COMM_WORLD must be 2, but is {}!", size);
                }
                let partitions: usize = (size - 1).try_into().unwrap();
                let osm_graph = parse_input(&input_file).unwrap();
                let my_graph = osm_graph.graph.clone();

                log::debug!(
                    "[{}] Root Size ({},{})",
                    rank,
                    my_graph.node_count(),
                    my_graph.edge_count()
                );

                log::debug!("[{}] Making {} partition(s)", rank, partitions);
                let start = std::time::Instant::now();
                match rank {
                    ROOT_RANK => {
                        root_event_loop(
                            num_vehicles,
                            finishing_threshold,
                            world,
                            size,
                            rank,
                            partitions,
                            &osm_graph,
                            &my_graph,
                            min_speed,
                            max_speed,
                        )?;
                    }
                    rank_number => {
                        log::debug!("[{}] Assigning leaf to rank", rank);
                        let r: usize = rank_number.try_into().unwrap();
                        let p = osm_graph.partition(partitions, r - 1)?;

                        log::debug!(
                            "[{}] Rank {} -> Size ({},{})",
                            rank,
                            r,
                            p.graph.node_count(),
                            p.graph.edge_count()
                        );

                        let mm = Arc::new(Mutex::new(p));
                        loop {
                            let (msg, status) =
                                world.process_at_rank(ROOT_RANK).receive_vec::<u8>();
                            log::debug!(
                                "[{}] Received message from rank {}",
                                rank,
                                status.source_rank()
                            );
                            if process_leaf_event(
                                parallelism,
                                thread_runtime,
                                world,
                                rank,
                                &mm,
                                msg,
                                status,
                            ) {
                                break;
                            }
                        }
                    }
                };
                let end = std::time::Instant::now();
                let time = end - start;
                log::info!("[{}] Finished in {:?} microseconds", rank, time.as_micros());
            } else {
                log::debug!("Running without MPI");
                let osm_graph = parse_input(&input_file).unwrap();
                let my_graph = osm_graph.graph.clone();

                log::debug!(
                    "Root Size ({},{})",
                    my_graph.node_count(),
                    my_graph.edge_count()
                );

                let start = std::time::Instant::now();
                let mut step_accumulator = 0;

                match parallelism {
                    Parallelism::SingleThreaded => {
                        for _ in 0..num_vehicles {
                            let mut v =
                                Vehicle::generate_default(&my_graph, min_speed, max_speed).unwrap();

                            v.drive(&osm_graph);
                            step_accumulator += v.steps;
                        }
                    }
                    Parallelism::MultiThreaded => match thread_runtime {
                        cli::ThreadRuntime::RustThreads => {
                            let mut handles = vec![];
                            for _ in 0..num_vehicles {
                                let mut v =
                                    Vehicle::generate_default(&my_graph, min_speed, max_speed)
                                        .unwrap();
                                let osm_graph = Arc::new(osm_graph.clone());
                                let handle = thread::spawn(move || {
                                    v.drive(&osm_graph);
                                    v
                                });
                                handles.push(handle);
                            }
                            for handle in handles {
                                let v = handle.join().unwrap();
                                step_accumulator += v.steps;
                            }
                        }
                        cli::ThreadRuntime::Tokio => {
                            let mut handles = vec![];
                            for _ in 0..num_vehicles {
                                let mut v =
                                    Vehicle::generate_default(&my_graph, min_speed, max_speed)
                                        .unwrap();
                                let osm_graph = Arc::new(osm_graph.clone());
                                let handle = tokio::spawn(async move {
                                    v.drive(&osm_graph);
                                    v
                                });
                                handles.push(handle);
                            }
                            for handle in handles {
                                let v = handle.await.unwrap();
                                step_accumulator += v.steps;
                            }
                        }
                    },
                }

                let end = std::time::Instant::now();
                let time = end - start;
                log::info!(
                    "[{}] Finished in {:?} microseconds",
                    ROOT_RANK,
                    time.as_micros()
                );
                log::info!(
                    "Finished {} vehicles in {} steps",
                    num_vehicles,
                    step_accumulator
                );
            }
            Ok(())
        }
    }
}

// Root main event loop
// This is the main event loop for the root process
// 1. It is responsible for sending vehicles to the leafs and receiving them back
// 2. It receives edge length requests from the leafs and sends the length back
// 3. It receives termination notifications from the leafs
// 4. Once all vehicles are done, it sends termination notifications to the leafs
//    and then terminates itself
fn root_event_loop(
    num_vehicles: usize,
    finishing_threshold: usize,
    world: SystemCommunicator,
    size: i32,
    rank: i32,
    partitions: usize,
    osm_graph: &OSMGraph,
    my_graph: &GraphMap<Osmid, f64, Directed>,
    min_speed: f64,
    max_speed: f64,
) -> Result<()> {
    let mut finished_vehicle_counter = 0;
    let mut step_accumulator = 0;
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
    let mut vehicle_counter = 0;
    // send vehicles
    while vehicle_counter < num_vehicles {
        let v = Vehicle::generate_default(my_graph, min_speed, max_speed).unwrap();

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
                        log::warn!("[{}] Failed to send vehicle after receive: {:?}", rank, err);
                        continue;
                    }
                }
            }
            EDGE_LENGTH_REQUEST => {
                log::debug!("[{}] Received edge length request len={}", rank, msg.len());
                let el_req = EdgeLengthRequest::from_bytes(msg).unwrap();
                log::debug!("[{}] {:?}", rank, el_req);
                let from = el_req.from;
                let to = el_req.to;

                let mut edges = my_graph.edges_directed(from, petgraph::Direction::Outgoing);

                log::debug!("[{}] Searching for edge from {} to {}", rank, from, to);
                let el: f64 = match edges.find(|e| e.1 == to) {
                    Some(e) => *e.2,
                    None => {
                        log::debug!(
                            "[{}] No edge found for from={} to={}. Recalculating the path.",
                            rank,
                            from,
                            to
                        );
                        // NOTE: recalculating the way, and send the distance of the path instead of 0; This is a hack
                        // The graph and the provided data can be inconsistent, due to inprecise
                        // annotations in the OSM data
                        get_path_length(from, to, my_graph.clone())
                    }
                };
                let v = vec![el];
                log::debug!("[{}] Sending edge length response", rank);
                world
                    .process_at_rank(status.source_rank())
                    .send_with_tag(&v[..], EDGE_LENGTH_RESPONSE);
                log::debug!("[{}] Sent edge length response", rank);
            }
            LEAF_ROOT_VEHICLE_FINISH => {
                let v = Vehicle::from_bytes(msg).unwrap();
                finished_vehicle_counter += 1;
                step_accumulator += v.steps;
                if finished_vehicle_counter >= finishing_threshold {
                    log::info!(
                        "[{}] Finished {} vehicles in {} steps",
                        rank,
                        finished_vehicle_counter,
                        step_accumulator
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
    Ok(())
}

// Handling of Events the leaf emits
fn process_leaf_event(
    parallelism: Parallelism,
    thread_runtime: ThreadRuntime,
    world: SystemCommunicator,
    rank: i32,
    mm: &Arc<Mutex<OSMGraph>>,
    msg: Vec<u8>,
    status: Status,
) -> bool {
    match status.tag() {
        ROOT_LEAF_VEHICLE => {
            let o_data = Arc::clone(mm);
            match parallelism {
                Parallelism::SingleThreaded => {
                    single_drive(world, rank, &msg, status, o_data);
                    todo!()
                }
                Parallelism::MultiThreaded => {
                    match thread_runtime {
                        cli::ThreadRuntime::RustThreads => {
                            // fire and forget
                            mpi_drive(world, rank, &msg, status, o_data);
                        }
                        cli::ThreadRuntime::Tokio => {
                            mpi_tokio_drive(world, rank, &msg, status, o_data);
                        }
                    }
                }
            }
        }
        ROOT_LEAF_TERMINATE => {
            log::debug!("[{}] Received termination notification", rank);
            return true;
        }
        // proxy edge length response
        EDGE_LENGTH_RESPONSE => {
            log::debug!("[{}] Received edge length response", rank);
            world
                .this_process()
                .send_with_tag(&msg[..], EDGE_LENGTH_RESPONSE);
        }
        _ => {
            log::error!(
                "[{}] Received unknown message with unknown tag {}->{} from {}",
                rank,
                status.tag(),
                msg.len(),
                status.source_rank()
            );
        }
    }
    false
}

// Processes a vehicle
fn single_drive(
    world: SystemCommunicator,
    rank: i32,
    msg: &[u8],
    status: Status,
    o_data: Arc<Mutex<OSMGraph>>,
) -> bool {
    let msg = msg.to_owned();
    let lock = o_data.lock().unwrap();
    let cont = process_vehicle(world, rank, &lock, msg, status);
    match cont {
        Ok(cont) => cont,
        Err(err) => {
            log::error!("[{}] Error while processing vehicle: {:?}", rank, err);
            false
        }
    }
}

// Processes a vehicle asyncronously
fn mpi_drive(
    world: SystemCommunicator,
    rank: i32,
    msg: &[u8],
    status: Status,
    o_data: Arc<Mutex<OSMGraph>>,
) {
    let msg = msg.to_owned();

    thread::spawn(move || {
        let lock = match o_data.lock() {
            Ok(lock) => lock,
            Err(err) => {
                log::error!("[{}] Error while locking data: {:?}", rank, err);
                return false;
            }
        };
        let cont = process_vehicle(world, rank, &lock, msg, status);
        match cont {
            Ok(cont) => cont,
            Err(err) => {
                log::error!("[{}] Error while processing vehicle: {:?}", rank, err);
                false
            }
        }
    });
}

// Processes a vehicle asyncronously using tokio
fn mpi_tokio_drive(
    world: SystemCommunicator,
    rank: i32,
    msg: &[u8],
    status: Status,
    o_data: Arc<Mutex<OSMGraph>>,
) -> JoinHandle<bool> {
    let msg = msg.to_owned();

    let result = std::panic::catch_unwind(|| {
        tokio::spawn(async move {
            let lock_result = o_data.lock();
            let lock = match lock_result {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let cont = process_vehicle(world, rank, &lock, msg.clone(), status);
            match cont {
                Ok(cont) => cont,
                Err(err) => {
                    log::error!("[{}] Error while processing vehicle: {:?}", rank, err);
                    false
                }
            }
        })
    });

    match result {
        Ok(join_handle) => join_handle,
        Err(_) => {
            log::error!("Error while spawning tokio task");
            tokio::spawn(async { false })
        }
    }
}

// Receives a vehicle, processes it and sends it to the next rank
fn process_vehicle(
    world: SystemCommunicator,
    rank: Rank,
    part: &OSMGraph,
    msg: Vec<u8>,
    status: Status,
) -> Result<bool> {
    let mut v = Vehicle::from_bytes(msg).unwrap();
    log::debug!(
        "[{}] Received vehicle from rank {} ID {}",
        rank,
        status.source_rank(),
        v.id
    );

    // vehicle is done
    if v.is_parked || v.prev_id == v.next_id {
        log::debug!("[{}] - 1 Vehicle {} is done driving", rank, v.id);
        return Ok(true);
    }

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
        .process_at_rank(ROOT_RANK)
        .send_with_tag(&buf[..], EDGE_LENGTH_REQUEST);

    // get edge length
    let (el_msg, _) = world
        .this_process()
        .receive_vec_with_tag::<f64>(EDGE_LENGTH_RESPONSE);

    v.delta += el_msg[0];

    log::debug!(
        "[{}] Vehicle {} is driving from {} to {}",
        rank,
        v.id,
        v.prev_id,
        v.next_id
    );

    loop {
        log::debug!("[{}] Stepping...", v.id);
        if v.is_parked {
            // v is done
            log::debug!("[{}] Vehicle {} is done driving", rank, v.id);
            // create buffer containing the number 1
            world
                .process_at_rank(ROOT_RANK)
                .send_with_tag(&Vehicle::to_bytes(v).unwrap()[..], LEAF_ROOT_VEHICLE_FINISH);
            break;
        } else if v.marked_for_deletion {
            log::debug!("[{}] Sending vehicle {} to root", rank, v.id);
            // send vehicle to root
            world
                .process_at_rank(ROOT_RANK)
                .send_with_tag(&Vehicle::to_bytes(v).unwrap()[..], LEAF_ROOT_VEHICLE);
            break;
        }
        v.step(part);
    }
    Ok(false)
}
