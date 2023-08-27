use crate::prelude::*;
use crate::{graph::graph::OSMGraph, models::vehicle::Vehicle};
use nanoid::nanoid;
use std::vec::Vec;

#[derive(Debug)]
pub struct VehicleBuilder {
    pub speed: f64,
    pub path_ids: Vec<usize>,

    pub delta: f64,
    pub is_parked: bool,

    pub prev_id: usize,
    pub next_id: usize,
}

impl VehicleBuilder {
    pub fn new() -> VehicleBuilder {
        VehicleBuilder {
            speed: 0.0,
            path_ids: Vec::new(),
            delta: 0.0,
            is_parked: false,
            prev_id: 0,
            next_id: 0,
            // graph,
        }
    }

    pub fn with_speed(mut self, speed: f64) -> VehicleBuilder {
        self.speed = speed;
        self
    }

    pub fn with_path_ids(mut self, path_ids: Vec<usize>) -> VehicleBuilder {
        self.path_ids = path_ids;
        self
    }

    // redundant?
    // pub fn with_graph(mut self, graph: &'a OSMGraph) -> VehicleBuilder {
    //     self.graph = graph;
    //     self
    // }

    pub fn with_prev_id(mut self, prev_id: usize) -> VehicleBuilder {
        self.prev_id = prev_id;
        self
    }
    pub fn with_next_id(mut self, next_id: usize) -> VehicleBuilder {
        self.next_id = next_id;
        self
    }

    pub fn with_delta(mut self, delta: f64) -> VehicleBuilder {
        self.delta = delta;
        self
    }

    pub fn with_is_parked(mut self, is_parked: bool) -> VehicleBuilder {
        self.is_parked = is_parked;
        self
    }

    fn check(&mut self) -> crate::prelude::Result<()> {
        if self.speed == 0.0 {
            return Err(Error::Generic(String::from("Speed is 0.0")));
        }
        if self.path_ids.len() < 2 {
            return Err(Error::Generic(String::from("PathIDs length < 2")));
        }
        if self.prev_id < 1 {
            return Err(Error::Generic(String::from("PrevID < 1")));
        }

        Ok(())
    }

    pub fn from_bytes() -> Vehicle {
        todo!("implement trait to vehicle builder?")
    }

    pub fn build(&mut self) -> crate::prelude::Result<Vehicle> {
        let alphabet: [char; 16] = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
        ];

        let id = nanoid!(10, &alphabet);

        self.check()?;

        Ok(Vehicle {
            id,
            path_ids: self.path_ids.clone(),
            speed: self.speed,
            delta: self.delta,
            next_id: self.next_id,
            prev_id: self.prev_id,
            is_parked: self.is_parked,
            distance_remaining: 0.0,
            marked_for_deletion: false,
        })
    }
}
