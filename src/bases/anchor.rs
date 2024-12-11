use super::{AnchorId, AnchorKey, Reporter};

pub trait Anchor: 'static + Clone {
    fn key(&self) -> &AnchorKey;
    fn reporter(&self) -> &Reporter;

    fn new(
        key: Vec<AnchorId>,
        id: Option<AnchorId>,
        reporter: &Reporter,
    ) -> Self;
}