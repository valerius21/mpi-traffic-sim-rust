use std::collections::{HashMap, HashSet};

use crate::{
    graph::rect::Point,
    models::graph_input::{Edge, Graph as GI, Vertex},
    prelude::{Error, Result},
};
use petgraph::{csr::NodeIndex, prelude::DiGraphMap, Directed, Graph};

use super::rect::Rect;

pub type GraphID = u32;
pub type OSMID = usize;

#[derive(Debug, Default, Clone)]
pub struct OSMGraph {
    osm: GI,
    pub graph: petgraph::prelude::GraphMap<OSMID, f64, Directed>,
}

// A lot of those methods dance around the fact that the graph
// uses it's own ID's / indcies and not the OSM ID's.
pub trait GUtils {
    fn new(osm_graph: GI) -> Result<OSMGraph>;
    fn get_graph(&self) -> &petgraph::prelude::GraphMap<OSMID, f64, Directed>;
    fn get_vertices(&self) -> Vec<Vertex>;
    fn get_edges(&self) -> Vec<Edge>;
    fn hashmap_osm_id_to_index(&self) -> HashMap<usize, usize>;
    fn hashmap_index_to_osm_id(&self) -> HashMap<usize, usize>;
}

pub trait GPartition {
    // n => number of partitions
    // i => index of partition
    // id => GraphID
    fn partition(&self, n: usize, i: usize) -> Result<OSMGraph>;
}

impl GPartition for OSMGraph {
    fn partition(&self, n: usize, i: usize) -> Result<OSMGraph> {
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

        // filter for vertices in target rect
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

        let verticies = target_rect
            .vertices
            .clone()
            .into_iter()
            .map(|v| v.osm_id)
            .collect::<HashSet<OSMID>>();
        let inside_edges = self.graph.all_edges().filter(|e: &(OSMID, OSMID, &f64)| {
            verticies.contains(&e.0) && verticies.contains(&e.1)
        });

        let child_graph: petgraph::prelude::GraphMap<OSMID, f64, Directed> =
            DiGraphMap::from_edges(inside_edges);

        let osm_g = OSMGraph {
            graph: child_graph,
            osm: self.osm.clone(),
        };

        Ok(osm_g)
    }
}

// TODO: needs proper builder pattern to allow construction for part graph
impl GUtils for OSMGraph {
    fn new(osm_graph: GI) -> Result<OSMGraph> {
        let e_lst: Vec<(OSMID, OSMID, f64)> = osm_graph
            .edges
            .iter()
            .map(|edge| (edge.from, edge.to, edge.length))
            .collect::<Vec<(OSMID, OSMID, f64)>>();

        let digraphmap: petgraph::prelude::GraphMap<OSMID, f64, Directed> =
            DiGraphMap::from_edges(&e_lst);

        Ok(Self {
            graph: digraphmap,
            osm: osm_graph,
        })
    }

    fn get_graph(&self) -> &petgraph::prelude::GraphMap<OSMID, f64, Directed> {
        &self.graph
    }

    fn get_vertices(&self) -> Vec<Vertex> {
        self.osm.vertices.clone()
    }

    fn get_edges(&self) -> Vec<Edge> {
        self.osm.edges.clone()
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
