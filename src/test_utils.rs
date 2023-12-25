pub mod subjects
{
    //! Test subjects.

    use std::fmt::Debug;
    use std::rc::Rc;
    use std::sync::Arc;

    use crate::interfaces::injectable::Injectable;
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

    use crate::ptr_buffer::PtrBuffer;
    use crate::util::use_double;

    impl<DIContainerT> Injectable<DIContainerT> for UserManager
    {
        fn resolve(
            _di_container: &DIContainerT,
            _dependency_history: DependencyHistory,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }

        fn into_ptr_buffer_box(self: Box<Self>) -> PtrBuffer
        {
            let me: Box<dyn IUserManager> = self;

            PtrBuffer::new_from(me)
        }

        fn into_ptr_buffer_rc(self: Rc<Self>) -> PtrBuffer
        {
            let me: Rc<dyn IUserManager> = self;

            PtrBuffer::new_from(me)
        }

        fn into_ptr_buffer_arc(self: Arc<Self>) -> PtrBuffer
        {
            let me: Arc<dyn IUserManager> = self;

            PtrBuffer::new_from(me)
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

    impl<DIContainerT> Injectable<DIContainerT> for Number
    {
        fn resolve(
            _di_container: &DIContainerT,
            _dependency_history: DependencyHistory,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }

        fn into_ptr_buffer_box(self: Box<Self>) -> PtrBuffer
        {
            let me: Box<dyn INumber> = self;

            PtrBuffer::new_from(me)
        }

        fn into_ptr_buffer_rc(self: Rc<Self>) -> PtrBuffer
        {
            let me: Rc<dyn INumber> = self;

            PtrBuffer::new_from(me)
        }

        fn into_ptr_buffer_arc(self: Arc<Self>) -> PtrBuffer
        {
            let me: Arc<dyn INumber> = self;

            PtrBuffer::new_from(me)
        }
    }

    #[derive(Debug)]
    pub struct Ninja;

    pub trait INinja {}

    impl INinja for Ninja {}
}

#[cfg(feature = "async")]
pub mod subjects_async
{
    //! Test subjects.

    use std::fmt::Debug;
    use std::rc::Rc;
    use std::sync::Arc;

    use async_trait::async_trait;

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

    use crate::ptr_buffer::PtrBuffer;
    use crate::util::use_double;

    #[async_trait]
    impl<DIContainerType> AsyncInjectable<DIContainerType> for UserManager
    {
        async fn resolve(
            _: &DIContainerType,
            _dependency_history: DependencyHistory,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }

        fn into_ptr_buffer_box(self: Box<Self>) -> PtrBuffer
        {
            let me: Box<dyn IUserManager> = self;

            PtrBuffer::new_from(me)
        }

        fn into_ptr_buffer_rc(self: Rc<Self>) -> PtrBuffer
        {
            let me: Rc<dyn IUserManager> = self;

            PtrBuffer::new_from(me)
        }

        fn into_ptr_buffer_arc(self: Arc<Self>) -> PtrBuffer
        {
            let me: Arc<dyn IUserManager> = self;

            PtrBuffer::new_from(me)
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

    #[async_trait]
    impl<DIContainerType> AsyncInjectable<DIContainerType> for Number
    {
        async fn resolve(
            _: &DIContainerType,
            _dependency_history: DependencyHistory,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }

        fn into_ptr_buffer_box(self: Box<Self>) -> PtrBuffer
        {
            let me: Box<dyn INumber> = self;

            PtrBuffer::new_from(me)
        }

        fn into_ptr_buffer_rc(self: Rc<Self>) -> PtrBuffer
        {
            let me: Rc<dyn INumber> = self;

            PtrBuffer::new_from(me)
        }

        fn into_ptr_buffer_arc(self: Arc<Self>) -> PtrBuffer
        {
            let me: Arc<dyn INumber> = self;

            PtrBuffer::new_from(me)
        }
    }
}

pub mod mocks
{
    #![allow(clippy::ref_option_ref)] // Caused by Mockall
    #![allow(dead_code)] // Not all mock functions may be used

    #[cfg(feature = "async")]
    pub mod async_provider
    {
        use async_trait::async_trait;
        use mockall::mock;

        use crate::errors::injectable::InjectableError;
        use crate::provider::r#async::{AsyncProvidable, IAsyncProvider};
        use crate::util::use_double;

        use_double!(crate::dependency_history::DependencyHistory);

        mock! {
            pub AsyncProvider<DIContainerT> {}

            #[async_trait]
            impl<DIContainerT> IAsyncProvider<DIContainerT>
                for AsyncProvider<DIContainerT>
            where
                DIContainerT: Send + Sync
            {
                async fn provide(
                    &self,
                    di_container: &DIContainerT,
                    dependency_history: DependencyHistory
                ) -> Result<AsyncProvidable<DIContainerT>, InjectableError>;

                fn do_clone(&self) ->
                    Box<dyn IAsyncProvider<DIContainerT>>;
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
