use crate::graph::osm_graph::Osmid;
use crate::utils::{get_random_vector_element, random_velocity, MpiMessageContent};
use crate::{graph::osm_graph::OSMGraph, prelude::*};
use bincode::{deserialize, serialize};
use petgraph::algo::astar;
use petgraph::prelude::GraphMap;
use petgraph::Directed;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::vehicle_builder::VehicleBuilder;

/// Represents a vehicle that can move within a graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct Vehicle {
    /// The unique identifier of the vehicle.
    pub id: String,
    /// The path IDs representing the route the vehicle follows.
    pub path_ids: Vec<Osmid>,
    /// The speed of the vehicle.
    pub speed: f64,
    /// The delta value for the vehicle.
    pub delta: f64,
    /// The next path ID the vehicle is moving toward.
    pub next_id: Osmid,
    /// The previous path ID the vehicle was at.
    pub prev_id: Osmid,
    /// Indicates whether the vehicle is parked.
    pub is_parked: bool,
    /// The remaining distance the vehicle has to travel.
    pub distance_remaining: f64,
    /// Indicates whether the vehicle is marked for deletion.
    pub marked_for_deletion: bool,
    /// The number of steps the vehicle has taken.
    pub steps: u64,
}

/// A trait for moveable objects.
pub trait Moveable {
    /// Drives the moveable object within a graph.
    fn drive(&mut self, graph: &OSMGraph);
    /// Takes a step for the moveable object.
    fn step(&mut self, graph: &OSMGraph);
    /// Gets the next node for the moveable object.
    fn get_next_node(&mut self, prev_id: Osmid, current_graph: &OSMGraph) -> Option<Osmid>;
    /// Calculates the step for the moveable object.
    fn calculate_step(&mut self);
}

impl Moveable for Vehicle {
    fn drive(&mut self, osm_graph: &OSMGraph) {
        while !self.is_parked {
            self.step(osm_graph);
        }
    }

    fn calculate_step(&mut self) {
        // NOTE: adding CPU-intensive placeholder function simulating a complex calculation by
        // generating a random prime number
        #[cfg(feature = "complex-calculation")]
        {
            let mut rng = rand::thread_rng();
            let number = rand::Rng::gen_range(&mut rng, 1_000_000..=3_000_000);
            let _some_unused_prime = primal::Primes::all().nth(number).unwrap();
        }
        // WARN: Actually crucial code. Do not remove.
        self.distance_remaining -= self.speed;
    }

    fn step(&mut self, osm_graph: &OSMGraph) {
        log::debug!("Vehicle {} is stepping", self.id);
        self.steps += 1;

        let gg = &osm_graph.graph;
        if !gg.contains_edge(self.prev_id, self.next_id) {
            let old_prev = self.prev_id;
            self.prev_id = self.next_id;
            self.next_id = match self.get_next_node(self.next_id, osm_graph) {
                Some(id) => id,
                None => {
                    self.prev_id = old_prev;
                    if self.marked_for_deletion || self.is_parked {
                        return;
                    }
                    log::error!(
                        "No next node found for prev_id={}, V={:?}",
                        self.prev_id,
                        self
                    );
                    self.next_id
                }
            }
        }

        let edge = match gg
            .edges_directed(self.prev_id, petgraph::Direction::Outgoing)
            .find(|e| e.1 == self.next_id)
        {
            Some(e) => e,
            None => {
                self.marked_for_deletion = true;
                log::debug!(
                    "No edge found {}->{}  @ {:?} while stepping. Marking for deletion",
                    self.prev_id,
                    self.next_id,
                    self.id,
                );
                return;
            }
        };
        let length = *edge.2;

        self.distance_remaining = length;
        self.distance_remaining += self.delta;

        // * NOTE: Vehicle travels the entire distance in one step
        // *        This is not correct, as the vehicle should only travel
        // *        the distance of its speed in one step.
        // *        Maybe change to "if"
        while (self.distance_remaining >= self.speed) && (self.distance_remaining - self.speed > 0.)
        {
            self.calculate_step();
        }
        self.delta = self.distance_remaining;
        self.distance_remaining = 0_f64;

        let tmp = match self.get_next_node(self.next_id, osm_graph) {
            Some(id) => id,
            None => {
                if self.marked_for_deletion {
                    log::debug!("Vehicle {} is marked for deletion", self.id);
                    return;
                } else if self.is_parked
                    || self.path_ids.last().is_some()
                        && self.path_ids.last().unwrap() == &self.prev_id
                {
                    self.is_parked = true;
                    log::debug!("Vehicle - 3 {} is done driving", self.id);
                    return;
                } else {
                    panic!(
                        "No next node found for prev_id={}, V={:?}",
                        self.prev_id, self
                    );
                }
            }
        };
        log::debug!("Prev={} Tmp={} Next={}", self.prev_id, tmp, self.next_id);
        self.prev_id = self.next_id;
        self.next_id = tmp;
        log::debug!(
            "Vehicle {} is stepping to {}->{}",
            self.id,
            self.prev_id,
            self.next_id
        );

        if self.next_id == 0 {
            self.is_parked = true;
        }
    }

    fn get_next_node(&mut self, prev_id: Osmid, current_graph: &OSMGraph) -> Option<Osmid> {
        // if prev_id is last element in path_ids, return None
        if self.path_ids.last().unwrap() == &prev_id
            || self.path_ids.get(self.path_ids.len() - 2).unwrap() == &prev_id
        {
            self.is_parked = true;
            return None;
        }

        // get the index of the next id, which is the index of prev_id + 1
        let next_id_index = match self.path_ids.par_iter().position_any(|&x| x == prev_id) {
            Some(i) => i + 1,
            None => {
                log::error!("No next id found for prev_id={}", prev_id);
                return None;
            }
        };

        // check if next_id_index is out of bounds
        if next_id_index >= self.path_ids.len() {
            log::error!(
                "next_id_index={} is out of bounds for path_ids.len()={}",
                next_id_index,
                self.path_ids.len()
            );
            return None;
        }

        // get the next id
        let next_id = self.path_ids[next_id_index];
        log::debug!("prev_id={} next_id={}", self.prev_id, next_id);

        // check if next_id is in the current graph, if not return None and mark for deletion
        let cg = current_graph.graph.clone();
        if !cg.nodes().any(|x| x == next_id) {
            log::debug!(
                "next_id={} is not in current graph. Marking VID={} for deletion",
                next_id,
                self.id
            );
            self.marked_for_deletion = true;
            self.prev_id = next_id;
            self.next_id = self.path_ids[next_id_index + 1];
            return None;
        }

        Some(next_id)
    }
}

impl MpiMessageContent<Vehicle> for Vehicle {
    fn from_bytes(data: Vec<u8>) -> Result<Vehicle> {
        match deserialize::<Vehicle>(&data) {
            Ok(v) => Ok(v),
            Err(err) => Err(Error::Bincode(err)),
        }
    }

    fn to_bytes(data: Vehicle) -> Result<Vec<u8>> {
        match serialize(&data) {
            Ok(arr) => Ok(arr),
            Err(err) => Err(Error::Bincode(err)),
        }
    }
}

impl Vehicle {
    /// Generates a default vehicle with a random route and speed within a given graph.
    pub fn generate_default(
        graph: &GraphMap<usize, f64, Directed>,
        min_speed: f64,
        max_speed: f64,
    ) -> Result<Vehicle> {
        let vtx: Vec<_> = graph.nodes().collect();
        let mut path = None;
        let mut path_length = 0;

        while path.is_none() || path_length < 5 {
            let start = match get_random_vector_element(&vtx) {
                Some(v) => *v,
                None => Err(Error::Generic(String::from("No random vertex found")))?,
            };

            let end = match get_random_vector_element(&vtx) {
                Some(v) => *v,
                None => Err(Error::Generic(String::from("No random vertex found")))?,
            };

            path = astar(&graph, start, |finish| finish == end, |e| *e.2, |_| 0.);

            path_length = match path.as_ref() {
                Some(p) => p.1.len(),
                None => 0,
            };
        }

        let path = match path {
            Some(p) => p.1,
            None => Err(Error::Generic(String::from("No path found")))?,
        };

        let velocity = random_velocity(min_speed, max_speed);

        let veh = VehicleBuilder::new()
            .with_delta(0.0)
            .with_is_parked(false)
            .with_speed(velocity)
            .with_path_ids(path.clone())
            .with_prev_id(path[0])
            .with_next_id(path[1])
            .build()?;
        Ok(veh)
    }
}
