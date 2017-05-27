extern crate itertools;

use chord::{RingKey, vdist};
use std::collections::{HashSet, HashMap};

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct SemiChain {
    pub final_id: RingKey,
    pub length: usize,
}

// Maintained finger:
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Finger {
    pub target_id: RingKey,
    pub schain: SemiChain,
    version: usize,
}

pub struct SortedFingersLeft {
    pub sorted_fingers: Vec<Finger>,
}

pub struct SortedFingersRight {
    pub sorted_fingers: Vec<Finger>,
}

pub struct NodeFingers {
    id: RingKey,
    pub left: SortedFingersLeft,
    pub right: SortedFingersRight,
    version: usize, // Current version, used for caching.
    updated_by: HashMap<RingKey, usize>,
}


/// Check if proposed new chain is better for the right finger.
fn is_right_finger_better(finger: &Finger, schain: &SemiChain, l:usize) -> bool {
    let cur_dist = (vdist(finger.target_id, finger.schain.final_id,l), finger.schain.length);
    let new_dist = (vdist(finger.target_id, schain.final_id,l), schain.length);
    new_dist < cur_dist
}

/// Check if proposed new chain is better for the left finger.
fn is_left_finger_better(finger: &Finger, schain: &SemiChain, l:usize) -> bool {
    let cur_dist = (vdist(finger.schain.final_id, finger.target_id, l), finger.schain.length);
    let new_dist = (vdist(schain.final_id, finger.target_id, l), schain.length);
    new_dist < cur_dist
}


impl SortedFingersRight {
    /// Add a new known chain, possibly update some fingers to use a new chain.
    /// Returns true if any chain was updated.
    fn update(&mut self, schain: &SemiChain,l: usize, version: usize) -> bool {
        let mut has_changed: bool = false;

        let fingers_len = self.sorted_fingers.len();
        // Find the last index where sorted_fingers[i].target_id <= schain.final_id:
        let last_index = (match self.sorted_fingers.binary_search_by_key(
            &schain.final_id, |finger| finger.target_id) {
            Ok(index) => index,
            Err(index) => (index + fingers_len - 1) % fingers_len,
        }) % self.sorted_fingers.len();

        let mut cur_index: usize = last_index;
        while is_right_finger_better(&self.sorted_fingers[cur_index], &schain, l) {
            self.sorted_fingers[cur_index].schain = schain.clone();
            self.sorted_fingers[cur_index].version = version;
            has_changed = true;
            cur_index = (cur_index + fingers_len - 1) % fingers_len;
        }
        has_changed
    }

    /// Check if chosen semi chains tips are optimal with respect to target_id.
    fn is_optimal(&self, sorted_keys: &Vec<RingKey>) -> bool {
        for fing in &self.sorted_fingers {
            let best_key = match sorted_keys.binary_search(&fing.target_id) {
                Ok(index) => sorted_keys[index],
                Err(index) => sorted_keys[index % sorted_keys.len()],
            };

            if fing.schain.final_id != best_key {
                return false
            }
        }
        return true;
    }

}

impl SortedFingersLeft {
    /// Add a new known chain, possibly update some fingers to use a new chain.
    /// Returns true if any chain was updated.
    fn update(&mut self, schain: &SemiChain,l: usize, version: usize) -> bool {
        let mut has_changed: bool = false;

        let fingers_len = self.sorted_fingers.len();
        // Find the first index where sorted_fingers[i].target_id >= chain[0]:
        let first_index = (match self.sorted_fingers.binary_search_by_key(
            &schain.final_id, |finger| finger.target_id) {
            Ok(index) => index,
            Err(index) => index % fingers_len,
        }) % self.sorted_fingers.len();

        let mut cur_index: usize = first_index;

        while is_left_finger_better(&self.sorted_fingers[cur_index], &schain, l) {
            self.sorted_fingers[cur_index].schain = schain.clone();
            self.sorted_fingers[cur_index].version = version;
            has_changed = true;
            cur_index = (cur_index + 1) % fingers_len;
        }
        has_changed
    }

    /// Check if chosen semi chains tips are optimal with respect to target_id.
    fn is_optimal(&self, sorted_keys: &Vec<RingKey>) -> bool {
        for fing in &self.sorted_fingers {
            let best_key = match sorted_keys.binary_search(&fing.target_id) {
                Ok(index) => sorted_keys[index],
                Err(index) => sorted_keys[
                    (index + sorted_keys.len() - 1) % sorted_keys.len()],
            };

            if fing.schain.final_id != best_key {
                return false
            }
        }
        return true;
    }
}


impl NodeFingers {
    pub fn new(x_id: RingKey, target_ids_left: &Vec<RingKey>, 
           target_ids_right: &Vec<RingKey>) -> NodeFingers {

        let mut nf = NodeFingers {
            id: x_id,
            left: SortedFingersLeft {sorted_fingers: Vec::new()},
            right: SortedFingersRight {sorted_fingers: Vec::new()},
            version: 0,
            updated_by: HashMap::new(),
        };


        // Insert all left fingers:
        for &target_id in target_ids_left {
            nf.left.sorted_fingers.push(
                Finger{
                    target_id: target_id, 
                    schain: SemiChain {
                        final_id: x_id,
                        length: 0,
                    },
                    version: 1,
                },
            );
        }

        // Insert all right fingers:
        for &target_id in target_ids_right {
            nf.right.sorted_fingers.push(
                Finger{
                    target_id: target_id, 
                    schain: SemiChain {
                        final_id: x_id,
                        length: 0,
                    },
                    version: 1,
                }
            );
        }

        // Sort all fingers:
        nf.left.sorted_fingers.sort_by_key(|finger| finger.target_id);
        nf.right.sorted_fingers.sort_by_key(|finger| finger.target_id);

        nf
    }

    /// Add a new known chain, possibly updating existing fingers.
    /// Returns true if any finger was updated.
    pub fn update(&mut self, schain: &SemiChain, l: usize) -> bool {
        let mut has_changed: bool = false;
        self.version += 1;
        has_changed |= self.left.update(&schain, l, self.version);
        has_changed |= self.right.update(&schain, l, self.version);

        // Version is increased only if anything has changed:
        if !has_changed {
            self.version -= 1;
        }

        has_changed
    }

    /// Check if fingers are keys global-optimal
    pub fn is_optimal(&self, sorted_keys: &Vec<RingKey>) -> bool {
        if !self.left.is_optimal(&sorted_keys) {
            return false;
        }
        if !self.right.is_optimal(&sorted_keys) {
            return false;
        }
        return true;
    }

    /// Get all node ids that this node is connected to using
    /// chains.
    pub fn all_schains(&self) -> Vec<SemiChain> {
        let mut unique_schains: HashSet<SemiChain> = HashSet::new();
        for fing in &self.left.sorted_fingers {
            unique_schains.insert(fing.schain.clone());
        }
        for fing in &self.right.sorted_fingers {
            unique_schains.insert(fing.schain.clone());
        }

        let mut unique_schains_vec = unique_schains.into_iter().collect::<Vec<SemiChain>>();
        unique_schains_vec.sort_by_key(|schain| (schain.final_id, schain.length));
        unique_schains_vec
    }

    /// Get all node ids that this node is connected to using
    /// chains.
    pub fn all_fingers(&self) -> Vec<Finger> {
        let mut unique_fingers: HashSet<Finger> = HashSet::new();
        for fing in &self.left.sorted_fingers {
            unique_fingers.insert(fing.clone());
        }
        for fing in &self.right.sorted_fingers {
            unique_fingers.insert(fing.clone());
        }

        let mut unique_fingers_vec = unique_fingers.into_iter().collect::<Vec<Finger>>();
        unique_fingers_vec.sort_by_key(|fing| 
                   (fing.target_id, fing.schain.final_id, fing.schain.length));
        unique_fingers_vec
    }

    /// Update finger's struct by all fingers in fingers_src,
    /// assuming that there is a connecting chain between the two
    /// of length chain_length.
    /// Return if any finger in self has changed.
    pub fn update_by_fingers(&mut self, fingers_src: &NodeFingers, 
                 chain_length: usize, l:usize) -> bool {

        // Get last_version we have of fingers_src.
        // 0 is a reserved version number, which means we know nothing of fingers_src.
        let last_version = match self.updated_by.get(&fingers_src.id) {
            Some(&last_version) => last_version,
            None => 0,
        };

        // It is not possible that we know a newer version of fingers_src
        // than fingers_src knows of.
        assert!(last_version <= fingers_src.version);

        if last_version == fingers_src.version {
            // We are already updated about this version of fingers_src.
            return false;
        }

        let mut has_changed = false; // Has any finger changed?

        for Finger {schain, version, .. } in fingers_src.all_fingers() {
            if last_version >= version {
                // We don't consider this finger if its version is too old.
                continue
            }
            let new_schain = SemiChain {
                final_id: schain.final_id,
                length: schain.length + chain_length,
            };

            has_changed |= self.update(&new_schain, l);
        }

        // Update known version of fingers_src:
        self.updated_by.insert(fingers_src.id, fingers_src.version);

        has_changed
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    /// Get finger structure by target_id. Used for testing and debugging
    fn finger_by_target_id(sorted_fingers: &Vec<Finger>, target_id: RingKey) -> Option<&Finger> {
        match sorted_fingers.binary_search_by_key(&target_id, |finger| finger.target_id) {
            Ok(index) => Some(&sorted_fingers[index]),
            Err(index) => None
        }
    }

    /* Right fingers */

    fn make_sorted_fingers_right() -> SortedFingersRight {
        let mut sfr = SortedFingersRight {
            sorted_fingers: Vec::new(),
        };
        sfr.sorted_fingers.push(Finger {
            target_id: 11,
            schain: SemiChain {
                final_id: 14,
                length: 5
            },
            version: 3,
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 12,
            schain: SemiChain {
                final_id: 14,
                length: 5
            },
            version: 4,
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 15,
            schain: SemiChain {
                final_id: 17,
                length: 4
            },
            version: 2,
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 18,
            schain: SemiChain {
                final_id: 14,
                length: 5
            },
            version: 2,
        });
        sfr.sorted_fingers.sort_by_key(|finger| finger.target_id);

        sfr
    }

    #[test]
    fn test_sorted_right_fingers_one_changed() {
        let mut sfr = make_sorted_fingers_right();
        let sc = SemiChain {
            final_id: 11,
            length: 4
        };
        assert!(sfr.update(&sc, 7, 2));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 11).unwrap().schain == sc);
    }

    #[test]
    fn test_sorted_right_fingers_unchanged() {
        let mut sfr = make_sorted_fingers_right();
        let sc = SemiChain {
            final_id: 17,
            length: 4
        };
        assert!(!sfr.update(&sc, 7, 4));
    }

    #[test]
    fn test_sorted_right_fingers_change_both() {
        let mut sfr = make_sorted_fingers_right();
        let sc = SemiChain {
            final_id: 13,
            length: 4
        };
        assert!(sfr.update(&sc, 7, 6));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 11).unwrap().schain == sc);
        assert!(finger_by_target_id(&sfr.sorted_fingers, 12).unwrap().schain == sc);
    }

    #[test]
    fn test_sorted_right_fingers_change_cyclic() {
        let mut sfr = make_sorted_fingers_right();
        let sc = SemiChain {
            final_id: 2,
            length: 4
        };
        assert!(sfr.update(&sc, 7, 8));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 18).unwrap().schain == sc);
    }

    /* Left fingers */

    fn make_sorted_fingers_left() -> SortedFingersLeft {
        let mut sfl = SortedFingersLeft {
            sorted_fingers: Vec::new(),
        };
        sfl.sorted_fingers.push(Finger {
            target_id: 5,
            schain: SemiChain {
                final_id: 21,
                length: 5
            },
            version: 1,
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 11,
            schain: SemiChain {
                final_id: 9,
                length: 5
            },
            version: 0,
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 12,
            schain: SemiChain {
                final_id: 9,
                length: 5
            },
            version: 2,
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 15,
            schain: SemiChain {
                final_id: 13,
                length: 4
            },
            version: 2,
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 18,
            schain: SemiChain {
                final_id: 16,
                length: 3
            },
            version: 6,
        });
        sfl.sorted_fingers.sort_by_key(|finger| finger.target_id);

        sfl
    }

    #[test]
    fn test_sorted_left_fingers_one_changed() {
        let mut sfr = make_sorted_fingers_left();
        let sc = SemiChain {
            final_id: 12,
            length: 4
        };
        assert!(sfr.update(&sc,7, 9));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 12).unwrap().schain == sc);
    }

    #[test]
    fn test_sorted_left_fingers_unchanged() {
        let mut sfr = make_sorted_fingers_left();
        let sc = SemiChain {
            final_id: 8,
            length: 4
        };
        assert!(!sfr.update(&sc, 7, 10));
    }

    #[test]
    fn test_sorted_left_fingers_change_both() {
        let mut sfr = make_sorted_fingers_left();
        let sc = SemiChain {
            final_id: 10,
            length: 4
        };
        assert!(sfr.update(&sc, 7, 11));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 11).unwrap().schain == sc);
        assert!(finger_by_target_id(&sfr.sorted_fingers, 12).unwrap().schain == sc);
    }

    #[test]
    fn test_sorted_left_fingers_change_cyclic() {
        let mut sfr = make_sorted_fingers_left();
        let sc = SemiChain {
            final_id: 29,
            length: 4
        };
        assert!(sfr.update(&sc, 7, 12));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 5).unwrap().schain == sc);
    }

    /* ***************************************************** */

    #[test]
    fn test_node_fingers_basic() {
        let mut nf = NodeFingers::new(6, &vec![1,3,7,11,54], &vec![5]);
        let sc = SemiChain {
            final_id: 3,
            length: 4
        };
        assert!(nf.update(&sc,7));
        let sc = SemiChain {
            final_id: 5,
            length: 4
        };
        assert!(nf.update(&sc,7));
        let sc = SemiChain {
            final_id: 6,
            length: 4
        };
        assert!(!nf.update(&sc,7));

        let mut all_schains = nf.all_schains();
        assert!(all_schains.len() > 0);

    }
}
