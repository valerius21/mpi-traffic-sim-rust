use petgraph::{algo::astar, prelude::GraphMap, Directed};

pub(crate) mod graph;
pub(crate) mod rect;

pub fn get_path_length<'a>(from: usize, to: usize, graph: GraphMap<usize, f64, Directed>) -> f64 {
    let path = astar(&graph, from, |finish| finish == to, |e| e.2.clone(), |_| 0.);

    match path {
        Some(p) => p.0,
        None => 0.,
    }
}
