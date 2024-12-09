use super::{NodeId, NodeKey, Reporter};

pub trait NodeBase: 'static {
    fn key(&self) -> &NodeKey;
    fn reporter(&self) -> &Reporter;

    fn new(
        key: Vec<NodeId>,
        id: Option<NodeId>,
        reporter: &Reporter,
    ) -> Self;
}