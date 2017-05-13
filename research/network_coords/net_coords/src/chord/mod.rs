extern crate petgraph;
extern crate rand;

pub mod ids_chain;
pub mod index_id;

use self::petgraph::graphmap::NodeTrait;
use network::{Network};
use self::index_id::{IndexId};
use self::ids_chain::{ids_chain};

use self::rand::{StdRng};
use self::rand::distributions::{Weighted, WeightedChoice, 
    IndependentSample, Range};

type RingKey = u64; // A key in the chord ring
type NodeChain = Vec<RingKey>;
type NeighborConnector = Vec<NodeChain>;

// Size of keyspace is 2^L:
const L: usize = 42;

// Global seed for deterministic randomization of randomized fingers.
const FINGERS_SEED: u64 = 0x1337;


pub struct ChordFingers {
    left: NodeChain<>, 
    right_positive: Vec<NodeChain>,
    right_negative: Vec<NodeChain>,
    // Connectors for neighbors:
    neighbor_connectors: Vec<NeighborConnector>,

    right_randomized: Vec<NodeChain>,

    // Additional random fingers from the keyspace:
    fully_randomized: Vec<NodeChain>, 
}



/// Calculate ring distance from x to y clockwise
fn vdist(xk:RingKey, yk: RingKey) -> RingKey {
    (yk.wrapping_sub(xk)) % 2_u64.pow(L as u32)
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


/// Add an id to chain. Eliminate cycle if created.
fn add_id_to_chain(chain: &mut NodeChain, id: RingKey) {
    match chain.iter().position(|&x| x == id) {
        None => {
            chain.push(id);
        },
        Some(position) => {
            chain.resize(position + 1, 0);
        }
    };
}

/// Prepare all candidate chains for node x_i.
fn prepare_candidates<Node: NodeTrait>(x_i: usize, net: &Network<Node>, index_id: &IndexId, 
                      fingers: &Vec<ChordFingers>) -> Vec<NodeChain> {
    let x_id: RingKey = index_id.index_to_id(x_i).unwrap();

    // Collect all chains to one vector. 
    let mut candidates: Vec<NodeChain> = Vec::new();

    // Add trivial chain (x):
    candidates.push(vec![x_id]);

    // Add trivial chains (x,nei) where nei is any neighbor of x:
    for neighbor_index in net.igraph.neighbors(x_i) {
        candidates.push(vec![index_id.index_to_id(neighbor_index).unwrap(), x_id])
    }

    // Add all "proposed" chains from all neighbors:
    for neighbor_index in net.igraph.neighbors(x_i) {
        // Add trivial chain (x,nei):
        candidates.push(vec![index_id.index_to_id(neighbor_index).unwrap(), x_id]);

        // Add proposed chains:
        for &chain in extract_chains(&fingers[neighbor_index]).iter() {
            let mut cchain = chain.clone();
            // Add our own id to the chain, possibly eliminating cycles:
            add_id_to_chain(&mut cchain, x_id);
            candidates.push(cchain);
        }
    }

    // Add all current chains:
    candidates.extend(
        extract_chains(&fingers[x_i]).iter().map(|&chain| chain.clone())
    );

    candidates
}


/// Checksum the contents of a chain
fn csum_chain(chain: &NodeChain) -> RingKey {
    chain.iter().fold(0, |acc, &x| acc.wrapping_add(x) % (2_u64.pow(L as u32)))
}


/// Perform one fingers iteration for node x: 
/// Take all chains from neighbors and update own chains to the best found chains.
fn iter_fingers<Node: NodeTrait>(x_i: usize, net: &Network<Node>, 
             index_id: &IndexId, fingers: &mut Vec<ChordFingers>) {

    let x_id: RingKey = index_id.index_to_id(x_i).unwrap();

    // Get all chain candidates:
    let candidates = prepare_candidates(x_i, &net,  &index_id, &fingers);

    // Update left finger:
    fingers[x_i].left = candidates.iter().min_by_key(|c: &&NodeChain| 
                                (vdist(c[0], x_id), c.len(), csum_chain(c) )).unwrap().clone();

    // Find the chain that is closest to target_id from the right.
    // Lexicographic sorting: 
    // We first care about closest id in keyspace. Next we want the shortest chain possible.
    let best_right_chain = |target_id| candidates.iter().min_by_key(|c| 
                                 (vdist(target_id, c[0]), c.len(), csum_chain(c) )).unwrap().clone();

    // Update all right fingers:
    for i in 0 .. L {
        fingers[x_i].right_positive[i] = 
            best_right_chain((x_id + 2_u64.pow(i as u32)) % 2_u64.pow(L as u32));
    }
    for i in 0 .. L {
        fingers[x_i].right_negative[i] = 
            best_right_chain((x_id - 2_u64.pow(i as u32)) % 2_u64.pow(L as u32));
    }

    // Update neighbor connectors.
    // For determinism, we sort the neighbors before iterating.
    let mut s_neighbors: Vec<usize> = net.igraph.neighbors(x_i).collect::<Vec<_>>();
    s_neighbors.sort();

    for (neighbor_vec_index, &neighbor_index) in s_neighbors.iter().enumerate() {
        let neighbor_id: RingKey = index_id.index_to_id(neighbor_index).unwrap();

        for (j,cur_id) in ids_chain(x_id, neighbor_id).enumerate() {
            fingers[x_i].neighbor_connectors[neighbor_vec_index][j] = 
                best_right_chain(cur_id);
        }
    }

    // Obtain deterministic rng to be used with the following randomized
    // fingers:
    let seed: &[_] = &[FINGERS_SEED as usize, x_i as usize];
    let mut rng: StdRng = rand::SeedableRng::<&[usize]>::from_seed(seed);

    // Update right randomized fingers:
    for i in 0 .. L {
        // Randomize a finger value in [2^i, 2^(i+1))
        let rand_range: Range<RingKey> = Range::new(2_u64.pow(i as u32),2_u64.pow((i + 1) as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        fingers[x_i].right_randomized[i] = best_right_chain(rand_id);
    }

    // Update random fingers:
    for i in 0 .. L {
        // Randomize a finger value in [0, 2^L). Completely random in the ring key space.
        let rand_range: Range<RingKey> = Range::new(0u64,2_u64.pow(L as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        fingers[x_i].fully_randomized[i] = best_right_chain(rand_id);
    }
}

/// Find next best chain of steps in the network to arrive the node dst_index.
fn next_chain<Node: NodeTrait>(cur_index: usize, dst_index: usize, 
        net: &Network<Node>, index_id: &IndexId, fingers: &Vec<ChordFingers>)
            -> Option<NodeChain>{

    // Get ids on the ring:
    let cur_id: RingKey = index_id.index_to_id(cur_index).unwrap();
    let dst_id: RingKey = index_id.index_to_id(dst_index).unwrap();

    // Get all chains of order 1:
    let chains1 = prepare_candidates(cur_index, &net,  &index_id, &fingers);
    let all_chains: Vec<NodeChain> = chains1.clone();

    for &c1 in chains1.iter() {
        let vneighbor_id: RingKey = c1[0];
        let vneighbor_index: usize = index_id.id_to_index(vneighbor_id).unwrap();

        // Concatenate pairs of chains.
        // Remember that a chain from id x to id y is of the form:
        // 0 1 2 3 4  <-- Vector index
        // y . . . x  <-- Value
        all_chains.extend(
            prepare_candidates(vneighbor_index, &net, &index_id, &fingers).iter()
                .map(|c| c.clone().extend(c1.iter().skip(1)))
        );
    }

    // TODO: Remove this later:
    None

    // Collect all relevant chains:
    // - Regular one iteration chain.
    // - Two iters chains: According to "Know thy neighbor" article

    // Pick the closest chain, with some tiebreaker.
    // Return chosen chain.

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d() {
        assert!(vdist(1u64,2) == 1);
        assert!(vdist(1u64,101) == 100);
        assert!(vdist(2_u64.pow(L as u32) - 1,1) == 2);
        assert!(vdist(2_u64.pow(L as u32) - 1,0) == 1);
        assert!(vdist(1,0) == 2_u64.pow(L as u32) - 1);
    }


    #[test]
    fn test_csum_chain() {
        assert!(csum_chain(&vec![1,2,3,4]) == 10);
        assert!(csum_chain(&vec![]) == 0);
        assert!(csum_chain(&vec![1]) == 1);
    }

    #[test]
    fn test_inner_lexicographic() {
        // Make sure that vectors participate inside
        // lexicographic comparison.
        let d = (1,2,vec![3,6]);
        let a = (1,2,vec![3,4]);
        let b = (1,2,vec![3,5]);
        let c = (1,2,vec![3,6]);

        assert!(a < b);
        assert!(a < c);
        assert!(a < d);

        let aa = (1,2,vec![4,4]);
        assert!(aa > a);

        let m = (5,2,vec![1,1]);
        assert!(m > a);
    }
}
