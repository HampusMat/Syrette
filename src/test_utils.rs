pub mod subjects
{
    //! Test subjects.

    use std::fmt::Debug;
    use std::rc::Rc;

    use syrette_macros::declare_interface;

    use crate::di_container::blocking::IDIContainer;
    use crate::interfaces::injectable::Injectable;
    use crate::private::cast::CastFromArc;
    use crate::ptr::TransientPtr;

    use_double!(crate::dependency_history::DependencyHistory);

    pub trait IUserManager
    {
        fn add_user(&self, user_id: i128);

        fn remove_user(&self, user_id: i128);
    }

    pub struct UserManager {}

    impl UserManager
    {
        pub fn new() -> Self
        {
            Self {}
        }
    }

    impl IUserManager for UserManager
    {
        fn add_user(&self, _user_id: i128)
        {
            // ...
        }

        fn remove_user(&self, _user_id: i128)
        {
            // ...
        }
    }

    use crate as syrette;
    use crate::util::use_double;

    declare_interface!(UserManager -> IUserManager);

    impl<DIContainerType> Injectable<DIContainerType> for UserManager
    where
        DIContainerType: IDIContainer,
    {
        fn resolve(
            _di_container: &Rc<DIContainerType>,
            _dependency_history: DependencyHistory,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }
    }

    pub trait INumber
    {
        fn get(&self) -> i32;

        fn set(&mut self, number: i32);
    }

    impl PartialEq for dyn INumber
    {
        fn eq(&self, other: &Self) -> bool
        {
            self.get() == other.get()
        }
    }

    impl Debug for dyn INumber
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            f.write_str(format!("{}", self.get()).as_str())
        }
    }

    pub struct Number
    {
        pub num: i32,
    }

    impl Number
    {
        pub fn new() -> Self
        {
            Self { num: 0 }
        }
    }

    impl INumber for Number
    {
        fn get(&self) -> i32
        {
            self.num
        }

        fn set(&mut self, number: i32)
        {
            self.num = number;
        }
    }

    declare_interface!(Number -> INumber);

    impl<DIContainerType> Injectable<DIContainerType> for Number
    where
        DIContainerType: IDIContainer,
    {
        fn resolve(
            _di_container: &Rc<DIContainerType>,
            _dependency_history: DependencyHistory,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }
    }

    #[derive(Debug)]
    pub struct Ninja;

    pub trait INinja: CastFromArc {}

    impl INinja for Ninja {}
}

#[cfg(feature = "async")]
pub mod subjects_async
{
    //! Test subjects.

    use std::fmt::Debug;
    use std::sync::Arc;

    use async_trait::async_trait;
    use syrette_macros::declare_interface;

    use crate::di_container::asynchronous::IAsyncDIContainer;
    use crate::interfaces::async_injectable::AsyncInjectable;
    use crate::ptr::TransientPtr;

    use_double!(crate::dependency_history::DependencyHistory);

    pub trait IUserManager: Send + Sync
    {
        fn add_user(&self, user_id: i128);

        fn remove_user(&self, user_id: i128);
    }

    pub struct UserManager {}

    impl UserManager
    {
        pub fn new() -> Self
        {
            Self {}
        }
    }

    impl IUserManager for UserManager
    {
        fn add_user(&self, _user_id: i128)
        {
            // ...
        }

        fn remove_user(&self, _user_id: i128)
        {
            // ...
        }
    }

    use crate as syrette;
    use crate::util::use_double;

    declare_interface!(UserManager -> IUserManager);

    #[async_trait]
    impl<DIContainerType> AsyncInjectable<DIContainerType> for UserManager
    where
        DIContainerType: IAsyncDIContainer,
    {
        async fn resolve(
            _: &Arc<DIContainerType>,
            _dependency_history: DependencyHistory,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }
    }

    pub trait INumber: Send + Sync
    {
        fn get(&self) -> i32;

        fn set(&mut self, number: i32);
    }

    impl PartialEq for dyn INumber
    {
        fn eq(&self, other: &Self) -> bool
        {
            self.get() == other.get()
        }
    }

    impl Debug for dyn INumber
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            f.write_str(format!("{}", self.get()).as_str())
        }
    }

    pub struct Number
    {
        pub num: i32,
    }

    impl Number
    {
        pub fn new() -> Self
        {
            Self { num: 0 }
        }
    }

    impl INumber for Number
    {
        fn get(&self) -> i32
        {
            self.num
        }

        fn set(&mut self, number: i32)
        {
            self.num = number;
        }
    }

    declare_interface!(Number -> INumber, threadsafe_sharable = true);

    #[async_trait]
    impl<DIContainerType> AsyncInjectable<DIContainerType> for Number
    where
        DIContainerType: IAsyncDIContainer,
    {
        async fn resolve(
            _: &Arc<DIContainerType>,
            _dependency_history: DependencyHistory,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }
    }
}

pub mod mocks
{
    #![allow(clippy::ref_option_ref)] // Caused by Mockall
    #![allow(dead_code)] // Not all mock functions may be used

    use mockall::mock;

    pub mod blocking_di_container
    {
        use std::rc::Rc;

        use super::*;
        use crate::di_container::blocking::binding::builder::BindingBuilder;
        use crate::di_container::blocking::details::DIContainerInternals;
        use crate::di_container::blocking::{BindingOptionsWithLt, IDIContainer};
        use crate::errors::di_container::DIContainerError;
        use crate::provider::blocking::IProvider;
        use crate::ptr::SomePtr;
        use crate::util::use_double;

        use_double!(crate::dependency_history::DependencyHistory);

        mock! {
            pub DIContainer {}

            impl IDIContainer for DIContainer
            {
                fn bind<Interface>(self: &mut Rc<Self>) -> BindingBuilder<Interface, Self>
                where
                    Interface: 'static + ?Sized;

                fn get<Interface>(self: &Rc<Self>) -> Result<SomePtr<Interface>, DIContainerError>
                where
                    Interface: 'static + ?Sized;

                fn get_named<Interface>(
                    self: &Rc<Self>,
                    name: &'static str,
                ) -> Result<SomePtr<Interface>, DIContainerError>
                where
                    Interface: 'static + ?Sized;

                #[doc(hidden)]
                fn get_bound<'opts, Interface>(
                    self: &Rc<Self>,
                    dependency_history: DependencyHistory,
                    binding_options: BindingOptionsWithLt,
                ) -> Result<SomePtr<Interface>, DIContainerError>
                where
                    Interface: 'static + ?Sized;
            }

            impl DIContainerInternals for DIContainer
            {
                fn has_binding<Interface>(self: &Rc<Self>, name: Option<&'static str>) -> bool
                where
                    Interface: ?Sized + 'static;

                #[doc(hidden)]
                fn set_binding<Interface>(
                    self: &Rc<Self>,
                    name: Option<&'static str>,
                    provider: Box<dyn IProvider<Self>>,
                ) where
                    Interface: 'static + ?Sized;

                fn remove_binding<Interface>(
                    self: &Rc<Self>,
                    name: Option<&'static str>,
                ) -> Option<Box<dyn IProvider<Self>>>
                where
                    Interface: 'static + ?Sized;
            }
        }
    }

    #[cfg(feature = "async")]
    pub mod async_di_container
    {
        use std::sync::Arc;

        use super::*;
        use crate::di_container::asynchronous::binding::builder::AsyncBindingBuilder;
        use crate::di_container::asynchronous::details::DIContainerInternals;
        use crate::di_container::asynchronous::IAsyncDIContainer;
        use crate::di_container::BindingOptions;
        use crate::errors::async_di_container::AsyncDIContainerError;
        use crate::provider::r#async::IAsyncProvider;
        use crate::ptr::SomePtr;
        use crate::util::use_double;

        use_double!(crate::dependency_history::DependencyHistory);

        mock! {
            pub AsyncDIContainer {}

            #[async_trait::async_trait]
            impl IAsyncDIContainer for AsyncDIContainer
            {
                fn bind<Interface>(self: &mut Arc<Self>) ->
                    AsyncBindingBuilder<Interface, Self>
                where
                    Interface: 'static + ?Sized + Send + Sync;

                async fn get<Interface>(
                    self: &Arc<Self>,
                ) -> Result<SomePtr<Interface>, AsyncDIContainerError>
                where
                    Interface: 'static + ?Sized + Send + Sync;

                async fn get_named<Interface>(
                    self: &Arc<Self>,
                    name: &'static str,
                ) -> Result<SomePtr<Interface>, AsyncDIContainerError>
                where
                    Interface: 'static + ?Sized + Send + Sync;

                #[doc(hidden)]
                async fn get_bound<Interface>(
                    self: &Arc<Self>,
                    dependency_history: DependencyHistory,
                    binding_options: BindingOptions<'static>
                ) -> Result<SomePtr<Interface>, AsyncDIContainerError>
                where
                    Interface: 'static + ?Sized + Send + Sync;
            }

            #[async_trait::async_trait]
            impl DIContainerInternals for AsyncDIContainer
            {
                async fn has_binding<Interface>(
                    self: &Arc<Self>,
                    name: Option<&'static str>,
                ) -> bool
                where
                    Interface: ?Sized + 'static;

                async fn set_binding<Interface>(
                    self: &Arc<Self>,
                    name: Option<&'static str>,
                    provider: Box<dyn IAsyncProvider<Self>>,
                ) where
                    Interface: 'static + ?Sized;

                async fn remove_binding<Interface>(
                    self: &Arc<Self>,
                    name: Option<&'static str>,
                ) -> Option<Box<dyn IAsyncProvider<Self>>>
                where
                    Interface: 'static + ?Sized;
            }
        }
    }

    #[cfg(feature = "async")]
    pub mod async_provider
    {
        use std::sync::Arc;

        use async_trait::async_trait;

        use super::*;
        use crate::di_container::asynchronous::IAsyncDIContainer;
        use crate::errors::injectable::InjectableError;
        use crate::provider::r#async::{AsyncProvidable, IAsyncProvider};
        use crate::util::use_double;

        use_double!(crate::dependency_history::DependencyHistory);

        mock! {
            pub AsyncProvider<DIContainerType: IAsyncDIContainer> {}

            #[async_trait]
            impl<
                DIContainerType: IAsyncDIContainer,
            > IAsyncProvider<DIContainerType> for AsyncProvider<
                DIContainerType,
            >
            {
                async fn provide(
                    &self,
                    di_container: &Arc<DIContainerType>,
                    dependency_history: DependencyHistory
                ) -> Result<AsyncProvidable<DIContainerType>, InjectableError>;

                fn do_clone(&self) ->
                    Box<dyn IAsyncProvider<DIContainerType>>;
            }
        }
    }

    pub mod blocking_provider
    {
        use std::rc::Rc;

        use super::*;
        use crate::di_container::blocking::IDIContainer;
        use crate::errors::injectable::InjectableError;
        use crate::provider::blocking::{IProvider, Providable};
        use crate::util::use_double;

        use_double!(crate::dependency_history::DependencyHistory);

        mock! {
            pub Provider<DIContainerType>
            where
                DIContainerType: IDIContainer
            {}

            impl<DIContainerType> IProvider<DIContainerType> for Provider<DIContainerType>
            where
                DIContainerType: IDIContainer,
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainerType>,
                    dependency_history: DependencyHistory,
                ) -> Result<Providable<DIContainerType>, InjectableError>;
            }
        }
    }
}

#[cfg(all(feature = "async", feature = "factory"))]
macro_rules! async_closure {
    (|$($args: ident),*| { $($inner: stmt);* }) => {
        Box::new(|$($args),*| {
            Box::pin(async move { $($inner)* })
        })
    };
    (|| { $($inner: stmt);* }) => {
        Box::new(|| {
            Box::pin(async move { $($inner)* })
        })
    };
}

#[cfg(all(feature = "async", feature = "factory"))]
pub(crate) use async_closure;
