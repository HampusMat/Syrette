use std::collections::HashSet;
use std::hash::Hash;

/// [`Iterator`] extension trait.
pub trait IteratorExt<Item>
where
    Item: Eq + Hash,
{
    /// Finds the first occurance of a duplicate item.
    ///
    /// This function is short-circuiting. So it will immedietly return `Some` when
    /// it comes across a item it has already seen.
    ///
    /// The returned tuple contains the first item occurance & the second item occurance.
    /// In that specific order.
    ///
    /// Both items are returned in the case of the hash not being representative of the
    /// whole item.
    fn find_duplicate(self) -> Option<(Item, Item)>;
}

impl<Iter> IteratorExt<Iter::Item> for Iter
where
    Iter: Iterator,
    Iter::Item: Eq + Hash,
{
    fn find_duplicate(self) -> Option<(Iter::Item, Iter::Item)>
    {
        let mut iterated_item_map = HashSet::<Iter::Item>::new();

        for item in self {
            if let Some(equal_item) = iterated_item_map.take(&item) {
                return Some((item, equal_item));
            }

            iterated_item_map.insert(item);
        }

        None
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn can_find_dupe()
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
            Some((
                &Fruit {
                    name: "Apple".to_string()
                },
                &Fruit {
                    name: "Apple".to_string()
                }
            ))
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
