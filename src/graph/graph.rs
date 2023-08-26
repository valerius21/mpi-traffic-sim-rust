use std::collections::{HashMap, HashSet};

use crate::{
    graph::rect::Point,
    models::graph_input::{Edge, Graph as GI, Vertex},
    prelude::{Error, Result},
};
use petgraph::{csr::NodeIndex, Directed, Graph};

use super::rect::Rect;

pub type GraphID = u32;

#[derive(Debug, Default, Clone)]
pub struct OSMGraph {
    pub id: GraphID,
    pub graph: Graph<Vertex, Edge, Directed, usize>,
}

// A lot of those methods dance around the fact that the graph
// uses it's own ID's / indcies and not the OSM ID's.
pub trait GUtils {
    fn new(id: GraphID, osm_graph: GI) -> Result<OSMGraph>;
    fn get_graph(&self) -> &Graph<Vertex, Edge, Directed, usize>;
    fn get_vertices(&self) -> Vec<Vertex>;
    fn get_edges(&self) -> Vec<Edge>;
    fn hashmap_osm_id_to_index(&self) -> HashMap<usize, usize>;
    fn hashmap_index_to_osm_id(&self) -> HashMap<usize, usize>;
}

pub trait GPartition {
    // n => number of partitions
    // i => index of partition
    // id => GraphID
    fn partition(&self, n: u32, i: u32, id: GraphID) -> Result<OSMGraph>;
}

impl GPartition for OSMGraph {
    fn partition(&self, n: u32, i: u32, id: GraphID) -> Result<OSMGraph> {
        let vtx_lst = self.get_vertices();
        let rect = Rect::new(vtx_lst.clone())?;
        let x_delta: f64 = (rect.top_right.x - rect.bottom_left.x) / n as f64;
        let x_offset: f64 = x_delta * i as f64;

        // new rect with offset
        let offset_bottom_left = Point {
            x: rect.bottom_left.x + x_offset,
            y: rect.bottom_left.y,
        };
        let offset_top_right = Point {
            x: offset_bottom_left.x + x_delta,
            y: rect.top_right.y,
        };

        // finish new rect with target_rect
        let mut target_rect = Rect {
            bottom_left: offset_bottom_left,
            top_right: offset_top_right,
            vertices: vtx_lst, // temporary clone all verticies from the struct
        };

        let mut t_vrtx = target_rect.vertices.clone();
        t_vrtx.retain(|x| target_rect.in_rect(x.clone()));

        // NOTE:Dancemove 💃
        let mut osmid_to_index_map = self.hashmap_osm_id_to_index();

        // filter for vertices in target rect
        osmid_to_index_map
            .retain(|_, index| target_rect.in_rect(self.get_vertices()[*index].clone()));

        target_rect.vertices = osmid_to_index_map
            .iter()
            .map(|(_, index)| self.get_vertices()[*index].clone())
            .collect();

        // NOTE: End Dancemove 💃
        let mut inside_edges = Vec::<_>::new();
        for e in self.graph.edge_indices() {
            let weight = match self.graph.edge_weight(e) {
                Some(weight) => weight,
                None => {
                    return Err(Error::NoWeightFound(String::from(
                        "No Edge Weight for Partitioning",
                    )))
                }
            };

            if osmid_to_index_map.contains_key(&weight.from)
                && osmid_to_index_map.contains_key(&weight.to)
            {
                inside_edges.push(weight.clone());
            }
        }

        let mut part_graph = Graph::<Vertex, Edge, Directed, usize>::with_capacity(
            target_rect.vertices.len(),
            inside_edges.len(),
        );

        let mut insertion_map = HashMap::<usize, _>::new();
        for vertex in target_rect.vertices.iter() {
            insertion_map.insert(vertex.osm_id, part_graph.add_node(vertex.clone()));
        }

        for edge in inside_edges.iter() {
            let from = match insertion_map.get(&edge.from) {
                Some(f) => *f,
                None => Err(Error::NoInnerEdge(String::from("No inner edge")))?,
            };
            let to = match insertion_map.get(&edge.to) {
                Some(t) => *t,
                None => Err(Error::NoInnerEdge(String::from("No inner edge")))?,
            };

            part_graph.add_edge(from, to, edge.clone());
        }

        let osm_g = OSMGraph {
            graph: part_graph,
            id,
        };

        Ok(osm_g)
    }
}

// TODO: needs proper builder pattern to allow construction for part graph
impl GUtils for OSMGraph {
    fn new(id: GraphID, osm_graph: GI) -> Result<OSMGraph> {
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
            let from = match vertex_vec.iter().position(|x| x.osm_id == edge.from) {
                Some(f) => f as NodeIndex<usize>,
                None => Err(Error::ElementNotInVector(String::from(
                    "Vertex vector does not include 'from'",
                )))?,
            };

            let to = match vertex_vec.iter().position(|x| x.osm_id == edge.to) {
                Some(t) => t as NodeIndex<usize>,
                None => Err(Error::ElementNotInVector(String::from(
                    "Vertex vector does not include 'to'",
                )))?,
            };

            r_graph.add_edge(from.into(), to.into(), edge.clone());
        }

        Ok(Self { graph: r_graph, id })
    }

    fn get_graph(&self) -> &Graph<Vertex, Edge, Directed, usize> {
        &self.graph
    }

    fn get_vertices(&self) -> Vec<Vertex> {
        self.get_graph()
            .node_indices()
            .map(|x| self.get_graph()[x].clone())
            .collect()
    }

    fn get_edges(&self) -> Vec<Edge> {
        self.get_graph()
            .edge_indices()
            .map(|x| self.get_graph()[x].clone())
            .collect()
    }

    fn hashmap_osm_id_to_index(&self) -> HashMap<usize, usize> {
        let mut map = HashMap::<usize, usize>::new();
        for (i, v) in self.get_vertices().iter().enumerate() {
            map.insert(v.osm_id, i);
        }
        map
    }

    fn hashmap_index_to_osm_id(&self) -> HashMap<usize, usize> {
        let mut map = HashMap::<usize, usize>::new();
        for (i, v) in self.get_vertices().iter().enumerate() {
            map.insert(i, v.osm_id);
        }
        map
    }
}
