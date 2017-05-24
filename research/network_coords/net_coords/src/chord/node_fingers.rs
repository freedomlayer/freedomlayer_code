extern crate itertools;

use network::{Network};
use chord::{RingKey, NodeChain, add_cyc, vdist};
use std::{iter, slice};
use std::collections::{HashSet};

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct SemiChain {
    pub final_id: RingKey,
    pub length: usize,
}

// Maintained finger:
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Finger {
    pub target_id: RingKey,
    pub schain: SemiChain,
}

pub struct SortedFingersLeft {
    pub sorted_fingers: Vec<Finger>,
}

pub struct SortedFingersRight {
    pub sorted_fingers: Vec<Finger>,
}

pub struct NodeFingers {
    pub left: SortedFingersLeft,
    pub right: SortedFingersRight,
}


/*
fn left_chain_key(chain: &NodeChain) -> (i64, usize, u64) {
    (-(chain[0] as i64), chain.len(), csum_chain(chain))
}

fn right_chain_key(chain: &NodeChain) -> (i64, usize, u64) {
    (chain[0] as i64, chain.len(), csum_chain(chain))
}
*/

/*
/// Check if three given points on the ring are clockwise ordered
/// In other words, check if b \in [a,c]
fn is_ordered(a: RingKey, b: RingKey, c: RingKey, l: usize) -> bool {
    vdist(a,b,l) + vdist(b,c,l) == vdist(a,c,l)
}
*/

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
    fn update(&mut self, schain: &SemiChain,l: usize) -> bool {
        let mut has_changed: bool = false;

        let fingers_len = self.sorted_fingers.len();
        // Find the last index where sorted_fingers[i].target_id <= chain[0]:
        let last_index = (match self.sorted_fingers.binary_search_by_key(
            &schain.final_id, |finger| finger.target_id) {
            Ok(index) => index,
            Err(index) => (index + fingers_len - 1) % fingers_len,
        }) % self.sorted_fingers.len();

        let mut cur_index: usize = last_index;
        
        while is_right_finger_better(&self.sorted_fingers[cur_index], &schain, l) {
            self.sorted_fingers[cur_index].schain = schain.clone();
            has_changed = true;
            cur_index = (cur_index + fingers_len - 1) % fingers_len;
        }
        has_changed
    }

}

impl SortedFingersLeft {
    /// Add a new known chain, possibly update some fingers to use a new chain.
    /// Returns true if any chain was updated.
    fn update(&mut self, schain: &SemiChain,l: usize) -> bool {
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
            has_changed = true;
            cur_index = (cur_index + 1) % fingers_len;
        }
        has_changed
    }
}


impl NodeFingers {
    pub fn new(x_id: RingKey, target_ids_left: &Vec<RingKey>, 
           target_ids_right: &Vec<RingKey>) -> NodeFingers {

        let mut nf = NodeFingers {
            left: SortedFingersLeft {sorted_fingers: Vec::new()},
            right: SortedFingersRight {sorted_fingers: Vec::new()}
        };


        // Insert all left fingers:
        for &target_id in target_ids_left {
            nf.left.sorted_fingers.push(
                Finger{
                    target_id: target_id, 
                    schain: SemiChain {
                        final_id: x_id,
                        length: 0,
                    }
                }
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
                    }
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
        has_changed |= self.left.update(&schain, l);
        has_changed |= self.right.update(&schain, l);

        has_changed
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

        unique_schains.into_iter().collect::<Vec<SemiChain>>()
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

        unique_fingers.into_iter().collect::<Vec<Finger>>()
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
            }
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 12,
            schain: SemiChain {
                final_id: 14,
                length: 5
            }
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 15,
            schain: SemiChain {
                final_id: 17,
                length: 4
            }
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 18,
            schain: SemiChain {
                final_id: 14,
                length: 5
            }
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
        assert!(sfr.update(&sc, 7));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 11).unwrap().schain == sc);
    }

    #[test]
    fn test_sorted_right_fingers_unchanged() {
        let mut sfr = make_sorted_fingers_right();
        let sc = SemiChain {
            final_id: 17,
            length: 4
        };
        assert!(!sfr.update(&sc, 7));
    }

    #[test]
    fn test_sorted_right_fingers_change_both() {
        let mut sfr = make_sorted_fingers_right();
        let sc = SemiChain {
            final_id: 13,
            length: 4
        };
        assert!(sfr.update(&sc, 7));
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
        assert!(sfr.update(&sc, 7));
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
            }
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 11,
            schain: SemiChain {
                final_id: 9,
                length: 5
            }
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 12,
            schain: SemiChain {
                final_id: 9,
                length: 5
            }
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 15,
            schain: SemiChain {
                final_id: 13,
                length: 4
            }
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 18,
            schain: SemiChain {
                final_id: 16,
                length: 3
            }
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
        assert!(sfr.update(&sc,7));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 12).unwrap().schain == sc);
    }

    #[test]
    fn test_sorted_left_fingers_unchanged() {
        let mut sfr = make_sorted_fingers_left();
        let sc = SemiChain {
            final_id: 8,
            length: 4
        };
        assert!(!sfr.update(&sc, 7));
    }

    #[test]
    fn test_sorted_left_fingers_change_both() {
        let mut sfr = make_sorted_fingers_left();
        let sc = SemiChain {
            final_id: 10,
            length: 4
        };
        assert!(sfr.update(&sc, 7));
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
        assert!(sfr.update(&sc, 7));
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
