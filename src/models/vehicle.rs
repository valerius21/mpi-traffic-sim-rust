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
    fn drive(&mut self);
    fn step(&mut self, graph: &OSMGraph);
    fn get_next_node() -> OSMID;
}

impl Moveable for Vehicle {
    fn drive(&mut self) {
        // while self.is_parked {
        //     self.step();
        // }
    }

    fn step(&mut self, osm_graph: &OSMGraph) {
        if self.next_id < 0 as usize {
            self.is_parked = true;
        }

        let gg = osm_graph.get_graph();

        // let edge = match gg.find_edge(self.prev_id, self.next_id) {
        //     Some(edge) => edge,
        //     None => panic!("edge not found in graph"),
        // };

        todo!()
    }

    fn get_next_node() -> OSMID {
        todo!()
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
