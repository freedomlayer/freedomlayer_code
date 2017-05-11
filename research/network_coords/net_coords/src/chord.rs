extern crate petgraph;

use std::collections::{HashMap};
use self::petgraph::graphmap::NodeTrait;
use network::{Network};

type RingKey = u64; // A key in the chord ring
type NodeChain = Vec<RingKey>;
type NeighborConnector = Vec<NodeChain>;

// Size of keyspace is 2^L:
const L: usize = 42;

pub struct ChordFingers {
    left: NodeChain<>, 
    right_positive: Vec<NodeChain>,
    right_negative: Vec<NodeChain>,
    // Connectors for neighbors:
    neighbor_connectors: Vec<NeighborConnector>,

    right_randomized: Vec<NodeChain>,
    // Additional random nodes from the keyspace:
    rand_nodes: Vec<NodeChain>, 
}

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

pub struct IdsChain {
    cur_id: RingKey, // Current id
    dst_id: RingKey, // Destination id
}

/// Find the msb bit index of a given number.
fn get_msb(mut x: RingKey) -> Option<usize> {
    match x {
        0 => None,
        _ => {
            let mut index: usize = 0;
            while x > 0 {
                x >>= 1;
                index += 1;
            }
            Some(index)
        }
    }
}

///
/// Iterator for a chain of ids between some source id and a destination id.
/// Every two adjacent produced ids have a difference which is an exact
/// power of 2.
/// This iterator is guaranteed to be deterministic. (It will return the same
/// chain for the same source and destination ids every time).
impl Iterator for IdsChain {
    type Item = RingKey;
    fn next(&mut self) -> Option<RingKey> {
        if self.cur_id == self.dst_id {
            // We have already arrived:
            return None
        }

        // Find the most significant different bit between cur_id and dst_id:
        let msb_diff: usize = get_msb(self.cur_id ^ self.dst_id).unwrap();

        // Check if we need to add or to subtract:
        let pow_diff: RingKey = 2_u64.pow(msb_diff as u32);
        if (self.cur_id >> msb_diff) & 1 == 0 {
            self.cur_id += pow_diff;
        } else {
            self.cur_id -= pow_diff;
        }
        Some(self.cur_id)
    }
}

fn ids_chain(src_id: RingKey, dst_id: RingKey) -> IdsChain {
    IdsChain {
        cur_id: src_id,
        dst_id: dst_id,
    }
}

/// Calculate ring distance from x to y clockwise
fn vdist(xk:RingKey, yk: RingKey) -> RingKey {
    (yk - xk) % 2_u64.pow(L as u32)
}


fn extract_chains<'a> (fingers: &'a ChordFingers) -> 
    Vec<&'a NodeChain> {

    let mut res: Vec<&NodeChain> = Vec::new();
    res.push(&fingers.left);
    {
        let mut push_chains = |chains: &'a Vec<NodeChain>| {
            for chain in chains {
                res.push(chain);
            }
        };
        push_chains(&fingers.right_positive);
        push_chains(&fingers.right_negative);
        push_chains(&fingers.right_randomized);
        for conn in &fingers.neighbor_connectors {
            push_chains(&conn)
        }
    }

    res
}

/// Pass over a chain of node ids. Remove cycles of node ids.
fn remove_cycles(chain: &NodeChain) {
}



/// Perform one fingers iteration for node x: 
/// Take all chains from neighbors and update own chains to the best found chains.
fn iter_fingers<Node: NodeTrait>(x_i: usize, net: Network<Node>, 
             index_id: &IndexId, fingers: &mut Vec<ChordFingers>) {

    let x_id: RingKey = index_id.index_to_id(x_i).unwrap();

    // Collect all chains to one vector. 
    let mut all_chains: Vec<NodeChain> = Vec::new();

    // Add trivial chain (x):
    all_chains.push(vec![x_id]);

    // Add trivial chains (x,nei) where nei is any neighbor of x:
    for neighbor_index in net.igraph.neighbors(x_i) {
        all_chains.push(vec![index_id.index_to_id(neighbor_index).unwrap(), x_id])
    }

    // Add all current chains:
    all_chains.extend(
        extract_chains(&fingers[x_i]).iter().map(|&chain| chain.clone())
    );

    fingers[x_i].left = all_chains.iter().min_by_key(|c| (vdist(c[0], x_id), c.len()) ).unwrap().clone();
    for i in 0 .. L {
        fingers[x_i].right_positive[i] = 
            all_chains.iter().min_by_key(|c| (vdist((x_id + 2_u64.pow(i as u32)) % 2_u64.pow(L as u32), c[0]), c.len()) ).unwrap().clone();
    }
    for i in 0 .. L {
        fingers[x_i].right_negative[i] = 
            all_chains.iter().min_by_key(|c| (vdist((x_id - 2_u64.pow(i as u32)) % 2_u64.pow(L as u32), c[0]), c.len()) ).unwrap().clone();
    }

    for neighbor_index in net.igraph.neighbors(x_i) {
        
    }
    // fingers.neighbor_connectors


    // For every maintained chain: Find the best chain.
    //  - Closest to wanted target.
    //  - Shortest possible.
    //      - Eliminate cycles?
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d() {
        assert!(d(1u64,2) == 1);
        assert!(d(1u64,101) == 100);
        assert!(d(0u64.wrapping_sub(1),1) == 2);
        assert!(d(0u64.wrapping_sub(1),0) == 1);
        assert!(d(1,0) == 0u64.wrapping_sub(1));
    }
}
