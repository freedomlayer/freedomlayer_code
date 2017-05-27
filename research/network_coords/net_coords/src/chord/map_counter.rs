use chord::{SemiChain};
use std::collections::{HashMap};
use std::hash;
use std::fmt::{Debug};

pub struct MapCounter<T> {
    count_map: HashMap<T, usize>,
}

impl<T: Eq + hash::Hash + Clone + Debug> MapCounter<T> {
    pub fn new() -> MapCounter<T> {
        MapCounter {
            count_map: HashMap::new(),
        }
    }

    /// Report to the counting map the departue of final id removed_id.
    pub fn remove(&mut self, removed_item: &T) {
        assert!(self.count_map.contains_key(removed_item), "Item not found!");
        let result_count_value = {
            let removed_entry = self.count_map.entry(removed_item.clone()).or_insert(0);
            *removed_entry -= 1;
            *removed_entry
        };
        if result_count_value == 0 {
            self.count_map.remove(&removed_item);
        }
    }

    /// Report to the counting map the departue of final id removed_id.
    pub fn insert(&mut self, inserted_item: T) {
        *self.count_map.entry(inserted_item).or_insert(0) += 1;
    }

    /// Find out the count of a given id.
    /// If it doesn't exist, return 0.
    pub fn get_count(&self, item: &T) -> usize {
        match self.count_map.get(&item) {
            Some(&count) => count,
            None => 0,
        }
    }

    pub fn len(&self) -> usize {
        self.count_map.len()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_counter() {
        let mut map_counter = MapCounter::new();
        assert!(map_counter.len() == 0);
        map_counter.insert(3);
        assert!(map_counter.len() == 1);
        map_counter.insert(4);
        assert!(map_counter.len() == 2);
        map_counter.remove(&3);
        assert!(map_counter.len() == 1);
        map_counter.remove(&4);
        assert!(map_counter.len() == 0);

        // Check duplicity:
        map_counter.insert(5);
        assert!(map_counter.len() == 1);
        map_counter.insert(5);
        assert!(map_counter.len() == 1);
    }

    #[test]
    #[should_panic]
    fn test_map_counter_remove_nonexistent() {
        let mut map_counter = MapCounter::new();
        map_counter.remove(&4);
    }

    #[test]
    #[should_panic]
    fn test_map_counter_remove_nonexistent_after_removed() {
        let mut map_counter = MapCounter::new();
        map_counter.insert(5);
        map_counter.remove(&5);
        map_counter.remove(&5);
    }
}
