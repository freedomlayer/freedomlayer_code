
use std::collections::{HashSet};
use chord::{RingKey, SemiChain};

/// A chains array. Used for quick searching.
pub struct SemiChainsArray {
    schains: Vec<SemiChain>,
    schains_set: HashSet<SemiChain>,
    is_indexed: bool,
}

/// Get the final id of the last SemiChain in a SemiRoute.
pub fn schain_final_id(schain: &SemiChain) -> RingKey {
    schain.final_id
}


impl SemiChainsArray {
    pub fn new() -> SemiChainsArray {
        SemiChainsArray {
            schains: Vec::new(),
            schains_set: HashSet::new(),
            is_indexed: false,
        }
    }

    /// Insert a new semi chain into the semi chains array.
    pub fn insert_schain(&mut self, schain: SemiChain) {
        assert!(!self.is_indexed, "Already indexed, aborting!");
        if self.schains_set.contains(&schain) {
            return
        }
        self.schains_set.insert(schain.clone());
        self.schains.push(schain);
    }


    /// Index all the semi chains, for quick searching.
    /// This could be slow.
    pub fn index(&mut self) {
        assert!(!self.is_indexed, "Already indexed, aborting!");
        assert!(self.schains.len() > 0, "Chains array is empty, aborting!");

        self.schains.sort_by_key(|schain| (schain.final_id, schain.length));
        self.is_indexed = true;
    }

    pub fn find_closest_left(&self, target_id: RingKey) -> &SemiChain {
        assert!(self.is_indexed, "Indexing is required before find_closest_right invocation!");

        let found_index = match self.schains.binary_search_by_key(&target_id, |schain| (schain.final_id)) {
            Ok(index) => index,
            Err(index) => (index + self.schains.len() - 1) % self.schains.len(),
        };
        &self.schains[found_index % self.schains.len()]
    }

    pub fn find_closest_right(&self, target_id: RingKey) -> &SemiChain {
        assert!(self.is_indexed, "Indexing is required before find_closest_left invocation!");
        let found_index = match self.schains.binary_search_by_key(&target_id, |schain| schain.final_id) {
            Ok(index) => index,
            Err(index) => index,
        };
        &self.schains[found_index % self.schains.len()]
    }

}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_semi_chains_array() {
        let mut semi_chains_array = SemiChainsArray::new();
        semi_chains_array.insert_schain(SemiChain{final_id: 3, length: 4});
        semi_chains_array.insert_schain(SemiChain{final_id: 4, length: 4});
        semi_chains_array.insert_schain(SemiChain{final_id: 5, length: 4});
        semi_chains_array.insert_schain(SemiChain{final_id: 5, length: 5});
        semi_chains_array.insert_schain(SemiChain{final_id: 6, length: 5});
        semi_chains_array.insert_schain(SemiChain{final_id: 7, length: 4});
        semi_chains_array.insert_schain(SemiChain{final_id: 11, length: 4});
        semi_chains_array.insert_schain(SemiChain{final_id: 18, length: 4});
        semi_chains_array.insert_schain(SemiChain{final_id: 25, length: 4});
        semi_chains_array.index();

        // Check exact match:
        assert!(*semi_chains_array.find_closest_left(7) == SemiChain{final_id: 7, length: 4});
        assert!(*semi_chains_array.find_closest_right(7) == SemiChain{final_id: 7, length: 4});

        assert!(*semi_chains_array.find_closest_left(11) == SemiChain{final_id: 11, length: 4});
        assert!(*semi_chains_array.find_closest_right(11) == SemiChain{final_id: 11, length: 4});

        // Check near match:
        assert!(*semi_chains_array.find_closest_left(12) == SemiChain{final_id: 11, length: 4});
        assert!(*semi_chains_array.find_closest_right(12) == SemiChain{final_id: 18, length: 4});

        // Check wraparound:
        assert!(*semi_chains_array.find_closest_right(26) == SemiChain{final_id: 3, length: 4});
        assert!(*semi_chains_array.find_closest_left(2) == SemiChain{final_id: 25, length: 4});

        // Prefer shorter chains:
        assert!(*semi_chains_array.find_closest_right(5) == SemiChain{final_id: 5, length: 4});
        assert!(*semi_chains_array.find_closest_left(5) == SemiChain{final_id: 5, length: 4});
    }
}
