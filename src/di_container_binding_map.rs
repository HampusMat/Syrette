use std::any::TypeId;

use ahash::AHashMap;

#[derive(Debug, PartialEq, Eq, Hash)]
struct DIContainerBindingKey
{
    type_id: TypeId,
    name: Option<&'static str>,
}

pub struct DIContainerBindingMap<Provider>
where
    Provider: 'static + ?Sized,
{
    bindings: AHashMap<DIContainerBindingKey, Box<Provider>>,
}

impl<Provider> DIContainerBindingMap<Provider>
where
    Provider: 'static + ?Sized,
{
    pub fn new() -> Self
    {
        Self {
            bindings: AHashMap::new(),
        }
    }

    pub fn get<Interface>(&self, name: Option<&'static str>) -> Option<&Provider>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self.bindings
            .get(&DIContainerBindingKey {
                type_id: interface_typeid,
                name,
            })
            .map(|provider| provider.as_ref())
    }

    pub fn set<Interface>(&mut self, name: Option<&'static str>, provider: Box<Provider>)
    where
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
    ) -> Option<Box<Provider>>
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
