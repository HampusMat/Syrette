use std::collections::HashMap;
use std::hash::Hash;

pub trait IteratorExt<Item>
{
    fn find_duplicate(&mut self) -> Option<Item>;
}

impl<Iter> IteratorExt<Iter::Item> for Iter
where
    Iter: Iterator,
    Iter::Item: Eq + Hash + Copy,
{
    fn find_duplicate(&mut self) -> Option<Iter::Item>
    {
        let mut iterated_item_map = HashMap::<Iter::Item, ()>::new();

        for item in self {
            if iterated_item_map.contains_key(&item) {
                return Some(item);
            }

            iterated_item_map.insert(item, ());
        }

        None
    }
}
