use crate::graph::graph::GUtils;
use crate::utils::MpiMessageContent;
use crate::{graph::graph::OSMGraph, prelude::*};
use bincode::{deserialize, serialize};
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

trait Moveable {
    fn drive(&mut self, graph: &OSMGraph);
    fn step(&mut self, graph: &OSMGraph);
    fn get_next_node(&mut self, id: OSMID, graph: &OSMGraph) -> OSMID;
}

impl Moveable for Vehicle {
    fn drive(&mut self, osm_graph: &OSMGraph) {
        while !self.is_parked {
            self.step(osm_graph);
        }
    }

    fn step(&mut self, osm_graph: &OSMGraph) {
        if self.next_id < 0 as usize {
            self.is_parked = true;
        }

        let gg = osm_graph.get_graph();

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
            None => panic!("No edge found"),
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
        let next_step_id = self.get_next_node(self.next_id, osm_graph);
        if self.marked_for_deletion {
            return;
        } else if next_step_id == 0 {
            self.is_parked = true;
            return;
        }

        self.prev_id = self.next_id;
        self.next_id = self.get_next_node(self.prev_id, osm_graph);

        if self.next_id == 0 {
            self.is_parked = true;
            return;
        }
    }

    fn get_next_node(&mut self, id: OSMID, osm_graph: &OSMGraph) -> OSMID {
        //     var prevIdIndex = -1
        let mut prev_id_index: isize = -1;

        let length = self.path_ids.len();

        for i in 0..length {
            if self.path_ids[i] == id {
                prev_id_index = i as isize;
            }
        }
        // isLastIdx := prevIdIndex == len(v.PathIDs)-1
        let is_last_idx = prev_id_index == (length - 1) as isize;

        if self.next_id == 0 || is_last_idx || self.is_parked {
            // if vehicle is parked nextID is not 0
            self.is_parked = true;
            return 0;
        }

        // nextID := v.PathIDs[prevIdIndex+1]
        let next_id = self.path_ids[(prev_id_index + 1) as usize];

        if !osm_graph.get_graph().contains_node(next_id) {
            // III.9.2
            self.marked_for_deletion = true;
            return 0;
        }

        next_id
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
