
use std::collections::{HashSet};
use chord::{RingKey, SemiChain, SemiRoute};

/// A chains array. Used for quick searching.
pub struct SemiRoutesArray {
    sroutes: Vec<SemiRoute>,
    sroutes_set: HashSet<SemiRoute>,
    is_indexed: bool,
}

/// Get the final id of the last SemiChain in a SemiRoute.
pub fn sroute_final_id(sroute: &SemiRoute) -> RingKey {
    sroute[sroute.len() - 1].final_id
}


impl SemiRoutesArray {
    pub fn new() -> SemiRoutesArray {
        SemiRoutesArray {
            sroutes: Vec::new(),
            sroutes_set: HashSet::new(),
            is_indexed: false,
        }
    }

    /// Insert a new semi route into the semi routes array.
    pub fn insert_sroute(&mut self, sroute: SemiRoute) {
        assert!(!self.is_indexed, "Already indexed, aborting!");
        if self.sroutes_set.contains(&sroute) {
            return
        }
        self.sroutes_set.insert(sroute.clone());
        self.sroutes.push(sroute);
    }


    /// Index all the semi routes, for quick searching.
    /// This could be slow.
    pub fn index(&mut self) {
        assert!(!self.is_indexed, "Already indexed, aborting!");
        assert!(self.sroutes.len() > 0, "Chains array is empty, aborting!");

        self.sroutes.sort_by_key(sroute_final_id);
        self.is_indexed = true;
    }

    pub fn find_closest_left(&self, target_id: RingKey) -> &SemiRoute {
        assert!(self.is_indexed, "Indexing is required before find_closest_right invocation!");

        let found_index = match self.sroutes.binary_search_by_key(&target_id, sroute_final_id) {
            Ok(index) => index,
            Err(index) => (index + self.sroutes.len() - 1) % self.sroutes.len(),
        };
        &self.sroutes[found_index % self.sroutes.len()]
    }

    pub fn find_closest_right(&self, target_id: RingKey) -> &SemiRoute {
        assert!(self.is_indexed, "Indexing is required before find_closest_left invocation!");
        let found_index = match self.sroutes.binary_search_by_key(&target_id, sroute_final_id) {
            Ok(index) => index,
            Err(index) => index,
        };
        &self.sroutes[found_index % self.sroutes.len()]
    }

}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_semi_routes_array() {
        let mut semi_routes_array = SemiRoutesArray::new();
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 3, length: 4}]);
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 4, length: 4}]);
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 5, length: 4}]);
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 5, length: 5}]);
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 6, length: 5}]);
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 7, length: 4}]);
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 11, length: 4}]);
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 18, length: 4}]);
        semi_routes_array.insert_sroute(vec![SemiChain{final_id: 25, length: 4}]);
        semi_routes_array.index();

        // Check exact match:
        assert!(*semi_routes_array.find_closest_left(7) == vec![SemiChain{final_id: 7, length: 4}]);
        assert!(*semi_routes_array.find_closest_right(7) == vec![SemiChain{final_id: 7, length: 4}]);

        assert!(*semi_routes_array.find_closest_left(11) == vec![SemiChain{final_id: 11, length: 4}]);
        assert!(*semi_routes_array.find_closest_right(11) == vec![SemiChain{final_id: 11, length: 4}]);

        // Check near match:
        assert!(*semi_routes_array.find_closest_left(12) == vec![SemiChain{final_id: 11, length: 4}]);
        assert!(*semi_routes_array.find_closest_right(12) == vec![SemiChain{final_id: 18, length: 4}]);

        // Check wraparound:
        assert!(*semi_routes_array.find_closest_right(26) == vec![SemiChain{final_id: 3, length: 4}]);
        assert!(*semi_routes_array.find_closest_left(2) == vec![SemiChain{final_id: 25, length: 4}]);

        // Prefer shorter chains:
        assert!(*semi_routes_array.find_closest_right(5) == vec![SemiChain{final_id: 5, length: 4}]);
        assert!(*semi_routes_array.find_closest_left(5) == vec![SemiChain{final_id: 5, length: 4}]);
    }
}
