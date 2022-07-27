#![allow(clippy::module_name_repetitions)]
use std::rc::Rc;

pub type TransientPtr<Interface> = Box<Interface>;

pub type SingletonPtr<Interface> = Rc<Interface>;

pub type FactoryPtr<FactoryInterface> = Rc<FactoryInterface>;
