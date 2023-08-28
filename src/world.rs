extern crate mpi;

use mpi::{environment::Universe, traits::*};
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

fn setup_mpi() -> (i32, i32) {
    // mpi setup
    let (universe, _) = mpi::initialize_with_threading(mpi::Threading::Multiple).unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    if size < 2 {
        panic!("Size of MPI_COMM_WORLD must be 2, but is {}!", size);
    }

    (size, rank)
}

fn setup_logging(level: cli::LoggingLevel) {
    let level = match level {
        cli::LoggingLevel::Debug => log::Level::Debug,
        cli::LoggingLevel::Info => log::Level::Info,
        cli::LoggingLevel::Warn => log::Level::Warn,
        cli::LoggingLevel::Error => log::Level::Error,
    };

    simple_logger::init_with_level(level).unwrap();
}

fn parse_input(input_file: &PathBuf) -> Result<OSMGraph> {
    // open input file
    let input_file = std::fs::File::open(input_file)?;
    // read input data for gprah
    let model: GraphInput = serde_json::from_reader(input_file).unwrap();
    // bootstrap the root graph
    OSMGraph::new(model.graph)
}

/// Entry point for the simulation
pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        cli::Commands::GraphParts {
            input_file,
            parallelism,
            num_vehicles,
            logging_level,
            thread_runtime,
            mpi,
            error_rate,
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

                log::info!(
                    "[{}] Root Size ({},{})",
                    rank,
                    my_graph.node_count(),
                    my_graph.edge_count()
                );

                log::info!("[{}] Making {} partition(s)", rank, partitions);
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
                        )?;
                    }
                    rank_number => {
                        log::debug!("[{}] Assigning leaf to rank", rank);
                        let r: usize = rank_number.try_into().unwrap();
                        let p = osm_graph.partition(partitions, r - 1)?;

                        log::info!(
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
                log::info!("[{}] Finished in {:?}", rank, end - start);
            }
            Ok(())
        }
    }
}

fn root_event_loop(
    num_vehicles: usize,
    finishing_threshold: usize,
    world: SystemCommunicator,
    size: i32,
    rank: i32,
    partitions: usize,
    osm_graph: &OSMGraph,
    my_graph: &GraphMap<Osmid, f64, Directed>,
) -> Result<()> {
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
    let mut vehicle_counter = 0;
    // send vehicles
    while vehicle_counter < num_vehicles {
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
                        log::error!("[{}] No edge found for from={} to={}", rank, from, to);
                        // NOTE: handle possible currupt algorithmic/path finding error
                        // recalculating the way, and send the distance of the path instead of 0
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
    Ok(())
}

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
            let o_data = Arc::clone(&mm);
            match parallelism {
                Parallelism::SingleThreaded => {
                    let lock = o_data.lock().unwrap();
                    let msg = msg.clone();
                    process_vehicle(world, rank, &*lock, msg, status).unwrap();
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
            log::info!("[{}] Received termination notification", rank);
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

fn mpi_drive(
    world: SystemCommunicator,
    rank: i32,
    msg: &Vec<u8>,
    status: Status,
    o_data: Arc<Mutex<OSMGraph>>,
) {
    let msg = msg.clone();
    thread::spawn(move || {
        let lock = o_data.lock().unwrap();
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

fn mpi_tokio_drive(
    world: SystemCommunicator,
    rank: i32,
    msg: &Vec<u8>,
    status: Status,
    o_data: Arc<Mutex<OSMGraph>>,
) -> JoinHandle<bool> {
    let msg = msg.clone();
    tokio::spawn(async move {
        let lock = o_data.lock().unwrap();
        let cont = process_vehicle(world, rank, &lock, msg.clone(), status);
        match cont {
            Ok(cont) => cont,
            Err(err) => {
                log::error!("[{}] Error while processing vehicle: {:?}", rank, err);
                false
            }
        }
    })
}

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

    if v.is_parked || v.prev_id == v.next_id {
        // vehicle is done
        log::info!("[{}] - 1 Vehicle {} is done driving", rank, v.id);
        return Ok(true);
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
        .process_at_rank(ROOT_RANK)
        .send_with_tag(&buf[..], EDGE_LENGTH_REQUEST);

    // get edge length
    let (el_msg, _) = world
        .this_process()
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
            log::info!("[{}] Vehicle {} is done driving", rank, v.id);
            // create buffer containing the number 1
            let buf = vec![1];
            world
                .process_at_rank(ROOT_RANK)
                .send_with_tag(&buf[..], LEAF_ROOT_VEHICLE_FINISH);
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
