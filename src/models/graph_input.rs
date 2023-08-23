use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphInput {
    pub filename: String,
    pub size: usize,
    pub graph: Graph,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
