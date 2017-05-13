use std::collections::{HashMap};
use chord::{RingKey};

/// A map between nodes and their IDs on the ring.
pub struct IndexId {
    index_id: Vec<RingKey>, // Index -> id
    id_index: HashMap<RingKey, usize>, // id -> Index
}

impl IndexId {
    pub fn new() -> Self {
        IndexId {
            // Two fast translation tables between index <--> id
            index_id: Vec::new(),
            id_index: HashMap::new(),
        }
    }
    pub fn index_to_id(&self,index: usize) -> Option<RingKey> {
        match index >= self.index_id.len() {
            true => None,
            _ => Some(self.index_id[index])
        }
    }
    pub fn id_to_index(&self,id: RingKey) -> Option<usize> {
        self.id_index.get(&id).map(|&index| index)
    }

    pub fn add_node(&mut self, id: RingKey) -> usize {
        let new_index:usize = self.index_id.len();
        self.index_id.push(id);
        self.id_index.insert(id,new_index);
        new_index
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_id() {
        let mut index_id = IndexId::new();
        index_id.add_node(2); // index: 0
        index_id.add_node(3); // index: 1
        assert!(index_id.index_to_id(0).unwrap() == 2);
        assert!(index_id.index_to_id(1).unwrap() == 3);
        assert!(index_id.index_to_id(2).is_none());
        assert!(index_id.index_to_id(3).is_none());
        assert!(index_id.index_to_id(4).is_none());

        assert!(index_id.id_to_index(2).unwrap() == 0);
        assert!(index_id.id_to_index(3).unwrap() == 1);
        assert!(index_id.id_to_index(4).is_none());
        assert!(index_id.id_to_index(0).is_none());
    }
}


