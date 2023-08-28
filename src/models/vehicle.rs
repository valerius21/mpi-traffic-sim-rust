use crate::streets::vehicle_builder::VehicleBuilder;
use crate::utils::{get_random_vector_element, MpiMessageContent};
use crate::{graph::graph::OSMGraph, prelude::*};
use bincode::{deserialize, serialize};
use petgraph::algo::astar;
use petgraph::prelude::GraphMap;
use petgraph::Directed;
use serde::{Deserialize, Serialize};

type OSMID = usize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: String,
    pub path_ids: Vec<OSMID>,
    pub speed: f64,
    pub delta: f64,
    pub next_id: OSMID,
    pub prev_id: OSMID,
    pub is_parked: bool,
    pub distance_remaining: f64,
    pub marked_for_deletion: bool,
}

pub trait Moveable {
    fn drive(&mut self, graph: &OSMGraph);
    fn step(&mut self, graph: &OSMGraph);
    fn get_next_node(&mut self, prev_id: OSMID, current_graph: &OSMGraph) -> Option<OSMID>;
}

impl Moveable for Vehicle {
    fn drive(&mut self, osm_graph: &OSMGraph) {
        while !self.is_parked {
            self.step(osm_graph);
        }
    }

    fn step(&mut self, osm_graph: &OSMGraph) {
        log::debug!("Vehicle {} is stepping", self.id);
        if self.next_id < 0 as usize {
            self.is_parked = true;
        }
        let gg = &osm_graph.graph;

        // let gg = osm_graph.get_graph();
        if !gg.contains_edge(self.prev_id, self.next_id) {
            let old_prev = self.prev_id;
            self.prev_id = self.next_id;
            self.next_id = match self.get_next_node(self.next_id, &osm_graph) {
                Some(id) => id,
                None => {
                    if self.marked_for_deletion || self.is_parked {
                        self.prev_id = old_prev;
                        return;
                    }
                    log::error!(
                        "No next node found for prev_id={}, V={:?}",
                        self.prev_id,
                        self
                    );
                    self.prev_id = old_prev;
                    self.next_id
                }
            }
        }

        let edge = match gg
            .all_edges()
            .filter(|e| {
                let from = e.0;
                let to = e.1;
                return from == self.prev_id && to == self.next_id;
            })
            .next()
        {
            Some(e) => e,
            None => panic!(
                "No edge found {}->{}  @ {:?}",
                self.prev_id,
                self.next_id,
                // gg.all_edges()
                //     .any(|e| e.0 == self.prev_id && e.1 == self.next_id),
                self
            ),
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
            self.distance_remaining -= self.speed;
        }
        self.delta = self.distance_remaining;
        self.distance_remaining = 0.;

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
                    log::info!("Vehicle - 3 {} is done driving", self.id);
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
            return;
        }
    }

    fn get_next_node(&mut self, prev_id: OSMID, current_graph: &OSMGraph) -> Option<OSMID> {
        // if prev_id is last element in path_ids, return None
        if self.path_ids.last().unwrap() == &prev_id
            || self.path_ids.get(self.path_ids.len() - 2).unwrap() == &prev_id
        {
            self.is_parked = true;
            return None;
        }

        // get the index of the next id, which is the index of prev_id + 1
        let next_id_index = match self.path_ids.iter().position(|&x| x == prev_id) {
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
    pub fn generate_default(graph: &GraphMap<usize, f64, Directed>) -> Result<Vehicle> {
        let vtx: Vec<_> = graph.nodes().collect();
        let mut path = None;
        let mut path_length = 0;

        while path.is_none() || path_length < 5 {
            let start = match get_random_vector_element(&vtx) {
                Some(v) => v.clone(),
                None => Err(Error::Generic(String::from("No random vertex found")))?,
            };

            let end = match get_random_vector_element(&vtx) {
                Some(v) => v.clone(),
                None => Err(Error::Generic(String::from("No random vertex found")))?,
            };

            path = astar(
                &graph,
                start,
                |finish| finish == end,
                |e| e.2.clone(),
                |_| 0.,
            );

            path_length = match path.as_ref() {
                Some(p) => p.1.len(),
                None => 0,
            };
        }

        let path = match path {
            Some(p) => p.1,
            None => Err(Error::Generic(String::from("No path found")))?,
        };

        let veh = VehicleBuilder::new()
            .with_delta(0.0)
            .with_delta(0.0)
            .with_is_parked(false)
            .with_speed(5.5)
            .with_path_ids(path.clone())
            .with_prev_id(path[0])
            .with_next_id(path[1])
            .build()?;
        Ok(veh)
    }
}
