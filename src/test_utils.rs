pub mod subjects
{
    //! Test subjects.

    use std::fmt::Debug;
    use std::rc::Rc;

    use syrette_macros::declare_interface;

    use crate::interfaces::injectable::Injectable;
    use crate::ptr::TransientPtr;
    use crate::DIContainer;

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

    declare_interface!(UserManager -> IUserManager);

    impl Injectable for UserManager
    {
        fn resolve(
            _di_container: &Rc<DIContainer>,
            _dependency_history: Vec<&'static str>,
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

    impl Injectable for Number
    {
        fn resolve(
            _di_container: &Rc<DIContainer>,
            _dependency_history: Vec<&'static str>,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }
    }
}

#[cfg(feature = "async")]
pub mod subjects_async
{
    //! Test subjects.

    use std::fmt::Debug;
    use std::sync::Arc;

    use async_trait::async_trait;
    use syrette_macros::declare_interface;

    use crate::interfaces::async_injectable::AsyncInjectable;
    use crate::ptr::TransientPtr;
    use crate::AsyncDIContainer;

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

    declare_interface!(UserManager -> IUserManager);

    #[async_trait]
    impl AsyncInjectable for UserManager
    {
        async fn resolve(
            _: &Arc<AsyncDIContainer>,
            _dependency_history: Vec<&'static str>,
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

    declare_interface!(Number -> INumber, async = true);

    #[async_trait]
    impl AsyncInjectable for Number
    {
        async fn resolve(
            _: &Arc<AsyncDIContainer>,
            _dependency_history: Vec<&'static str>,
        ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
        where
            Self: Sized,
        {
            Ok(TransientPtr::new(Self::new()))
        }
    }
}
