use std::collections::HashMap;
use std::hash::Hash;

pub trait IteratorExt<Item>
{
    fn find_duplicate(&mut self) -> Option<Item>;
}

impl<Iter> IteratorExt<Iter::Item> for Iter
where
    Iter: Iterator,
    Iter::Item: Eq + Hash + Clone,
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn can_find_duplicate()
    {
        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        struct Fruit
        {
            name: String,
        }

        assert_eq!(
            vec![
                Fruit {
                    name: "Apple".to_string(),
                },
                Fruit {
                    name: "Banana".to_string(),
                },
                Fruit {
                    name: "Apple".to_string(),
                },
                Fruit {
                    name: "Orange".to_string(),
                },
            ]
            .iter()
            .find_duplicate(),
            Some(&Fruit {
                name: "Apple".to_string()
            })
        );

        assert_eq!(
            vec![
                Fruit {
                    name: "Banana".to_string(),
                },
                Fruit {
                    name: "Apple".to_string(),
                },
                Fruit {
                    name: "Orange".to_string(),
                },
            ]
            .iter()
            .find_duplicate(),
            None
        );
    }
}
