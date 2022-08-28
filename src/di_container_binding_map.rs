use std::any::{type_name, TypeId};

use ahash::AHashMap;

use crate::errors::di_container::DIContainerError;
use crate::provider::IProvider;

#[derive(Debug, PartialEq, Eq, Hash)]
struct DIContainerBindingKey
{
    type_id: TypeId,
    name: Option<&'static str>,
}

pub struct DIContainerBindingMap
{
    bindings: AHashMap<DIContainerBindingKey, Box<dyn IProvider>>,
}

impl DIContainerBindingMap
{
    pub fn new() -> Self
    {
        Self {
            bindings: AHashMap::new(),
        }
    }

    pub fn get<Interface>(
        &self,
        name: Option<&'static str>,
    ) -> Result<&dyn IProvider, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        Ok(self
            .bindings
            .get(&DIContainerBindingKey {
                type_id: interface_typeid,
                name,
            })
            .ok_or_else(|| DIContainerError::BindingNotFound {
                interface: type_name::<Interface>(),
                name,
            })?
            .as_ref())
    }

    pub fn set<Interface>(
        &mut self,
        name: Option<&'static str>,
        provider: Box<dyn IProvider>,
    ) where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self.bindings.insert(
            DIContainerBindingKey {
                type_id: interface_typeid,
                name,
            },
            provider,
        );
    }

    pub fn remove<Interface>(
        &mut self,
        name: Option<&'static str>,
    ) -> Option<Box<dyn IProvider>>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self.bindings.remove(&DIContainerBindingKey {
            type_id: interface_typeid,
            name,
        })
    }

    pub fn has<Interface>(&self, name: Option<&'static str>) -> bool
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self.bindings.contains_key(&DIContainerBindingKey {
            type_id: interface_typeid,
            name,
        })
    }

    /// Only used by tests in the `di_container` module.
    #[cfg(test)]
    pub fn count(&self) -> usize
    {
        self.bindings.len()
    }
}
