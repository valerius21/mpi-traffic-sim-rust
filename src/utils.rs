use crate::graph::graph::OSMGraph;
use crate::prelude::Result;

// * NOTE: use bincode https://github.com/bincode-org/bincode
pub(crate) trait MpiMessageContent<T> {
    fn to_bytes(data: T) -> Result<Vec<u8>>;
    fn from_bytes(data: Vec<u8>) -> Result<T>;
}

pub(crate) trait GraphReference {
    fn set_graph_ref(graph: &OSMGraph);
    fn get_graph_ref() -> &'static OSMGraph;
}
