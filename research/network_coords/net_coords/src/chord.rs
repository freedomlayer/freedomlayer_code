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
    right_randomized: Vec<NodeChain>,
    // Connectors for neighbors:
    neighbor_connectors: Vec<NeighborConnector>,
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

/// Calculate ring distance from x to y clockwise
fn d(xk:RingKey, yk: RingKey) -> RingKey {
    yk.wrapping_sub(xk)
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

    // Collect all chains to one vector. 
    let mut all_chains: Vec<NodeChain> = Vec::new();

    // Add trivial chain to x:
    all_chains.push(vec![index_id.index_to_id(x_i).unwrap()]);

    // Add trivial chains to all neighbors:
    for neighbor_index in net.igraph.neighbors(x_i) {
        all_chains.push(vec![index_id.index_to_id(neighbor_index).unwrap(), index_id.index_to_id(x_i).unwrap()])
    }

    // Add all current chains:
    all_chains.extend(
        extract_chains(&fingers[x_i]).iter().map(|&chain| chain.clone())
    );

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
