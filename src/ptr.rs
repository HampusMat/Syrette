#![allow(clippy::module_name_repetitions)]
use std::rc::Rc;

pub type InterfacePtr<Interface> = Box<Interface>;

pub type FactoryPtr<FactoryInterface> = Rc<FactoryInterface>;
