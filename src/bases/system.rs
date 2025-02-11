use crate::ext::*;

pub trait Fallback: State {
    fn fallback(
        node: Self::Node<'_>, 
        message: Self::Message, 
        delta: Option<std::time::Duration>,
    );
}

pub trait System: Fallback {
    fn handle(
        node: Self::Node<'_>, 
        message: Self::Message, 
        delta: Option<std::time::Duration>,
    ) {
        match message {
            message => Self::fallback(node, message, delta),
        }        
    }
}