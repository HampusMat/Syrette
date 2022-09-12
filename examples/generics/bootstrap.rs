use std::rc::Rc;

use syrette::{di_container_bind, DIContainer};

use crate::interfaces::printer::IPrinter;
use crate::printer::Printer;

pub fn bootstrap() -> Rc<DIContainer>
{
    let mut di_container = DIContainer::new();

    di_container_bind!(IPrinter<String> => Printer, di_container);
    di_container_bind!(IPrinter<i32> => Printer, di_container);

    di_container
}
