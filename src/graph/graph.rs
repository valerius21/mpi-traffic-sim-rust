use std::collections::HashSet;

use crate::models::graph_input::{Edge, Graph as GI, Vertex};
use petgraph::{csr::NodeIndex, Directed, Graph};

#[derive(Debug, Clone)]
pub struct OSMGraph {
    pub graph: Graph<Vertex, Edge, Directed, usize>,
}

pub trait GUtils {
    fn new(osm_graph: GI) -> OSMGraph;
}

impl GUtils for OSMGraph {
    fn new(osm_graph: GI) -> OSMGraph {
        let e_lst = osm_graph
            .edges
            .iter()
            .map(|edge| (edge.from, edge.to))
            .collect::<Vec<(usize, usize)>>();

        let mut vertex_vec = Vec::<Vertex>::new();

        for edge in osm_graph.edges.iter() {
            for vertex in osm_graph.vertices.iter() {
                if vertex.osm_id == edge.from {
                    vertex_vec.push(vertex.clone());
                }
                if vertex.osm_id == edge.to {
                    vertex_vec.push(vertex.clone());
                }
            }
        }

        let mut vtx = HashSet::new();
        vertex_vec.retain(|x| vtx.insert(x.osm_id));

        let mut r_graph =
            Graph::<Vertex, Edge, Directed, usize>::with_capacity(vertex_vec.len(), e_lst.len());

        for vertex in vertex_vec.iter() {
            r_graph.add_node(vertex.clone());
        }

        for edge in osm_graph.edges.iter() {
            let from = vertex_vec
                .iter()
                .position(|x| x.osm_id == edge.from)
                .unwrap() as NodeIndex<usize>;
            let to = vertex_vec.iter().position(|x| x.osm_id == edge.to).unwrap();
            r_graph.add_edge(from.into(), to.into(), edge.clone());
        }

        Self { graph: r_graph }
    }
}
