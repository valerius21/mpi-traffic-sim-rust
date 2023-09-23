// src/models/graph_input.rs

use std::hash::Hash;

use serde::{Deserialize, Serialize};

/// Represents input data for a graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphInput {
    /// The name of the file associated with this input.
    pub filename: String,
    /// The size of the graph.
    pub size: usize,
    /// The graph data structure.
    pub graph: Graph,
}

/// Represents a graph containing vertices and edges.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Graph {
    /// A list of vertices in the graph.
    pub vertices: Vec<Vertex>,
    /// A list of edges in the graph.
    pub edges: Vec<Edge>,
}

/// Represents an edge in the graph.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge {
    /// The source vertex of the edge.
    pub from: usize,
    /// The target vertex of the edge.
    pub to: usize,
    /// The length of the edge.
    pub length: f64,
    /// The maximum speed on the edge.
    pub max_speed: String,
    /// The name of the edge.
    pub name: String,
    /// The OpenStreetMap (OSM) ID of the edge.
    pub osm_id: String,
}

/// Represents a vertex in the graph.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Vertex {
    /// The X-coordinate of the vertex.
    pub x: f64,
    /// The Y-coordinate of the vertex.
    pub y: f64,
    /// The OpenStreetMap (OSM) ID of the vertex.
    pub osm_id: usize,
}

impl std::cmp::PartialEq for Vertex {
    /// Implements partial equality for vertices based on their OSM IDs.
    fn eq(&self, other: &Self) -> bool {
        self.osm_id == other.osm_id
    }
}

impl std::cmp::Eq for Vertex {}

impl Hash for Vertex {
    /// Implements hashing for vertices based on their OSM IDs.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.osm_id.hash(state);
    }
}
