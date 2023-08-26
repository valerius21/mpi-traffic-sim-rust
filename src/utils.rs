use crate::graph::graph::OSMGraph;

// * NOTE: use bincode https://github.com/bincode-org/bincode
pub(crate) trait MpiMessageContent<T> {
    fn to_bytes(data: T) -> Vec<u8>;
    fn from_bytes(data: Vec<u8>) -> T;
}

pub(crate) trait GraphReference {
    fn set_graph_ref(graph: &OSMGraph);
    fn get_graph_ref() -> &'static OSMGraph;
}
