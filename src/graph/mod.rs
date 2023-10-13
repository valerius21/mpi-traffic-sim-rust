use petgraph::{algo::astar, prelude::GraphMap, Directed};

pub mod osm_graph;
pub mod rect;

// This function calculates the length of the shortest path between two nodes in a directed graph.
// Parameters:
// - `from`: The starting node of the path.
// - `to`: The target node of the path.
// - `graph`: A directed graph (GraphMap) represented as a map of nodes and edge weights.
// Returns:
// - A floating-point number representing the length of the shortest path from `from` to `to` in meters.
pub fn get_path_length(from: usize, to: usize, graph: GraphMap<usize, f64, Directed>) -> f64 {
    // Use the A* algorithm from the petgraph crate to find the shortest path.
    // - The `from` parameter specifies the starting node.
    // - The closure `|finish| finish == to` defines the goal condition for the path search.
    //   It stops when the current node equals the target node `to`.
    // - The closure `|e| *e.2` specifies the edge cost function, which extracts the weight of the edge.
    // - The closure `|_| 0.` sets the heuristic function to zero, meaning no heuristic is used.
    let path = astar(&graph, from, |finish| finish == to, |e| *e.2, |_| 0.);

    match path {
        // If a valid path is found, return its length.
        Some(p) => p.0,
        // If no path is found, return 0.0 as a default value.
        None => 0.,
    }
}
