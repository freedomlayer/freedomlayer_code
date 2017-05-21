
use std::collections::{HashSet};
use chord::{RingKey, NodeChain};

/// A chains array. Used for quick searching.
pub struct ChainsArray {
    raw_chains: Vec<NodeChain>,
    chains_set: HashSet<NodeChain>,
    sorted_chains_left: Vec<NodeChain>,
    sorted_chains_right: Vec<NodeChain>,
    is_indexed: bool,
}


/// Checksum the contents of a chain
fn csum_chain(chain: &NodeChain) -> u64 {
    chain.iter().fold(0, |acc, &x| acc.wrapping_add(x))
}


fn left_search_key(chain: &NodeChain) -> (i64, usize, u64) {
    (-(chain[0] as i64), chain.len(), csum_chain(chain))
}

fn right_search_key(chain: &NodeChain) -> (i64, usize, u64) {
    (chain[0] as i64, chain.len(), csum_chain(chain))
}

impl ChainsArray {
    pub fn new() -> ChainsArray {
        ChainsArray {
            raw_chains: Vec::new(),
            chains_set: HashSet::new(),
            sorted_chains_left: Vec::new(),
            sorted_chains_right: Vec::new(),
            is_indexed: false,
        }
    }

    /// Insert a new chain into the chains array.
    pub fn insert_chain(&mut self, chain: NodeChain) {
        assert!(!self.is_indexed, "Already indexed, aborting!");
        if self.chains_set.contains(&chain) {
            return
        }
        self.raw_chains.push(chain);
    }


    /// Index all the chains, for quick searching.
    /// This could be slow.
    pub fn index(&mut self) {
        assert!(!self.is_indexed, "Already indexed, aborting!");
        assert!(self.raw_chains.len() > 0, "Chains array is empty, aborting!");

        self.sorted_chains_left = self.raw_chains.clone();
        self.sorted_chains_right = self.raw_chains.clone();

        self.sorted_chains_left.sort_by_key(left_search_key);
        self.sorted_chains_right.sort_by_key(right_search_key);

        // Possibly clean up structures here to save some memory.

        self.is_indexed = true;
    }


    pub fn find_closest_left(&self, target_id: RingKey) -> &NodeChain {
        assert!(self.is_indexed, "Indexing is required before find_closest_left invocation!");
        let found_index = match self.sorted_chains_left.binary_search_by_key(&(-(target_id as i64), 0, 0), left_search_key) {
            Ok(index) => index,
            Err(index) => index,
        };
        &self.sorted_chains_left[found_index % self.sorted_chains_left.len()]
    }

    pub fn find_closest_right(&self, target_id: RingKey) -> &NodeChain {
        assert!(self.is_indexed, "Indexing is required before find_closest_right invocation!");

        let found_index = match self.sorted_chains_right.binary_search_by_key(&(target_id as i64, 0, 0), right_search_key) {
            Ok(index) => index,
            Err(index) => index,
        };
        &self.sorted_chains_right[found_index % self.sorted_chains_right.len()]
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csum_chain() {
        assert!(csum_chain(&vec![1,2,3,4]) == 10);
        assert!(csum_chain(&vec![]) == 0);
        assert!(csum_chain(&vec![1]) == 1);
    }

    #[test]
    fn test_chains_array() {
        let mut chains_array = ChainsArray::new();
        chains_array.insert_chain(vec![3,9,8,1]);
        chains_array.insert_chain(vec![4,3,2,1]);
        chains_array.insert_chain(vec![5,3,2,1]);
        chains_array.insert_chain(vec![5,3,17,2,1]);
        chains_array.insert_chain(vec![6,9,8,1]);
        chains_array.insert_chain(vec![7,5,3,1]);
        chains_array.insert_chain(vec![11,5,3,1]);
        chains_array.insert_chain(vec![18,5,3,1]);
        chains_array.insert_chain(vec![25,5,3,1]);
        chains_array.index();

        // Check exact match:
        assert!(*chains_array.find_closest_left(7) == vec![7,5,3,1]);
        assert!(*chains_array.find_closest_right(7) == vec![7,5,3,1]);

        assert!(*chains_array.find_closest_left(11) == vec![11,5,3,1]);
        assert!(*chains_array.find_closest_right(11) == vec![11,5,3,1]);

        // Check near match:
        assert!(*chains_array.find_closest_left(12) == vec![11,5,3,1]);
        assert!(*chains_array.find_closest_right(12) == vec![18,5,3,1]);

        // Check wraparound:
        assert!(*chains_array.find_closest_right(26) == vec![3,9,8,1]);
        assert!(*chains_array.find_closest_left(2) == vec![25,5,3,1]);

        // Prefer shorter chains:
        assert!(*chains_array.find_closest_right(5) == vec![5,3,2,1]);
        assert!(*chains_array.find_closest_left(5) == vec![5,3,2,1]);
    }
}
