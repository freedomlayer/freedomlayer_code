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
        match index > self.index_id.len() {
            true => None,
            _ => Some(self.index_id[index])
        }
    }
    pub fn id_to_index(&self,id: RingKey) -> Option<usize> {
        self.id_index.get(&id).map(|&index| index)
    }

    pub fn add_node(&mut self, index: usize, id: RingKey) {
        let new_index:usize = self.index_id.len();
        self.index_id.push(id);
        self.id_index.insert(id,index);
    }
}
