
extern crate petgraph;
extern crate rand;

pub mod ids_chain;
pub mod semi_chains_array;
pub mod node_fingers;
pub mod network_gen;

use std::collections::{HashSet};

use self::rand::{Rng};
use self::rand::distributions::{IndependentSample, Range};

use network::{Network};
use self::ids_chain::{ids_chain};
use self::semi_chains_array::{SemiChainsArray};
use self::node_fingers::{NodeFingers, SemiChain};
use index_pair::{index_pair, Pair};


pub type RingKey = u64; // A key in the chord ring
pub type NodeChain = Vec<RingKey>;
pub type NeighborConnector = Vec<NodeChain>;


/*
 * TODO:
 * - Write a function to calculate graph's average path length 
 *   (Between two average nodes).
 * - Check for graphs of various shapes: 
 *      - Random graphs,
 *      - Random graphs with constant amount of neighbors,
 *      - Grid (2d)
 */


/// Calculate ring distance from x to y clockwise
fn vdist(xk:RingKey, yk: RingKey, l: usize) -> RingKey {
    (yk.wrapping_sub(xk)) % 2_u64.pow(l as u32)
}

/// Add cyclic (x + diff) % max_key
fn add_cyc(x: RingKey, diff: i64, l: usize) -> RingKey {
    let max_key = 2_u64.pow(l as u32);
    (if diff >= 0 {
        x.wrapping_add(diff as u64)
    } else {
        x.wrapping_sub((-diff) as u64)
    }) % max_key
}


/// Generate a vector of maintained left target_ids for node with id x_id.
fn gen_left_target_ids(x_id: RingKey, l: usize) -> Vec<RingKey> {
    vec![add_cyc(x_id,-1,l)]
}

/// Generate a vector of maintained right target_ids for node with id x_id.
fn gen_right_target_ids<R: Rng>(x_id: RingKey, net: &Network<RingKey>, 
                                l: usize, mut rng: &mut R) -> Vec<RingKey> {

    let mut target_ids_set: HashSet<RingKey> = HashSet::new();
    // let mut target_ids: Vec<RingKey> = Vec::new();

    // Basic right fingers:
    for i in 0 .. l {
        let pow_val = 2_i64.pow(i as u32);
        target_ids_set.insert(add_cyc(x_id,pow_val,l));
        target_ids_set.insert(add_cyc(x_id,-pow_val,l));
    }

    // Neighbor connectors:
    let x_i = net.node_to_index(&x_id).unwrap();
    let mut neighbors = net.igraph.neighbors(x_i).into_iter().collect::<Vec<_>>();
    neighbors.sort();
    for neighbor_index in neighbors {
        let neighbor_id: RingKey = net.index_to_node(neighbor_index).unwrap().clone();
        for cur_id in ids_chain(x_id, neighbor_id) {
            target_ids_set.insert(cur_id);
        }
    }

    // Right randomized fingers:
    for i in 0 .. l {
        // Randomize a finger value in [2^i, 2^(i+1))
        let rand_range: Range<RingKey> = 
            Range::new(2_u64.pow(i as u32),2_u64.pow((i + 1) as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        target_ids_set.insert(rand_id);
    }

    // Update random fingers:
    for _ in 0 .. l {
        // Randomize a finger value in [0, 2^l). Completely random in the ring key space.
        let rand_range: Range<RingKey> = Range::new(0u64,2_u64.pow(l as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        target_ids_set.insert(rand_id);
    }
    
    target_ids_set.into_iter().collect::<Vec<RingKey>>()

}

/// Initialize maintained fingers for node with index x_i.
fn create_node_fingers<R: Rng>(x_i: usize, net: &Network<RingKey>, 
             l: usize, mut rng: &mut R) -> NodeFingers {

    let x_id = net.index_to_node(x_i).unwrap().clone();
    let target_ids_left = gen_left_target_ids(x_id, l);
    let target_ids_right = gen_right_target_ids(x_id, &net, l, &mut rng);

    let nf = NodeFingers::new(x_id, &target_ids_left, &target_ids_right);
    
    nf
}

pub fn init_fingers<R: Rng>(net: &Network<RingKey>, 
                l: usize, mut rng: &mut R) -> Vec<NodeFingers> {

    let mut res_fingers = Vec::new();
    for x_i in 0 .. net.igraph.node_count() {
        res_fingers.push(create_node_fingers(x_i, &net, l, &mut rng));
    }

    res_fingers
}



/// Perform one iteration of fingers for all nodes
fn iter_fingers(net: &Network<RingKey>, 
                mut fingers: &mut Vec<NodeFingers>, l: usize) -> bool {

    // Check if any finger has changed:
    let mut has_changed = false;

    // Keep iterating until no changes happen:
    for x_i in 0 .. net.igraph.node_count() {
        // let x_id = net.index_to_node(x_i).unwrap().clone();
        // Every node sends an UpdateRequest, and gets back an UpdateResponse message.

        for remote_schain in fingers[x_i].all_schains() {
            let remote_i = net.node_to_index(&remote_schain.final_id).unwrap();

            if x_i == remote_i {
                continue;
            }


            // Get two mutable indices (x_i and remote_i):
            let (m_x_i, m_remote_i) = match index_pair(&mut fingers, x_i, remote_i) {
                Pair::Two(m_x_i,m_remote_i) => (m_x_i, m_remote_i),
                _ => panic!("Invalid index pair: {}, {}", x_i, remote_i),
            };
            
            // UpdateRequest:
            // Every finger of x_id will get all of x_id's fingers.
            has_changed |= m_remote_i.update_by_fingers(&m_x_i, 
                       remote_schain.length, l);

            // UpdateResponse:
            // x_id will get all of the fingers of his fingers
            has_changed |= m_x_i.update_by_fingers(&m_remote_i,
                        remote_schain.length, l);
                
        }
    }

    has_changed
}


/// Get to converging state of fingers for all the network.
pub fn converge_fingers(net: &Network<RingKey>, 
             mut fingers: &mut Vec<NodeFingers>, l: usize) {

    // First iteration: We insert all edges:
    for x_i in 0 .. net.igraph.node_count() {
        let mut neighbors = net.igraph.neighbors(x_i).into_iter().collect::<Vec<_>>();
        neighbors.sort();
        for neighbor_i in neighbors {
            let neighbor_id = net.index_to_node(neighbor_i).unwrap().clone();
            let schain = SemiChain {
                final_id: neighbor_id,
                length: 1,
            };
            fingers[x_i].update(&schain,l);
        }
    }

    // println!("Iterating fingers...");
    // print!(".");
    while iter_fingers(&net, &mut fingers, l) {
        // print!(".");
    }
    // println!("");
}

/// Make sure that every finger reaches the best globally key possible
/// (As closest as possible to its target_id).
pub fn verify_global_optimality(net: &Network<RingKey>, fingers: &Vec<NodeFingers>) -> bool {
    // Obtain a sorted vector of all keys in the network:
    let mut all_keys: Vec<RingKey> = (0 .. net.igraph.node_count())
        .map(|x_i| net.index_to_node(x_i).unwrap().clone())
        .collect::<Vec<_>>();
    all_keys.sort();

    for x_i in 0 .. net.igraph.node_count() {
        if !fingers[x_i].is_optimal(&all_keys) {
            return false;
        }
    }
    return true;
}



fn create_semi_chains_node(x_i: usize, 
                           fingers: &Vec<NodeFingers>) -> SemiChainsArray {

    let mut schains_array = SemiChainsArray::new();

    for schain in fingers[x_i].all_schains() {
        schains_array.insert_schain(schain);
    }

    schains_array.index();
    schains_array
}

pub fn create_semi_chains(net: &Network<RingKey>, 
                          fingers: &Vec<NodeFingers>) -> Vec<SemiChainsArray> {

    let mut res_vec = Vec::new();
    for x_i in 0 .. net.igraph.node_count() {
        res_vec.push(create_semi_chains_node(x_i, &fingers));
    }
    res_vec
}

/// Returns a length of a found path between src_id to dst_id, or 
/// None if no path was found.
pub fn find_path(src_id: RingKey, dst_id: RingKey, net: &Network<RingKey>, 
                 semi_chains: &Vec<SemiChainsArray>) -> Option<usize> {

    let mut cur_id = src_id;
    let mut length: usize = 0;
    while cur_id != dst_id {
        let cur_semi_chains = &semi_chains[net.node_to_index(&cur_id).unwrap()];
        let schain = cur_semi_chains.find_closest_left(dst_id);
        if schain.final_id == cur_id {
            return None;
        }

        length += schain.length;
        cur_id = schain.final_id;
    }
    Some(length)
}






#[cfg(test)]
mod tests {
    use super::*;
    use self::rand::{StdRng};
    use self::network_gen::random_net_chord;

    #[test]
    fn test_d() {
        let l = 42;
        assert!(vdist(1u64,2,l) == 1);
        assert!(vdist(1u64,101,l) == 100);
        assert!(vdist(2_u64.pow(l as u32) - 1,1,l) == 2);
        assert!(vdist(2_u64.pow(l as u32) - 1,0,l) == 1);
        assert!(vdist(1,0,l) == 2_u64.pow(l as u32) - 1);
    }

    #[test]
    fn test_add_cyc() {
        // Check add:
        assert!(add_cyc(0,1,5) == 1);
        assert!(add_cyc(1,1,5) == 2);
        assert!(add_cyc(30,1,5) == 31);
        assert!(add_cyc(31,1,5) == 0);

        // Check sub:
        assert!(add_cyc(2,-1,5) == 1);
        assert!(add_cyc(1,-1,5) == 0);
        assert!(add_cyc(0,-1,5) == 31);
        assert!(add_cyc(31,-1,30) == 30);
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



    #[test]
    fn test_chord_basic() {
        let seed: &[_] = &[1,2,3,4,9];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let num_nodes = 5;
        let num_neighbors = 2;
        let l: usize = 6; // Size of keyspace
        let net = random_net_chord(num_nodes,num_neighbors,l,&mut rng);
        let mut fingers = init_fingers(&net,l, &mut rng);
        converge_fingers(&net, &mut fingers,l);
        assert!(verify_global_optimality(&net, &fingers));
        let semi_chains = create_semi_chains(&net, &fingers);

        for index_a in 0 .. num_nodes {
            for index_b in index_a + 1 .. num_nodes {
                // Try to find a path:
                let src_id = net.index_to_node(index_a).unwrap().clone();
                let dst_id = net.index_to_node(index_b).unwrap().clone();
                let _ = find_path(src_id, dst_id, &net, 
                                      &semi_chains).unwrap();

            }
        }
    }

}
