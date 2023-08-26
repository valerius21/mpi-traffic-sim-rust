use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphInput {
    pub filename: String,
    pub size: usize,
    pub graph: Graph,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Graph {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub length: f64,
    pub max_speed: String,
    pub name: String,
    pub osm_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub osm_id: usize,
}

impl std::cmp::PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.osm_id == other.osm_id
    }
}

impl std::cmp::Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.osm_id.hash(state);
    }
}
