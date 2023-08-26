use crate::{graph::graph::GraphID, utils::MpiMessageContent};
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Vehicle {
    id: String,
    path_ids: Vec<usize>,
    speed: f64,
    delta: f64,
    next_id: usize,
    prev_id: usize,
    is_parked: bool,
    distance_remaining: f64,
    marked_for_deletion: bool,
    graph: GraphID, // TODO: use option
}

impl MpiMessageContent<Vehicle> for Vehicle {
    fn from_bytes(data: Vec<u8>) -> Vehicle {
        match deserialize(&data) {
            Ok(v) => v,
            Err(_) => panic!("Cannot serialize vehicle"),
        }
    }

    fn to_bytes(data: Vehicle) -> Vec<u8> {
        // TODO: implement results
        match serialize(&data) {
            Ok(arr) => arr,
            Err(_) => panic!("Cannot deserialize vehicle"),
        }
    }
}
