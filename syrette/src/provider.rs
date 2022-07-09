use std::marker::PhantomData;

extern crate error_stack;

use crate::injectable::{Injectable, ResolveError};
use crate::DIContainer;

pub trait IInjectableTypeProvider
{
    fn provide(
        &self,
        di_container: &DIContainer,
    ) -> error_stack::Result<Box<dyn Injectable>, ResolveError>;
}

pub struct InjectableTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    _phantom_data: PhantomData<InjectableType>,
}

impl<InjectableType> InjectableTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    pub fn new() -> Self
    {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<InjectableType> IInjectableTypeProvider for InjectableTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    fn provide(
        &self,
        di_container: &DIContainer,
    ) -> error_stack::Result<Box<dyn Injectable>, ResolveError>
    {
        Ok(InjectableType::resolve(di_container)?)
    }
}
