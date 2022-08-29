pub trait ICat: Send + Sync
{
    fn meow(&self);
}
