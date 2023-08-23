use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphInput {
    pub filename: String,
    pub size: u64,
    pub graph: Graph,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Graph {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge {
    pub from: u64,
    pub to: u64,
    pub length: f64,
    pub max_speed: String,
    pub name: String,
    pub osm_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub osm_id: u64,
}
