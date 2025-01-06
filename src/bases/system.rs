use super::Accessor;

pub trait Fallback: Accessor {
    fn fallback(&self, message: Self::Message, delta: Option<f32>);
}

pub trait System: Fallback {
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        match message {
            message => self.fallback(message, delta)
        }        
    }
}