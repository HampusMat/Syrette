use std::any::{type_name, TypeId};

use ahash::AHashMap;
use error_stack::report;

use crate::{errors::di_container::DIContainerError, provider::IProvider};

pub struct DIContainerBindingMap
{
    bindings: AHashMap<TypeId, Box<dyn IProvider>>,
}

impl DIContainerBindingMap
{
    pub fn new() -> Self
    {
        Self {
            bindings: AHashMap::new(),
        }
    }

    pub fn get<Interface>(&self) -> error_stack::Result<&dyn IProvider, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        Ok(self
            .bindings
            .get(&interface_typeid)
            .ok_or_else(|| {
                report!(DIContainerError).attach_printable(format!(
                    "No binding exists for interface '{}'",
                    type_name::<Interface>()
                ))
            })?
            .as_ref())
    }

    pub fn set<Interface>(&mut self, provider: Box<dyn IProvider>) -> Option<()>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        if self.bindings.contains_key(&interface_typeid) {
            return None;
        }

        self.bindings.insert(interface_typeid, provider);

        Some(())
    }

    /// Only used by tests in the `di_container` module.
    #[cfg(test)]
    pub fn count(&self) -> usize
    {
        self.bindings.len()
    }
}
