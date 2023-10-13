use rand::Rng;

use crate::prelude::Result;

// Message content interface for inter MPI communication
pub trait MpiMessageContent<T> {
    fn to_bytes(data: T) -> Result<Vec<u8>>;
    fn from_bytes(data: Vec<u8>) -> Result<T>;
}

// Get a random element from a vector
pub fn get_random_vector_element<T>(v: &Vec<T>) -> Option<&T> {
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..v.len());
    v.get(random_index)
}

pub const MAX_NUMBER_OF_VEHICLES: usize = usize::MAX / 2;

// Get a random number between min and max, representing velocity in m/s
pub fn random_velocity(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}
