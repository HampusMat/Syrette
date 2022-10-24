use std::any::TypeId;

use ahash::AHashMap;

pub struct DIContainerBindingStorage<Provider>
where
    Provider: 'static + ?Sized,
{
    inner: AHashMap<BindingIdentification, Box<Provider>>,
}

impl<Provider> DIContainerBindingStorage<Provider>
where
    Provider: 'static + ?Sized,
{
    pub fn new() -> Self
    {
        Self {
            inner: AHashMap::new(),
        }
    }

    #[allow(clippy::borrowed_box)]
    pub fn get<Interface>(&self, name: Option<&'static str>) -> Option<&Box<Provider>>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self.inner.get(&BindingIdentification {
            type_id: interface_typeid,
            name,
        })
    }

    pub fn set<Interface>(&mut self, name: Option<&'static str>, provider: Box<Provider>)
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self.inner.insert(
            BindingIdentification {
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

        self.inner.remove(&BindingIdentification {
            type_id: interface_typeid,
            name,
        })
    }

    pub fn has<Interface>(&self, name: Option<&'static str>) -> bool
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self.inner.contains_key(&BindingIdentification {
            type_id: interface_typeid,
            name,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct BindingIdentification
{
    type_id: TypeId,
    name: Option<&'static str>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    mod subjects
    {
        pub trait SomeProvider
        {
            fn get_id(&self) -> u8;
        }

        pub struct SomeProviderImpl
        {
            pub id: u8,
        }

        impl SomeProvider for SomeProviderImpl
        {
            fn get_id(&self) -> u8
            {
                self.id
            }
        }
    }

    #[test]
    fn can_get()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.inner.insert(
            BindingIdentification {
                type_id: TypeId::of::<Interface>(),
                name: None,
            },
            Box::new(subjects::SomeProviderImpl { id: 20 }),
        );

        assert!(binding_map
            .get::<Interface>(None)
            .map_or_else(|| false, |provider| provider.get_id() == 20));
    }

    #[test]
    fn can_get_with_name()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.inner.insert(
            BindingIdentification {
                type_id: TypeId::of::<Interface>(),
                name: Some("hello"),
            },
            Box::new(subjects::SomeProviderImpl { id: 11 }),
        );

        assert!(binding_map
            .get::<Interface>(Some("hello"))
            .map_or_else(|| false, |provider| provider.get_id() == 11));

        assert!(binding_map.get::<Interface>(None).is_none());
    }

    #[test]
    fn can_set()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map
            .set::<Interface>(None, Box::new(subjects::SomeProviderImpl { id: 65 }));

        let expected_key = &BindingIdentification {
            type_id: TypeId::of::<Interface>(),
            name: None,
        };

        assert!(binding_map.inner.contains_key(expected_key));

        assert_eq!(binding_map.inner[expected_key].get_id(), 65);
    }

    #[test]
    fn can_set_with_name()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.set::<Interface>(
            Some("special"),
            Box::new(subjects::SomeProviderImpl { id: 3 }),
        );

        let expected_key = &BindingIdentification {
            type_id: TypeId::of::<Interface>(),
            name: Some("special"),
        };

        assert!(binding_map.inner.contains_key(expected_key));

        assert_eq!(binding_map.inner[expected_key].get_id(), 3);
    }

    #[test]
    fn can_remove()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.inner.insert(
            BindingIdentification {
                type_id: TypeId::of::<Interface>(),
                name: None,
            },
            Box::new(subjects::SomeProviderImpl { id: 103 }),
        );

        binding_map.remove::<Interface>(None);

        let expected_key = &BindingIdentification {
            type_id: TypeId::of::<Interface>(),
            name: None,
        };

        assert!(!binding_map.inner.contains_key(expected_key));
    }

    #[test]
    fn can_remove_with_name()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.inner.insert(
            BindingIdentification {
                type_id: TypeId::of::<Interface>(),
                name: Some("cool"),
            },
            Box::new(subjects::SomeProviderImpl { id: 42 }),
        );

        binding_map.remove::<Interface>(Some("cool"));

        let expected_key = &BindingIdentification {
            type_id: TypeId::of::<Interface>(),
            name: Some("cool"),
        };

        assert!(!binding_map.inner.contains_key(expected_key));
    }

    #[test]
    fn can_get_has()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        assert!(!binding_map.has::<Interface>(None));

        binding_map.inner.insert(
            BindingIdentification {
                type_id: TypeId::of::<Interface>(),
                name: None,
            },
            Box::new(subjects::SomeProviderImpl { id: 103 }),
        );

        assert!(binding_map.has::<Interface>(None));
    }

    #[test]
    fn can_get_has_with_name()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        assert!(!binding_map.has::<Interface>(Some("awesome")));

        binding_map.inner.insert(
            BindingIdentification {
                type_id: TypeId::of::<Interface>(),
                name: Some("awesome"),
            },
            Box::new(subjects::SomeProviderImpl { id: 101 }),
        );

        assert!(binding_map.has::<Interface>(Some("awesome")));
    }
}
