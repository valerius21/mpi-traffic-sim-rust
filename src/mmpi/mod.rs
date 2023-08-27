use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
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
