use std::{cell::RefCell, rc::Rc};
use crate::bases::Packet;

pub type Callback = Rc<RefCell<dyn FnMut(Packet)>>;