use std::collections::HashMap;

use bincode::{deserialize, serialize};
use mpi::topology::SystemCommunicator;
use mpi::traits::*;
use serde::{Deserialize, Serialize};

use crate::models::vehicle::Vehicle;
use crate::prelude::*;
use crate::{graph::osm_graph::Osmid, utils::MpiMessageContent};

pub enum Tags {
    // Root to leaf vehicle sending tag
    ROOT_LEAF_VEHICLE = 0x01,

    // Leaf to root vehicle sending tag
    LEAF_ROOT_VEHICLE = 0x02,

    // Leaf asks root for edge length
    EDGE_LENGTH_REQUEST = 0x03,

    // Root responds to leaf with edge length
    EDGE_LENGTH_RESPONSE = 0x04,

    // Leaf to Root vehicle finishng notification
    LEAF_ROOT_VEHICLE_FINISH = 0x05,

    // Root to leaf program termination notification
    ROOT_LEAF_TERMINATE = 0x06,
}

pub const ROOT_RANK: i32 = 0;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdgeLengthRequest {
    pub from: Osmid,
    pub to: Osmid,
}

impl MpiMessageContent<EdgeLengthRequest> for EdgeLengthRequest {
    fn to_bytes(data: EdgeLengthRequest) -> Result<Vec<u8>> {
        Ok(serialize(&data)?)
    }

    fn from_bytes(data: Vec<u8>) -> Result<EdgeLengthRequest> {
        Ok(deserialize(&data)?)
    }
}

pub fn map_vehicle_to_rank(
    v: Vehicle,
    node_to_rank: &HashMap<usize, i32>,
    rank: i32,
    world: SystemCommunicator,
) -> Result<()> {
    let node = v.next_id;
    let r = match node_to_rank.get(&node) {
        Some(r) => *r,
        None => {
            log::warn!("[{}] No rank found for node={}, {:?}", rank, node, v);
            return Err(crate::prelude::Error::Generic(String::from(
                "No rank found for node",
            )));
        }
    };

    let vb = Vehicle::to_bytes(v).unwrap();

    world
        .process_at_rank(r)
        .send_with_tag(&vb[..], ROOT_LEAF_VEHICLE);
    log::debug!("[{}] Sent vehicle to rank {}", rank, r);
    Ok(())
}
