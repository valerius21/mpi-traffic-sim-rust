use petgraph::{algo::astar, prelude::GraphMap, Directed};

pub mod osm_graph;
pub mod rect;

pub fn get_path_length(from: usize, to: usize, graph: GraphMap<usize, f64, Directed>) -> f64 {
    let path = astar(&graph, from, |finish| finish == to, |e| *e.2, |_| 0.);

    match path {
        Some(p) => p.0,
        None => 0.,
    }
}
