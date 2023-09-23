use crate::models::vehicle::Vehicle;
use crate::prelude::*;
use nanoid::nanoid;
use std::vec::Vec;

/// Builder for creating instances of the `Vehicle` struct.
#[derive(Debug)]
pub struct VehicleBuilder {
    /// The speed of the vehicle.
    pub speed: f64,
    /// The path IDs associated with the vehicle's route.
    pub path_ids: Vec<usize>,

    /// The delta value for the vehicle.
    pub delta: f64,
    /// Indicates whether the vehicle is parked.
    pub is_parked: bool,

    /// The previous ID of the vehicle.
    pub prev_id: usize,
    /// The next ID of the vehicle.
    pub next_id: usize,
}

impl VehicleBuilder {
    /// Creates a new `VehicleBuilder` instance with default values.
    pub fn new() -> VehicleBuilder {
        VehicleBuilder {
            speed: 0.0,
            path_ids: Vec::new(),
            delta: 0.0,
            is_parked: false,
            prev_id: 0,
            next_id: 0,
        }
    }

    /// Sets the speed of the vehicle.
    pub fn with_speed(mut self, speed: f64) -> VehicleBuilder {
        self.speed = speed;
        self
    }

    /// Sets the path IDs for the vehicle's route.
    pub fn with_path_ids(mut self, path_ids: Vec<usize>) -> VehicleBuilder {
        self.path_ids = path_ids;
        self
    }

    /// Sets the previous ID of the vehicle.
    pub fn with_prev_id(mut self, prev_id: usize) -> VehicleBuilder {
        self.prev_id = prev_id;
        self
    }

    /// Sets the next ID of the vehicle.
    pub fn with_next_id(mut self, next_id: usize) -> VehicleBuilder {
        self.next_id = next_id;
        self
    }

    /// Sets the delta value for the vehicle.
    pub fn with_delta(mut self, delta: f64) -> VehicleBuilder {
        self.delta = delta;
        self
    }

    /// Sets whether the vehicle is parked.
    pub fn with_is_parked(mut self, is_parked: bool) -> VehicleBuilder {
        self.is_parked = is_parked;
        self
    }

    /// Performs validation checks on the builder's data.
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

    /// Builds a `Vehicle` instance from the builder's data.
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
            steps: 0,
        })
    }
}
