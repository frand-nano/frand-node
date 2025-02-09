use crate::ext::*;

pub trait Fallback: State {
    fn fallback<CS: System>(
        node: Self::Node<'_, CS>, 
        message: Self::Message, 
        delta: Option<std::time::Duration>,
    );
}

pub trait System: Fallback {
    fn handle<CS: System>(
        node: Self::Node<'_, CS>, 
        message: Self::Message, 
        delta: Option<std::time::Duration>,
    ) {
        match message {
            message => Self::fallback(node, message, delta),
        }        
    }
}