use rand::Rng;

use crate::prelude::Result;

// * NOTE: use bincode https://github.com/bincode-org/bincode
pub trait MpiMessageContent<T> {
    fn to_bytes(data: T) -> Result<Vec<u8>>;
    fn from_bytes(data: Vec<u8>) -> Result<T>;
}

pub fn get_random_vector_element<T>(v: &Vec<T>) -> Option<&T> {
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..v.len());
    v.get(random_index)
}

pub const MAX_NUMBER_OF_VEHICLES: usize = usize::MAX / 2;

pub fn random_velocity(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}
