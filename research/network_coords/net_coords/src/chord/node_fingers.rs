extern crate itertools;

use network::{Network};
use chord::{RingKey, NodeChain, add_cyc, vdist};
use std::{iter, slice};

// Maintained finger:
struct Finger {
    target_id: RingKey,
    chain: NodeChain,
}

struct SortedFingersLeft {
    sorted_fingers: Vec<Finger>,
}

struct SortedFingersRight {
    sorted_fingers: Vec<Finger>,
}

pub struct NodeFingers {
    left: SortedFingersLeft,
    right: SortedFingersRight,
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
fn is_right_finger_better(finger: &Finger, chain: &NodeChain, l:usize) -> bool {
    let cur_dist = (vdist(finger.target_id, finger.chain[0],l), finger.chain.len(), &finger.chain);
    let new_dist = (vdist(finger.target_id, chain[0],l), chain.len(), chain);
    new_dist < cur_dist
}

/// Check if proposed new chain is better for the left finger.
fn is_left_finger_better(finger: &Finger, chain: &NodeChain, l:usize) -> bool {
    let cur_dist = (vdist(finger.chain[0], finger.target_id, l), finger.chain.len(), &finger.chain);
    let new_dist = (vdist(chain[0], finger.target_id, l), chain.len(), chain);
    new_dist < cur_dist
}


impl SortedFingersRight {
    /// Add a new known chain, possibly update some fingers to use a new chain.
    /// Returns true if any chain was updated.
    fn update(&mut self, chain: &NodeChain,l: usize) -> bool {
        let mut has_changed: bool = false;

        let fingers_len = self.sorted_fingers.len();
        // Find the last index where sorted_fingers[i].target_id <= chain[0]:
        let last_index = (match self.sorted_fingers.binary_search_by_key(
            &chain[0], |finger| finger.target_id) {
            Ok(index) => index,
            Err(index) => (index + fingers_len - 1) % fingers_len,
        }) % self.sorted_fingers.len();

        let mut cur_index: usize = last_index;
        
        while is_right_finger_better(&self.sorted_fingers[cur_index], &chain, l) {
            self.sorted_fingers[cur_index].chain = chain.clone();
            has_changed = true;
            cur_index = (cur_index + fingers_len - 1) % fingers_len;
        }
        has_changed
    }

}

impl SortedFingersLeft {
    /// Add a new known chain, possibly update some fingers to use a new chain.
    /// Returns true if any chain was updated.
    fn update(&mut self, chain: &NodeChain,l: usize) -> bool {
        let mut has_changed: bool = false;

        let fingers_len = self.sorted_fingers.len();
        // Find the first index where sorted_fingers[i].target_id >= chain[0]:
        let first_index = (match self.sorted_fingers.binary_search_by_key(
            &chain[0], |finger| finger.target_id) {
            Ok(index) => index,
            Err(index) => index % fingers_len,
        }) % self.sorted_fingers.len();

        let mut cur_index: usize = first_index;

        while is_left_finger_better(&self.sorted_fingers[cur_index], &chain, l) {
            self.sorted_fingers[cur_index].chain = chain.clone();
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
                    chain: vec![x_id],
                }
            );
        }

        // Insert all right fingers:
        for &target_id in target_ids_right {
            nf.right.sorted_fingers.push(
                Finger{
                    target_id: target_id, 
                    chain: vec![x_id],
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
    pub fn update(&mut self, chain: &NodeChain, l: usize) -> bool {
        let mut has_changed: bool = false;
        has_changed |= self.left.update(&chain, l);
        has_changed |= self.right.update(&chain, l);

        has_changed
    }

    pub fn all_chains(&self) -> Vec<NodeChain> {
        self.left.sorted_fingers.iter().map(|finger| finger.chain.clone())
            .chain(self.right.sorted_fingers.iter().map(|finger| finger.chain.clone()))
            .collect::<Vec<NodeChain>>()
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
            chain: vec![14,6,4,8,1],
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 12,
            chain: vec![14,6,4,8,1],
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 15,
            chain: vec![17,6,4,1],
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 18,
            chain: vec![14,6,4,8,1],
        });
        sfr.sorted_fingers.sort_by_key(|finger| finger.target_id);

        sfr
    }

    #[test]
    fn test_sorted_right_fingers_one_changed() {
        let mut sfr = make_sorted_fingers_right();
        assert!(sfr.update(&vec![11,9,3,5], 7));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 11).unwrap().chain == vec![11,9,3,5]);
    }

    #[test]
    fn test_sorted_right_fingers_unchanged() {
        let mut sfr = make_sorted_fingers_right();
        assert!(!sfr.update(&vec![17,8,9,10], 7));
    }

    #[test]
    fn test_sorted_right_fingers_change_both() {
        let mut sfr = make_sorted_fingers_right();
        assert!(sfr.update(&vec![13,2,3,5], 7));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 11).unwrap().chain == vec![13,2,3,5]);
        assert!(finger_by_target_id(&sfr.sorted_fingers, 12).unwrap().chain == vec![13,2,3,5]);
    }

    #[test]
    fn test_sorted_right_fingers_change_cyclic() {
        let mut sfr = make_sorted_fingers_right();
        assert!(sfr.update(&vec![2,6,3,5], 7));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 18).unwrap().chain == vec![2,6,3,5]);
    }

    /* Left fingers */

    fn make_sorted_fingers_left() -> SortedFingersLeft {
        let mut sfl = SortedFingersLeft {
            sorted_fingers: Vec::new(),
        };
        sfl.sorted_fingers.push(Finger {
            target_id: 5,
            chain: vec![21,7,4,8,1],
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 11,
            chain: vec![9,8,4,8,1],
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 12,
            chain: vec![9,8,4,8,1],
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 15,
            chain: vec![13,6,9,1],
        });
        sfl.sorted_fingers.push(Finger {
            target_id: 18,
            chain: vec![16,8,1],
        });
        sfl.sorted_fingers.sort_by_key(|finger| finger.target_id);

        sfl
    }

    #[test]
    fn test_sorted_left_fingers_one_changed() {
        let mut sfr = make_sorted_fingers_left();
        assert!(sfr.update(&vec![12,8,3,5], 7));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 12).unwrap().chain == vec![12,8,3,5]);
    }

    #[test]
    fn test_sorted_left_fingers_unchanged() {
        let mut sfr = make_sorted_fingers_left();
        assert!(!sfr.update(&vec![8,4,1,10], 7));
    }

    #[test]
    fn test_sorted_left_fingers_change_both() {
        let mut sfr = make_sorted_fingers_left();
        assert!(sfr.update(&vec![10,2,6,5], 7));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 11).unwrap().chain == vec![10,2,6,5]);
        assert!(finger_by_target_id(&sfr.sorted_fingers, 12).unwrap().chain == vec![10,2,6,5]);
    }

    #[test]
    fn test_sorted_left_fingers_change_cyclic() {
        let mut sfr = make_sorted_fingers_left();
        assert!(sfr.update(&vec![29,14,3,6], 7));
        assert!(finger_by_target_id(&sfr.sorted_fingers, 5).unwrap().chain == vec![29,14,3,6]);
    }

    /* ***************************************************** */

    #[test]
    fn test_node_fingers_basic() {
        let mut nf = NodeFingers::new(6, &vec![1,3,7,11,54], &vec![5]);
        assert!(nf.update(&vec![5,6,7,8],7));
        assert!(!nf.update(&vec![5,6,7,8],7));
        assert!(!nf.update(&vec![6,6,7,8],7));

        assert!(nf.all_chains().len() > 0);

    }
}
