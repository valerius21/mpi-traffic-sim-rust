use std::collections::HashMap;

use crate::models::graph_input::{Graph as GI, Vertex};
use petgraph::{csr::NodeIndex, stable_graph::DefaultIx, Directed, Graph};

#[derive(Debug, Clone)]
pub struct OSMGraph {
    pub graph: Graph<(), (), Directed, usize>,
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

        println!("Creating graph with {} edges", e_lst.len());

        let mut graph =
            Graph::<(), (), Directed, usize>::with_capacity(osm_graph.vertices.len(), e_lst.len());

        let mut map_osm_id_node_idx = HashMap::<usize, NodeIndex<usize>>::new();

        osm_graph.vertices.iter().for_each(|vertex| {
            let node = graph.add_node(());
            map_osm_id_node_idx.insert(vertex.osm_id, node.index());
        });

        println!("Created graph with {} nodes", graph.node_count());

        // e_lst.iter().for_each(|edge| {
        //     let src = *map_osm_id_node_idx.get(&edge.0).unwrap();
        //     let dst = *map_osm_id_node_idx.get(&edge.1).unwrap();
        //     let n_src: NodeIndex<usize> = NodeIndex::new(src);
        // });

        OSMGraph { graph }
    }
}
