use std::collections::HashSet;

use crate::{graph::rect::Point, models::graph_input::Graph as GI, prelude::Result};
use petgraph::{prelude::DiGraphMap, Directed};
use rayon::prelude::*;

use super::rect::Rect;

pub type Osmid = usize;

#[derive(Debug, Default, Clone)]
pub struct OSMGraph {
    osm: GI,
    pub graph: petgraph::prelude::GraphMap<Osmid, f64, Directed>,
}

// Graph Partitioning trait
pub trait GPartition {
    // n => number of partitions
    // i => index of partition
    // id => GraphID
    fn partition(&self, n: usize, i: usize) -> Result<OSMGraph>;
}

// Helper function to determine the rect for a given graph
fn determine_rects(target_graph: &OSMGraph, n: usize, i: usize) -> Result<OSMGraph> {
    let vtx_lst = target_graph.osm.vertices.clone();
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
    t_vrtx.retain(|x| target_rect.in_rect(*x));
    target_rect.vertices = t_vrtx;

    let verticies = target_rect
        .vertices
        .clone()
        .into_iter()
        .map(|v| v.osm_id)
        .collect::<HashSet<Osmid>>();
    let inside_edges = target_graph
        .graph
        .all_edges()
        .filter(|e: &(Osmid, Osmid, &f64)| verticies.contains(&e.0) && verticies.contains(&e.1));

    let child_graph: petgraph::prelude::GraphMap<Osmid, f64, Directed> =
        DiGraphMap::from_edges(inside_edges);

    let osm_g = OSMGraph {
        graph: child_graph,
        osm: target_graph.osm.clone(),
    };

    Ok(osm_g)
}

impl GPartition for OSMGraph {
    fn partition(&self, n: usize, i: usize) -> Result<OSMGraph> {
        // create all possible rects
        let mut graphs = Vec::<OSMGraph>::new();

        // create all possible rects
        for j in 0..n {
            let osm_g = determine_rects(self, n, j)?;
            graphs.push(osm_g);
        }

        // create a set of all vertices in all rects
        let mut vertex_set = HashSet::<Osmid>::new();
        for g in &graphs {
            let gg = g.graph.clone();
            let v = gg.nodes();
            vertex_set.extend(v);
        }

        // create a vector with the difference of all vertices and the set
        let mut diff = Vec::<Osmid>::new();
        let graph = self.graph.clone();
        for v in graph.nodes() {
            if !vertex_set.contains(&v) {
                diff.push(v);
            }
        }

        // Post-Partition Validation
        let mut all_nodes: HashSet<Osmid> = self.graph.nodes().collect();
        for g in &graphs {
            for node in g.graph.nodes() {
                all_nodes.remove(&node);
            }
        }

        // Handle Missing Nodes
        if !all_nodes.is_empty() {
            // For simplicity, adding missing nodes to the first partition
            // TODO: Adjust this to add to the nearest partition based on x-coordinate
            let first_partition = &mut graphs[0];
            for missing_node in all_nodes {
                first_partition.graph.add_node(missing_node);
            }
        }

        Ok(graphs[i].clone())
    }
}

impl OSMGraph {
    pub fn new(osm_graph: GI) -> Result<OSMGraph> {
        let e_lst: Vec<(Osmid, Osmid, f64)> = osm_graph
            .edges
            .par_iter()
            .map(|edge| (edge.from, edge.to, edge.length))
            .collect::<Vec<(Osmid, Osmid, f64)>>();

        let digraphmap: petgraph::prelude::GraphMap<Osmid, f64, Directed> =
            DiGraphMap::from_edges(&e_lst);

        Ok(Self {
            graph: digraphmap,
            osm: osm_graph,
        })
    }
}
