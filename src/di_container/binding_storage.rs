use std::any::TypeId;

use ahash::AHashMap;

use crate::di_container::BindingOptions;

pub struct DIContainerBindingStorage<Provider>
where
    Provider: 'static + ?Sized,
{
    inner: AHashMap<BindingId<'static>, Box<Provider>>,
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
    pub fn get<'this, Interface>(
        &'this self,
        options: BindingOptions<'this>,
    ) -> Option<&'this Box<Provider>>
    where
        Interface: 'static + ?Sized,
    {
        self.inner.get(&BindingId::new::<Interface>(options))
    }

    pub fn set<Interface>(
        &mut self,
        options: BindingOptions<'static>,
        provider: Box<Provider>,
    ) where
        Interface: 'static + ?Sized,
    {
        self.inner
            .insert(BindingId::new::<Interface>(options), provider);
    }

    pub fn remove<Interface>(
        &mut self,
        options: BindingOptions<'static>,
    ) -> Option<Box<Provider>>
    where
        Interface: 'static + ?Sized,
    {
        self.inner.remove(&BindingId::new::<Interface>(options))
    }

    pub fn has<Interface>(&self, options: BindingOptions) -> bool
    where
        Interface: 'static + ?Sized,
    {
        self.inner
            .contains_key(&BindingId::new::<Interface>(options))
    }
}

impl<Provider> Default for DIContainerBindingStorage<Provider>
where
    Provider: 'static + ?Sized,
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct BindingId<'opts>
{
    type_id: TypeId,
    options: BindingOptions<'opts>,
}

impl<'opts> BindingId<'opts>
{
    fn new<Interface>(options: BindingOptions<'opts>) -> Self
    where
        Interface: ?Sized + 'static,
    {
        Self {
            type_id: TypeId::of::<Interface>(),
            options,
        }
    }
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
            BindingId::new::<Interface>(BindingOptions::new()),
            Box::new(subjects::SomeProviderImpl { id: 20 }),
        );

        assert!(binding_map
            .get::<Interface>(BindingOptions::new())
            .map_or_else(|| false, |provider| provider.get_id() == 20));
    }

    #[test]
    fn can_get_with_name()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.inner.insert(
            BindingId::new::<Interface>(BindingOptions::new().name("hello")),
            Box::new(subjects::SomeProviderImpl { id: 11 }),
        );

        assert!(binding_map
            .get::<Interface>(BindingOptions::new().name("hello"))
            .map_or_else(|| false, |provider| provider.get_id() == 11));

        assert!(binding_map
            .get::<Interface>(BindingOptions::new())
            .is_none());
    }

    #[test]
    fn can_set()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.set::<Interface>(
            BindingOptions::new(),
            Box::new(subjects::SomeProviderImpl { id: 65 }),
        );

        let expected_key = BindingId::new::<Interface>(BindingOptions::new());

        assert!(binding_map.inner.contains_key(&expected_key));

        assert_eq!(binding_map.inner[&expected_key].get_id(), 65);
    }

    #[test]
    fn can_set_with_name()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.set::<Interface>(
            BindingOptions::new().name("special"),
            Box::new(subjects::SomeProviderImpl { id: 3 }),
        );

        let expected_key =
            BindingId::new::<Interface>(BindingOptions::new().name("special"));

        assert!(binding_map.inner.contains_key(&expected_key));

        assert_eq!(binding_map.inner[&expected_key].get_id(), 3);
    }

    #[test]
    fn can_remove()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.inner.insert(
            BindingId::new::<Interface>(BindingOptions::new()),
            Box::new(subjects::SomeProviderImpl { id: 103 }),
        );

        binding_map.remove::<Interface>(BindingOptions::new());

        assert!(!binding_map
            .inner
            .contains_key(&BindingId::new::<Interface>(BindingOptions::new())));
    }

    #[test]
    fn can_remove_with_name()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        binding_map.inner.insert(
            BindingId::new::<Interface>(BindingOptions::new().name("cool")),
            Box::new(subjects::SomeProviderImpl { id: 42 }),
        );

        binding_map.remove::<Interface>(BindingOptions::new().name("cool"));

        assert!(
            !binding_map.inner.contains_key(&BindingId::new::<Interface>(
                BindingOptions::new().name("cool")
            ))
        );
    }

    #[test]
    fn can_get_has()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        assert!(!binding_map.has::<Interface>(BindingOptions::new()));

        binding_map.inner.insert(
            BindingId::new::<Interface>(BindingOptions::new()),
            Box::new(subjects::SomeProviderImpl { id: 103 }),
        );

        assert!(binding_map.has::<Interface>(BindingOptions::new()));
    }

    #[test]
    fn can_get_has_with_name()
    {
        type Interface = ();

        let mut binding_map =
            DIContainerBindingStorage::<dyn subjects::SomeProvider>::new();

        assert!(!binding_map.has::<Interface>(BindingOptions::new().name("awesome")));

        binding_map.inner.insert(
            BindingId::new::<Interface>(BindingOptions::new().name("awesome")),
            Box::new(subjects::SomeProviderImpl { id: 101 }),
        );

        assert!(binding_map.has::<Interface>(BindingOptions::new().name("awesome")));
    }
}
