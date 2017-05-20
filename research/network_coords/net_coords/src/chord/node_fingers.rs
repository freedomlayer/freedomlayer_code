use network::{Network};
use chord::{RingKey, NodeChain, add_cyc, vdist};

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

/// Get finger structure by target_id. Used for testing and debugging
fn finger_by_target_id(sorted_fingers: &Vec<Finger>, target_id: RingKey) -> Option<&Finger> {
    match sorted_fingers.binary_search_by_key(&target_id, |finger| finger.target_id) {
        Ok(index) => Some(&sorted_fingers[index]),
        Err(index) => None
    }
}

impl SortedFingersRight {
    /// Possibly update some fingers to use a new chain.
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
    /// Possibly update some fingers to use a new chain.
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


/*
impl NodeFingers {
    fn new(x_id: RingKey) -> NodeFingers {

    }
}
*/

/*
/// Generate all ids to maintain for chord 
fn gen_node_fingers(x_id: RingKey, net: &Network<RingKey>, l: usize, mut rng: &mut StdRng) 
    -> NodeFingers {

    assert!(false);

}
*/

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_sorted_fingers_right() {
        let mut sfr = SortedFingersRight {
            sorted_fingers: Vec::new(),
        };
        sfr.sorted_fingers.push(Finger {
            target_id: 15,
            chain: vec![17,6,4,1],
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 12,
            chain: vec![14,6,4,8,1],
        });
        sfr.sorted_fingers.push(Finger {
            target_id: 11,
            chain: vec![14,6,4,8,1],
        });
        sfr.update(&vec![11,9,3,5], 7);

        assert!(finger_by_target_id(&sfr.sorted_fingers, 11).unwrap().chain == vec![11,9,3,5]);
    }
}
