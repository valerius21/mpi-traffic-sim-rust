use crate::models::graph_input::{Edge, Graph as GI, Vertex};
use petgraph::{Directed, Graph};
pub struct OSMGraph {
    pub graph: Graph<Vertex, Edge>,
}

pub trait GUtils {
    fn get_vertex_by_id(id: u64, osm_graph: &GI) -> Vertex;

    fn new(osm_graph: GI) -> OSMGraph;
}

impl GUtils for OSMGraph {
    fn get_vertex_by_id(id: u64, osm_graph: &GI) -> Vertex {
        osm_graph
            .vertices
            .iter()
            .find(|node| node.osm_id == id)
            .unwrap()
            .clone()
    }

    fn new(osm_graph: GI) -> OSMGraph {
        let mut graph = Graph::<Vertex, Edge, Directed>::new();
        osm_graph.edges.iter().for_each(|edge| {
            let from = OSMGraph::get_vertex_by_id(edge.from, &osm_graph);
            let to = OSMGraph::get_vertex_by_id(edge.to, &osm_graph);
            let source = graph.add_node(from);
            let target = graph.add_node(to);
            graph.update_edge(source, target, edge.clone());
        });

        OSMGraph { graph }
    }
}
