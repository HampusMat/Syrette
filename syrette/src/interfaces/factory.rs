use crate::libs::intertrait::CastFrom;

pub trait IFactory<Args, Return>: Fn<Args, Output = Box<Return>> + CastFrom
where
    Return: 'static + ?Sized,
{
}
