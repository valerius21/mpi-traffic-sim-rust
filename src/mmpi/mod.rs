use std::collections::HashMap;

use bincode::{deserialize, serialize};
use mpi::topology::SystemCommunicator;
use mpi::traits::*;
use serde::{Deserialize, Serialize};

use crate::models::vehicle::Vehicle;
use crate::prelude::*;
use crate::{graph::graph::OSMID, utils::MpiMessageContent};

// Root to leaf vehicle sending tag
pub const ROOT_LEAF_VEHICLE: i32 = 0x01;

// Leaf to root vehicle sending tag
pub const LEAF_ROOT_VEHICLE: i32 = 0x02;

// Leaf asks root for edge length
pub const EDGE_LENGTH_REQUEST: i32 = 0x03;

// Root responds to leaf with edge length
pub const EDGE_LENGTH_RESPONSE: i32 = 0x04;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdgeLengthRequest {
    pub from: OSMID,
    pub to: OSMID,
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
        Some(r) => r.clone(),
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
